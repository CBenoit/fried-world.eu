#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate chrono;
extern crate hoedown;
extern crate rand;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate walkdir;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

mod cacher;
mod config;
mod data;
pub mod handlers;
pub mod lang;
pub mod paste;
pub mod static_serving;
