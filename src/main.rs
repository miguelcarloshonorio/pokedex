mod api;
mod domain;
mod repositories;

#[macro_use]
extern crate rouille;
extern crate serde;

fn main() {
    api::serve("localhost:8000");
}
