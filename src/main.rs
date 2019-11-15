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

const UNBRANDED_OWNER: &str = "Not a Branded Item";

impl Food {
    fn header(&self) -> String {
        if self.brand_owner == UNBRANDED_OWNER.to_owned() {
            return format!("\"{}\"", self.description);
        }
        format!("\"{}\" (by {})", self.description, self.brand_owner)
    }

    fn to_tweets(&self) -> Vec<String> {
        self.ingredients.iter().enumerate().fold(
            vec![self.header() + "\n\nIngredients: "],
            |mut acc, (idx, x)| {
                let last = acc.last_mut().unwrap();
                if last.len() + x.len() > 240 {
                    last.truncate(last.len() - 1);
                    acc.push(x.to_string() + ",")
                } else {
                    if last.ends_with(',') {
                        last.push_str(" ");
                    }
                    last.push_str(x);
                    if idx != self.ingredients.len() - 1 {
                        last.push_str(",");
                    }
                }
                acc
            },
        )
    }
}

fn title_case(s: String) -> String {
    titlecase::titlecase(&s)
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
                .split(',')
                .map(|s| s.trim().to_string())
                .map(title_case)
                .collect::<Vec<String>>(),
        })
    });

    println!("{}", food.unwrap().to_tweets().join("\n---\n"));

    Ok(())
}
