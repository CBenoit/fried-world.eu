// This is for development purpose only.

use std::path::{Path, PathBuf};

use rocket::response::NamedFile;

#[cfg(feature = "serves_static")]
pub fn mount_static(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket.mount("/", routes![static_serving])
}

#[cfg(not(feature = "serves_static"))]
pub fn mount_static(rocket: rocket::Rocket) -> rocket::Rocket {
    rocket
}

#[get("/static/<path..>", rank = 10)]
fn static_serving(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/static")).join(path)).ok()
}
