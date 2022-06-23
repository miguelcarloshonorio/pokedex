use repositories::pokemon::{InMemoryRepository, Repository, SqliteRepository};
use std::sync::Arc;

mod api;
mod domain;
mod repositories;

#[macro_use]
extern crate rouille;
extern crate serde;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

fn main() {
    let repo = Arc::new(InMemoryRepository::new());

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("cli").long("cli").help("Runs in CLI mode"))
        .arg(Arg::with_name("sqlite").long("sqlite").value_name("PATH"))
        .get_matches();

    match matches.occurrences_of("cli") {
        0 => api::serve("localhost:8000", repo),
        _ => unreachable!(),
    }
}

fn build_repo(sqlite_value: Option<&str>) -> Arc<dyn Repository> {
    if let Some(path) = sqlite_value {
        match SqliteRepository::try_new(path) {
            Ok(repo) => return Arc::new(repo),
            _ => panic!("Error while creating sqlite repo"),
        }
    }

    Arc::new(InMemoryRepository::new())
}
