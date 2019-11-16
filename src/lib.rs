extern crate egg_mode;
extern crate phf;
extern crate rusqlite;
extern crate simple_server;
extern crate titlecase;
extern crate tokio;

use phf::phf_map;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use std::collections::HashSet;

use tokio::runtime::current_thread::block_on_all;

#[derive(Debug)]
pub struct Food {
    brand_owner: String,
    description: String,
    ingredients: Vec<String>,
}

const UNBRANDED_OWNER: &str = "Not a Branded Item";

impl Food {
    fn header(&self) -> String {
        if self.brand_owner == UNBRANDED_OWNER {
            return format!("\"{}\"", self.description);
        }
        format!("\"{}\" (by {})", self.description, self.brand_owner)
    }

    pub fn to_tweets(&self) -> Vec<String> {
        let attributes_str: String = self
            .attributes()
            .iter()
            .map(|a| a.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let header = format!(
            "{} \n\n{}\n\nIngredients ({}): ",
            self.header(),
            attributes_str,
            self.ingredients.len()
        );
        self.ingredients
            .iter()
            .enumerate()
            .fold(vec![header], |mut acc, (idx, x)| {
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
            })
    }

    fn attributes(&self) -> HashSet<Attribute> {
        self.ingredients
            .iter()
            .flat_map(|i| Attribute::for_ingredient(i.to_string()))
            .collect()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Attribute {
    Dairy,
    Soy,
    Corn,
    Wheat,
    Potato,
    Tomato,
    Honey,
    Coconut,
    Sunflower,
    Peanut,
    Egg,
    Palm,
    Liquor,
    Chicken,
    Beef,
    Rice,
    Chocolate,
    Pumpkin,
    Artificial,
    Mushroom,
    Salt,
    Cheese,
}

static KEYWORDS: phf::Map<&'static str, Attribute> = phf_map! {
    "wheat" => Attribute::Wheat,
    "gluten" => Attribute::Wheat,
    "corn" => Attribute::Corn,
    "soy" => Attribute::Soy,
    "milk" => Attribute::Dairy,
    "casein" => Attribute::Dairy,
    "lactose" => Attribute::Dairy,
    "lactic" => Attribute::Dairy,
    "butter" => Attribute::Dairy,
    "cream" => Attribute::Dairy,
    "whey" => Attribute::Dairy,
    "cheese" => Attribute::Cheese,
    "potato" => Attribute::Potato,
    "tomato" => Attribute::Tomato,
    "tomatillos" => Attribute::Tomato,
    "honey" => Attribute::Honey,
    "coconut" => Attribute::Coconut,
    "sunflower" => Attribute::Sunflower,
    "peanut" => Attribute::Peanut,
    "egg" => Attribute::Egg,
    "palm" => Attribute::Palm,
    "liquor" => Attribute::Liquor,
    "spirit" => Attribute::Liquor,
    "chicken" => Attribute::Chicken,
    "beef" => Attribute::Beef,
    "steak" => Attribute::Beef,
    "rice" => Attribute::Rice,
    "chocolate" => Attribute::Chocolate,
    "cocoa" => Attribute::Chocolate,
    "pumpkin" => Attribute::Pumpkin,
    "squash" => Attribute::Pumpkin,
    "artificial" => Attribute::Artificial,
    "hydrogen" => Attribute::Artificial,
    "phosphate" => Attribute::Artificial,
    "sulfite" => Attribute::Artificial,
    "sulfate" => Attribute::Artificial,
    "nitrate" => Attribute::Artificial,
    "modified" => Attribute::Artificial,
    "preservative" => Attribute::Artificial,
    "mushroom" => Attribute::Mushroom,
    "porcini" => Attribute::Mushroom,
    "portobell" => Attribute::Mushroom,
    "shiitake" => Attribute::Mushroom,
    "salt" => Attribute::Salt,
    "sodium" => Attribute::Salt,
};

impl Attribute {
    pub fn for_ingredient(ingredient: String) -> HashSet<Attribute> {
        let ingredient = ingredient.to_ascii_lowercase();

        KEYWORDS
            .entries()
            .filter(|(&k, _)| ingredient.contains(k))
            .map(|(_, v)| *v)
            .collect()
    }
}

impl std::string::ToString for Attribute {
    fn to_string(&self) -> String {
        match self {
            Attribute::Dairy => "🥛",
            Attribute::Soy => "🍢",
            Attribute::Wheat => "🍞",
            Attribute::Corn => "🌽",
            Attribute::Potato => "🥔",
            Attribute::Tomato => "🍅",
            Attribute::Honey => "🍯",
            Attribute::Coconut => "🥥",
            Attribute::Sunflower => "🌻",
            Attribute::Peanut => "🥜",
            Attribute::Egg => "🥚",
            Attribute::Palm => "🌴",
            Attribute::Liquor => "🍸",
            Attribute::Chicken => "🐓",
            Attribute::Beef => "🐄",
            Attribute::Rice => "🍚",
            Attribute::Chocolate => "🍫",
            Attribute::Pumpkin => "🎃",
            Attribute::Artificial => "🧪",
            Attribute::Mushroom => "🍄",
            Attribute::Salt => "🧂",
            Attribute::Cheese => "🧀",
        }
        .to_string()
    }
}

pub struct TwitterClient {
    pub token: egg_mode::Token,
}

impl TwitterClient {
    pub fn load() -> Self {
        let consumer_key = std::env::var("CONSUMER_KEY").unwrap().trim().to_string();
        let consumer_secret = std::env::var("CONSUMER_SECRET").unwrap().trim().to_string();
        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let access_token = std::env::var("ACCESS_TOKEN").unwrap().trim().to_string();
        let access_secret = std::env::var("ACCESS_SECRET").unwrap().trim().to_string();
        let acc_token = egg_mode::KeyPair::new(access_token, access_secret);

        TwitterClient {
            token: egg_mode::Token::Access {
                consumer: con_token,
                access: acc_token,
            },
        }
    }

    pub fn post_food(&self, f: &Food) {
        let mut prev_tweet_id: Option<u64> = None;
        for tweet in f.to_tweets().iter() {
            let mut draft = egg_mode::tweet::DraftTweet::new(tweet);
            if let Some(tweet_id) = prev_tweet_id {
                draft = draft.in_reply_to(tweet_id);
            }
            let tweet = block_on_all(draft.send(&self.token)).unwrap();
            prev_tweet_id = Some(tweet.id);
        }
    }
}

fn title_case(s: String) -> String {
    titlecase::titlecase(&s)
}

pub fn get_food() -> rusqlite::Result<Food> {
    let conn = Connection::open(std::env::var("FOOD_DB").unwrap_or("food.db".to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT brand_owner, description, ingredients
        FROM branded_food
        JOIN food ON food.fdc_id = branded_food.fdc_id
        WHERE LENGTH(TRIM(ingredients)) > 0
        ORDER BY RANDOM()
        LIMIT 11;",
    )?;

    stmt.query_row(NO_PARAMS, |row| {
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
    })
}
