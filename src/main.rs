extern crate egg_mode;
extern crate rusqlite;
extern crate titlecase;
extern crate tokio;

use rusqlite::Connection;
use rusqlite::NO_PARAMS;
use tokio::runtime::current_thread::block_on_all;

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
            vec![self.header() + &format!("\n\nIngredients ({}): ", self.ingredients.len())],
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

struct Config {
    pub token: egg_mode::Token,
}

impl Config {
    fn load() -> Self {
        let consumer_key = std::env::var("CONSUMER_KEY").unwrap().trim().to_string();
        let consumer_secret = std::env::var("CONSUMER_SECRET").unwrap().trim().to_string();
        let con_token = egg_mode::KeyPair::new(consumer_key, consumer_secret);

        let access_token = std::env::var("ACCESS_TOKEN").unwrap().trim().to_string();
        let access_secret = std::env::var("ACCESS_SECRET").unwrap().trim().to_string();
        let acc_token = egg_mode::KeyPair::new(access_token, access_secret);

        Config {
            token: egg_mode::Token::Access {
                consumer: con_token,
                access: acc_token,
            },
        }
    }
}

fn title_case(s: String) -> String {
    titlecase::titlecase(&s)
}

fn main() -> rusqlite::Result<()> {
    let conn = Connection::open(std::env::var("FOOD_DB").unwrap_or("food.db".to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT brand_owner, description, ingredients
        FROM branded_food
        JOIN food ON food.fdc_id = branded_food.fdc_id
        WHERE LENGTH(TRIM(ingredients)) > 0
        ORDER BY RANDOM()
        LIMIT 11;",
    )?;

    let food = stmt
        .query_row(NO_PARAMS, |row| {
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
        .unwrap();

    println!("{}", food.to_tweets().join("\n---\n"));
    let config = Config::load();

    let mut prev_tweet_id: Option<u64> = None;
    for tweet in food.to_tweets().iter() {
        let mut draft = egg_mode::tweet::DraftTweet::new(tweet);
        if let Some(tweet_id) = prev_tweet_id {
            draft = draft.in_reply_to(tweet_id);
        }
        let tweet = block_on_all(draft.send(&config.token)).unwrap();
        prev_tweet_id = Some(tweet.id);
    }

    Ok(())
}
