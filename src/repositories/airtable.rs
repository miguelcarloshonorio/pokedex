use std::convert::TryFrom;

use serde::Deserialize;

use crate::domain::entities::{Pokemon, PokemonName, PokemonNumber, PokemonTypes};

use super::pokemon::{DeleteError, FetchAllError, FetchOneError, InsertError, Repository};

pub struct AirtableRepository {
    url: String,
    auth_header: String,
}

impl AirtableRepository {
    pub fn try_new(api_key: &str, workspace_id: &str) -> Result<Self, ()> {
        let url = format!("https://api.airtable.com/v0/{}/pokemons", workspace_id);
        let auth_header = format!("Bearer {}", api_key);

        if let Err(_) = ureq::get(&url).set("Authorization", &auth_header).call() {
            return Err(());
        }

        println!("airtable connected");

        Ok(Self { url, auth_header })
    }

    fn fetch_pokemon_rows(&self, number: Option<u16>) -> Result<AirtableJson, ()> {
        let url = match number {
            Some(number) => format!("{}?filterByFormula=number%3D{}", self.url, number),
            None => format!("{}?sort%5B0%5D%5Bfield%5D=number", self.url),
        };

        let res = match ureq::get(&url)
            .set("Authorization", &self.auth_header)
            .call()
        {
            Ok(res) => res,
            Err(e) => {
                println!("error on call airtable: {}", e);
                return Err(());
            }
        };

        match res.into_json::<AirtableJson>() {
            Ok(json) => Ok(json),
            Err(e) => {
                println!("error on convert to json: {}", e);
                return Err(());
            }
        }
    }
}

impl Repository for AirtableRepository {
    fn insert(
        &self,
        number: PokemonNumber,
        name: PokemonName,
        types: PokemonTypes,
    ) -> Result<Pokemon, InsertError> {
        let json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(InsertError::Unknown),
        };

        if !json.records.is_empty() {
            return Err(InsertError::Conflict);
        }

        let body = ureq::json!({
            "records": [{
                "fields": {
                    "number": u16::from(number.clone()),
                    "name": String::from(name.clone()),
                    "types": Vec::<String>::from(types.clone()),
                },
            }],
        });

        if let Err(_) = ureq::post(&self.url)
            .set("Authorization", &self.auth_header)
            .send_json(body)
        {
            return Err(InsertError::Unknown);
        }

        Ok(Pokemon::new(number, name, types))
    }

    fn fetch_all(&self) -> Result<Vec<Pokemon>, FetchAllError> {
        let json = match self.fetch_pokemon_rows(None) {
            Ok(json) => json,
            _ => return Err(FetchAllError::Unknown),
        };

        let mut pokemons = vec![];

        for record in json.records.into_iter() {
            match (
                PokemonNumber::try_from(record.fields.number),
                PokemonName::try_from(record.fields.name),
                PokemonTypes::try_from(record.fields.types),
            ) {
                (Ok(number), Ok(name), Ok(types)) => {
                    pokemons.push(Pokemon::new(number, name, types))
                }
                _ => return Err(FetchAllError::Unknown),
            }
        }

        Ok(pokemons)
    }

    fn fetch_one(&self, number: PokemonNumber) -> Result<Pokemon, FetchOneError> {
        let mut json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(FetchOneError::Unknown),
        };

        if json.records.is_empty() {
            return Err(FetchOneError::NotFound);
        }

        let record = json.records.remove(0);

        match (
            PokemonNumber::try_from(record.fields.number),
            PokemonName::try_from(record.fields.name),
            PokemonTypes::try_from(record.fields.types),
        ) {
            (Ok(number), Ok(name), Ok(types)) => Ok(Pokemon::new(number, name, types)),
            _ => Err(FetchOneError::Unknown),
        }
    }

    fn delete(&self, number: PokemonNumber) -> Result<(), DeleteError> {
        let mut json = match self.fetch_pokemon_rows(Some(u16::from(number.clone()))) {
            Ok(json) => json,
            _ => return Err(DeleteError::Unknown),
        };

        if json.records.is_empty() {
            return Err(DeleteError::NotFound);
        }

        let record = json.records.remove(0);

        match ureq::delete(&format!("{}/{}", self.url, record.id))
            .set("Authorization", &self.auth_header)
            .call()
        {
            Ok(_) => Ok(()),
            _ => Err(DeleteError::Unknown),
        }
    }
}

#[derive(Deserialize)]
struct AirtableJson {
    records: Vec<AirtableRecord>,
}

#[derive(Deserialize)]
struct AirtableRecord {
    id: String,
    fields: AirtableFields,
}

#[derive(Deserialize)]
struct AirtableFields {
    number: u16,
    name: String,
    types: Vec<String>,
}
