use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

use rocket::http::{Cookie, Cookies};
use rocket::request::Request;
use rocket::request::State;
use rocket_contrib::Template;

use cacher::Cacher;
use config;
use data;
use lang;

lazy_static! {
    static ref cacher: Mutex<Cacher<String, Vec<data::PageInfo>>> = Mutex::new(Cacher::new(
        Duration::from_secs(600),
        Duration::from_secs(1000)
    ));
}

// mount routes for an already ignited rocket.
pub fn mount_handlers(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
        .mount("/", routes![index, page, programming, japanese, wakaru])
        .catch(catchers![not_found])
}

#[get("/")]
fn index(locals: State<lang::Locals>, cookies: Cookies) -> Template {
    let context = data::MinimalPageContext::new(data::CorePageContext::new(
        &locals.get(
            cookies
                .get("lang")
                .map(|c| c.value())
                .unwrap_or(config::DEFAULT_LANG),
        ),
        String::from("/"),
    ));
    Template::render("index", context)
}

#[get("/programming")]
fn programming(locals: State<lang::Locals>, cookies: Cookies) -> Template {
    template_category_index(
        &locals.get(
            cookies
                .get("lang")
                .map(|c| c.value())
                .unwrap_or(config::DEFAULT_LANG),
        ),
        String::from("/programming"),
        "programming",
        config::MARKDOWN_PROGRAMMING_PATH,
    )
}

#[get("/japanese")]
fn japanese(locals: State<lang::Locals>, cookies: Cookies) -> Template {
    template_category_index(
        &locals.get(
            cookies
                .get("lang")
                .map(|c| c.value())
                .unwrap_or(config::DEFAULT_LANG),
        ),
        String::from("/japanese"),
        "japanese",
        config::MARKDOWN_JAPANESE_PATH,
    )
}

#[get("/<page..>", rank = 20)]
fn page(page: PathBuf, locals: State<lang::Locals>, cookies: Cookies) -> Option<Template> {
    template_page(
        &locals,
        cookies,
        page.parent().unwrap().to_str(),
        "page",
        &format!("/{}", page.to_str().unwrap()),
    )
}

#[get("/japanese/wakaru/<page..>")]
fn wakaru(page: PathBuf, locals: State<lang::Locals>, cookies: Cookies) -> Option<Template> {
    template_page(
        &locals,
        cookies,
        Some("japanese"),
        "wakaru",
        &format!("/japanese/wakaru/{}", page.to_str().unwrap()),
    )
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    let cookies = req.cookies();
    let locals = req.guard::<State<lang::Locals>>().unwrap();
    let context = data::MinimalPageContext::new(data::CorePageContext::new(
        &locals.get(
            cookies
                .get("lang")
                .map(|c| c.value())
                .unwrap_or(config::DEFAULT_LANG),
        ),
        req.uri().path().to_string(),
    ));
    Template::render("404", context)
}

// add cache for sorted index.
fn template_category_index(
    local_map: &lang::LocalMap,
    uri: String,
    template_name: &'static str,
    folder: &str,
) -> Template {
    let context;

    {
        let mut guard = cacher.lock().unwrap();
        let pages_info = guard.get_or_insert(template_name.to_string(), move || {
            let mut pages_info = data::list_available_pages_from_dir(folder);
            pages_info.sort_unstable_by(|a, b| {
                if a.date.is_none() {
                    Ordering::Less
                } else if b.date.is_none() {
                    Ordering::Greater
                } else {
                    b.date.unwrap().cmp(&a.date.unwrap())
                }
            });
            pages_info
        });

        context = data::IndexPageContext::new(
            pages_info.clone(),
            data::CorePageContext::new(local_map, uri),
        );
    }

    Template::render(template_name, context)
}

fn template_page(
    locals: &lang::Locals,
    mut cookies: Cookies,
    category_name: Option<&str>,
    template_name: &'static str,
    page: &str,
) -> Option<Template> {
    match data::make_data_from_markdown(page) {
        Some(data) => {
            // this part about cookies could be greatly improved on
            // the library side.
            let has_lang_cookie = cookies.get("lang").is_some();
            let lang = if has_lang_cookie {
                cookies.get("lang").unwrap().value().to_string()
            } else {
                cookies.add(
                    Cookie::build("lang", data.info.lang.clone())
                        .path("/")
                        .finish(),
                );
                data.info.lang.clone()
            };

            let context = data::PageContext::new(
                data,
                category_name.unwrap_or("").to_string(),
                data::CorePageContext::new(&locals.get(&lang), page.to_string()),
            );
            Some(Template::render(template_name, context))
        }
        None => {
            println!("    => page {} was not found", page);
            None
        }
    }
}
