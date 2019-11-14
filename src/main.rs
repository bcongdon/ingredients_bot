extern crate rusqlite;
extern crate titlecase;

use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Food {
    brand_owner: String,
    description: String,
    ingredients: Vec<String>,
}

fn title_case(s: String) -> String {
    titlecase::titlecase(&s).to_owned()
}

fn main() -> Result<()> {
    let conn = Connection::open("food.db")?;

    let mut stmt = conn.prepare(
        "SELECT brand_owner, description, ingredients
        FROM branded_food
        JOIN food ON food.fdc_id = branded_food.fdc_id
        ORDER BY RANDOM()
        LIMIT 11;",
    )?;

    let food = stmt.query_row(NO_PARAMS, |row| {
        let ingredients_raw: String = row.get(2)?;
        Ok(Food {
            brand_owner: title_case(row.get(0)?),
            description: title_case(row.get(1)?),
            ingredients: ingredients_raw
                .split(",")
                .map(|s| s.trim().to_string())
                .map(title_case)
                .collect::<Vec<String>>(),
        })
    });

    println!("{:?}", food.unwrap());

    Ok(())
}
