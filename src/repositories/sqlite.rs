use std::{
    convert::TryFrom,
    sync::{Mutex, MutexGuard},
};

use rusqlite::{params, params_from_iter, Connection, Error::SqliteFailure, OpenFlags};

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

use super::pokemon::{DeleteError, FetchAllError, FetchOneError, InsertError, Repository};

pub struct SqliteRepository {
    connection: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn try_new(path: &str) -> Result<Self, ()> {
        let connection = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        {
            Ok(connection) => connection,
            _ => return Err(()),
        };

        match connection.execute("pragma foreign_keys = 1", []) {
            Ok(_) => Ok(Self {
                connection: Mutex::new(connection),
            }),
            _ => Err(()),
        }
    }

    fn fetch_pokemon_rows(
        lock: &MutexGuard<'_, Connection>,
        number: Option<u16>,
    ) -> Result<Vec<(u16, String)>, ()> {
        let (query, params) = match number {
            Some(number) => (
                "select number, name from pokemons where number = ?",
                vec![number],
            ),
            _ => ("select number, name from pokemons", vec![]),
        };

        let mut stmt = match lock.prepare(query) {
            Ok(stmt) => stmt,
            _ => return Err(()),
        };

        let mut rows = match stmt.query(params_from_iter(params)) {
            Ok(rows) => rows,
            _ => return Err(()),
        };

        let mut pokemon_rows = vec![];

        while let Ok(Some(row)) = rows.next() {
            match (row.get::<usize, u16>(0), row.get::<usize, String>(1)) {
                (Ok(number), Ok(name)) => pokemon_rows.push((number, name)),
                _ => return Err(()),
            };
        }

        Ok(pokemon_rows)
    }

    fn fetch_type_rows(lock: &MutexGuard<'_, Connection>, number: u16) -> Result<Vec<String>, ()> {
        let mut stmt = match lock.prepare("select name from types where pokemon_number = ?") {
            Ok(stmt) => stmt,
            _ => return Err(()),
        };

        let mut rows = match stmt.query([number]) {
            Ok(rows) => rows,
            _ => return Err(()),
        };

        let mut type_rows = vec![];

        while let Ok(Some(row)) = rows.next() {
            match row.get::<usize, String>(0) {
                Ok(name) => type_rows.push(name),
                _ => return Err(()),
            };
        }

        Ok(type_rows)
    }
}

impl Repository for SqliteRepository {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError> {
        let mut lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(InsertError::Unknown),
        };

        let transaction = match lock.transaction() {
            Ok(transaction) => transaction,
            _ => return Err(InsertError::Unknown),
        };

        match transaction.execute(
            "insert into pokemons (number, name) values (?, ?)",
            params![u16::from(number.clone()), String::from(name.clone())],
        ) {
            Ok(_) => {}
            Err(SqliteFailure(_, Some(message))) => {
                if message == "UNIQUE constraint failed: pokemons.number" {
                    return Err(InsertError::Conflict);
                } else {
                    return Err(InsertError::Unknown);
                }
            }
            _ => return Err(InsertError::Unknown),
        };

        for _type in Vec::<String>::from(types.clone()) {
            if let Err(_) = transaction.execute(
                "insert into types (pokemon_number, name) values (?, ?)",
                params![u16::from(number.clone()), _type],
            ) {
                return Err(InsertError::Unknown);
            }
        }

        match transaction.commit() {
            Ok(_) => Ok(Pokemon::new(number, name, types)),
            _ => Err(InsertError::Unknown),
        }
    }

    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchAllError::Unknown),
        };

        let pokemon_rows = match Self::fetch_pokemon_rows(&lock, None) {
            Ok(pokemon_rows) => pokemon_rows,
            _ => return Err(FetchAllError::Unknown),
        };

        let mut pokemons = vec![];

        for pokemon_row in pokemon_rows {
            let type_rows = match Self::fetch_type_rows(&lock, pokemon_row.0) {
                Ok(type_rows) => type_rows,
                _ => return Err(FetchAllError::Unknown),
            };

            let pokemon = match (
                PokemonNumber::try_from(pokemon_row.0),
                PokemonName::try_from(pokemon_row.1),
                PokemonTypes::try_from(type_rows),
            ) {
                (Ok(number), Ok(name), Ok(types)) => Pokemon::new(number, name, types),
                _ => return Err(FetchAllError::Unknown),
            };

            pokemons.push(pokemon);
        }

        Ok(pokemons)
    }

    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(FetchOneError::Unknown),
        };

        let mut pokemon_rows =
            match Self::fetch_pokemon_rows(&lock, Some(u16::from(number.clone()))) {
                Ok(pokemon_rows) => pokemon_rows,
                _ => return Err(FetchOneError::Unknown),
            };

        if pokemon_rows.is_empty() {
            return Err(FetchOneError::NotFound);
        }

        let pokemon_row = pokemon_rows.remove(0);

        let type_rows = match Self::fetch_type_rows(&lock, pokemon_row.0) {
            Ok(type_rows) => type_rows,
            _ => return Err(FetchOneError::Unknown),
        };

        match (
            PokemonNumber::try_from(pokemon_row.0),
            PokemonName::try_from(pokemon_row.1),
            PokemonTypes::try_from(type_rows),
        ) {
            (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
            _ => Err(FetchOneError::Unknown),
        }
    }

    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
        let lock = match self.connection.lock() {
            Ok(lock) => lock,
            _ => return Err(DeleteError::Unknown),
        };

        match lock.execute(
            "delete from pokemons where number = ?",
            params![u16::from(number)],
        ) {
            Ok(0) => Err(DeleteError::NotFound),
            Ok(_) => Ok(()),
            _ => Err(DeleteError::Unknown),
        }
    }
}
