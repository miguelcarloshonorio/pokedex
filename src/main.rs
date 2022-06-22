use std::sync::Arc;
use repositories::pokemon::InMemoryRepository;

mod api;
mod domain;
mod repositories;

#[macro_use]
extern crate rouille;
extern crate serde;

fn main() {
    let repo = Arc::new(InMemoryRepository::new());
    api::serve("localhost:8000", repo);
}
