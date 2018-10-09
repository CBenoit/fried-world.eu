use std::fs;
use std::fs::File;
use std::io;
use std::path::PathBuf;

use rocket;
use rocket::fairing::AdHoc;
use rocket::http::Cookies;
use rocket::request::Form;
use rocket::request::State;
use rocket::response::content;
use rocket::response::Redirect;
use rocket_contrib::Template;

use self::paste_id::PasteID;
use data;
use lang;

mod config;
mod paste_id;
#[cfg(test)]
mod tests;

// mount routes for an already ignited rocket.
pub fn mount_paste(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
        .mount(config::ROOT_ROUTE, routes![index, upload, view_raw, view])
        .attach(AdHoc::on_attach(|rocket| {
            let upload_dir = rocket
                .config()
                .get_str("paste_upload_dir")
                .unwrap_or("pastes/")
                .to_string();

            Ok(rocket.manage(UploadDir(upload_dir)))
        }))
}

#[derive(FromForm)]
pub struct Paste {
    pub content: String,
}

// managed state
struct UploadDir(String);

#[derive(Clone, Serialize)]
pub struct PastePageContext<'a> {
    pub paste: String,
    pub id: String,
    pub core: data::CorePageContext<'a>,
}

impl<'a> PastePageContext<'a> {
    pub fn new(paste: String, id: String, core: data::CorePageContext<'a>) -> PastePageContext<'a> {
        PastePageContext { paste, id, core }
    }
}

#[get("/")]
fn index(locals: State<lang::Locals>, cookies: Cookies) -> Template {
    let context = data::MinimalPageContext::new(data::CorePageContext::new(
        &locals.get(
            cookies
                .get("lang")
                .map(|c| c.value())
                .unwrap_or(::config::DEFAULT_LANG),
        ),
        format!("{}", config::ROOT_ROUTE),
    ));
    Template::render("paste/index", context)
}

#[post("/upload", data = "<paste_form>")]
fn upload(paste_form: Form<Paste>, upload_dir: State<UploadDir>) -> Redirect {
    let paste = paste_form.into_inner();
    let id = PasteID::new(config::ID_LENGTH);
    let path: PathBuf = [&upload_dir.0, &id.to_string()].iter().collect();

    fs::write(path.as_path(), paste.content).expect("Unable to write file");

    Redirect::to(format!("{}/view/{}", config::ROOT_ROUTE, id).as_str())
}

#[get("/view/<id>")]
fn view(
    id: PasteID,
    upload_dir: State<UploadDir>,
    locals: State<lang::Locals>,
    cookies: Cookies,
) -> io::Result<Template> {
    let filename = format!("{upload_dir}/{id}", upload_dir = upload_dir.0, id = id);

    let context = PastePageContext::new(
        fs::read_to_string(filename)?,
        id.to_string(),
        data::CorePageContext::new(
            &locals.get(
                cookies
                    .get("lang")
                    .map(|c| c.value())
                    .unwrap_or(::config::DEFAULT_LANG),
            ),
            format!("{}/view/{}", config::ROOT_ROUTE, id),
        ),
    );

    Ok(Template::render("paste/view", context))
}

#[get("/view/raw/<id>")]
fn view_raw(id: PasteID, upload_dir: State<UploadDir>) -> Option<content::Plain<File>> {
    let filename = format!("{upload_dir}/{id}", upload_dir = upload_dir.0, id = id);
    File::open(&filename).map(|f| content::Plain(f)).ok()
}
