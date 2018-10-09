#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate fried_world;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;

use rocket_contrib::Template;

use fried_world::*;

fn rocket() -> rocket::Rocket {
    let mut rocket = rocket::ignite();

    // attach Template fairing
    rocket = rocket.attach(Template::fairing());

    // manage locals
    rocket = lang::mount_locals(rocket);

    // paste
    rocket = paste::mount_paste(rocket);

    // static serving
    rocket = static_serving::mount_static(rocket);

    // root
    handlers::mount_handlers(rocket)
}

fn main() {
    rocket().launch();
}
