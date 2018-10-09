use std::collections::HashMap;
use std::fs;
use std::path::Path;

use rocket::fairing::AdHoc;
use rocket::http::{Cookie, Cookies};
use rocket::response::Redirect;
use serde_json;

pub type LocalMap = HashMap<String, String>;

pub struct Locals {
    default: &'static str,
    locals: HashMap<String, LocalMap>,
}

impl Locals {
    pub fn new(default: &'static str) -> Locals {
        Locals {
            default,
            locals: HashMap::new(),
        }
    }

    /// panics if file not found or json parsing error,
    /// as such, late runtime calls should be avoided.
    pub fn load(&mut self, key: String, path: &Path) {
        let json = fs::read_to_string(path).unwrap();
        let local = serde_json::from_str(&json).unwrap();
        self.locals.insert(key, local);
    }

    pub fn get(&self, key: &str) -> &LocalMap {
        self.locals
            .get(key)
            .unwrap_or_else(|| self.locals.get(self.default).unwrap())
    }
}

pub fn mount_locals(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
        .mount("/", routes![set_lang, get_lang])
        .attach(AdHoc::on_attach(|rocket| {
            let mut locals = Locals::new("en");
            locals.load("fr".to_string(), Path::new("langs/fr.json"));
            locals.load("en".to_string(), Path::new("langs/en.json"));

            Ok(rocket.manage(locals))
        }))
}

#[derive(FromForm)]
pub struct RedirectPage {
    pub page: String,
}

#[get("/set-lang/<lang>?<redirect_page>")]
pub fn set_lang(
    lang: String,
    redirect_page: Option<RedirectPage>,
    mut cookies: Cookies,
) -> Redirect {
    cookies.add(Cookie::build("lang", lang).path("/").finish());
    match redirect_page {
        Some(rpage) => Redirect::to(&rpage.page),
        None => Redirect::to("/"),
    }
}

#[get("/get-lang")]
pub fn get_lang(cookies: Cookies) -> String {
    cookies.get("lang").unwrap().value().to_string()
}
