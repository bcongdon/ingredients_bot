extern crate egg_mode;
extern crate indexmap;
extern crate phf;
extern crate rusqlite;
extern crate simple_server;
extern crate titlecase;
extern crate tokio;

use indexmap::IndexSet;
use phf::phf_map;
use rusqlite::Connection;
use rusqlite::NO_PARAMS;

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
        if self.brand_owner == UNBRANDED_OWNER || self.brand_owner.is_empty() {
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
            "{} {}\n\nIngredients ({}): ",
            self.header(),
            if !attributes_str.is_empty() {
                format!("\n\n{}", attributes_str)
            } else {
                "".to_string()
            },
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

    fn attributes(&self) -> IndexSet<Attribute> {
        self.ingredients
            .iter()
            .flat_map(|i| Attribute::for_ingredient(i.to_string()))
            .collect()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Attribute {
    Apple,
    Artificial,
    Avocado,
    Banana,
    Beef,
    Bone,
    Broccoli,
    Butter,
    Carrot,
    Cheese,
    Cherry,
    Chicken,
    Chocolate,
    Coconut,
    Corn,
    Cucumber,
    Dairy,
    Duck,
    Egg,
    Eggplant,
    Fish,
    Garlic,
    Grape,
    Honey,
    Lemon,
    Liquor,
    Lobster,
    Mango,
    Melon,
    Microbe,
    Mushroom,
    Onion,
    Orange,
    Palm,
    Peanut,
    Pear,
    Pepper,
    Pig,
    Pineapple,
    Potato,
    Pumpkin,
    Rice,
    Salt,
    Soy,
    Strawberry,
    Sugar,
    Sunflower,
    Tomato,
    Water,
    Wheat,
}

static KEYWORDS: phf::Map<&'static str, Attribute> = phf_map! {
    "apple" => Attribute::Apple,
    "artificial" => Attribute::Artificial,
    "aspartame" => Attribute::Artificial,
    "avocado" => Attribute::Avocado,
    "bacon" => Attribute::Pig,
    "bacteria" => Attribute::Microbe,
    "banana" => Attribute::Banana,
    "beef" => Attribute::Beef,
    "bone" => Attribute::Bone,
    "broccoli" => Attribute::Broccoli,
    "butter" => Attribute::Butter,
    "carrot" => Attribute::Carrot,
    "casein" => Attribute::Dairy,
    "cheese" => Attribute::Cheese,
    "cherry" => Attribute::Cherry,
    "chicken" => Attribute::Chicken,
    "chili" => Attribute::Pepper,
    "chilli" => Attribute::Pepper,
    "chocolate" => Attribute::Chocolate,
    "cocoa" => Attribute::Chocolate,
    "coconut" => Attribute::Coconut,
    "collagen" => Attribute::Bone,
    "corn" => Attribute::Corn,
    "cream" => Attribute::Dairy,
    "cucumber" => Attribute::Cucumber,
    "culture" => Attribute::Microbe,
    "duck" => Attribute::Duck,
    "egg" => Attribute::Egg,
    "eggplant" => Attribute::Eggplant,
    "fish" => Attribute::Fish,
    "garlic" => Attribute::Garlic,
    "gluten" => Attribute::Wheat,
    "grape" => Attribute::Grape,
    "halibut" => Attribute::Fish,
    "ham" => Attribute::Pig,
    "honey" => Attribute::Honey,
    "hydrogen" => Attribute::Artificial,
    "jalapeno" => Attribute::Pepper,
    "lactic" => Attribute::Dairy,
    "lactose" => Attribute::Dairy,
    "lard" => Attribute::Pig,
    "lemon" => Attribute::Lemon,
    "liquor" => Attribute::Liquor,
    "lobster" => Attribute::Lobster,
    "mango" => Attribute::Mango,
    "melon" => Attribute::Melon,
    "microbe" => Attribute::Microbe,
    "milk" => Attribute::Dairy,
    "modified" => Attribute::Artificial,
    "mushroom" => Attribute::Mushroom,
    "nitrate" => Attribute::Artificial,
    "onion" => Attribute::Onion,
    "orange" => Attribute::Orange,
    "palm" => Attribute::Palm,
    "paprika" => Attribute::Pepper,
    "peanut" => Attribute::Peanut,
    "pear" => Attribute::Pear,
    "phosphate" => Attribute::Artificial,
    "pig" => Attribute::Pig,
    "pineapple" => Attribute::Pineapple,
    "pollock" => Attribute::Fish,
    "porcini" => Attribute::Mushroom,
    "pork" => Attribute::Pig,
    "portobell" => Attribute::Mushroom,
    "potato" => Attribute::Potato,
    "preservative" => Attribute::Artificial,
    "pumpkin" => Attribute::Pumpkin,
    "red pepper" => Attribute::Pepper,
    "rice" => Attribute::Rice,
    "salmon" => Attribute::Fish,
    "salt" => Attribute::Salt,
    "shiitake" => Attribute::Mushroom,
    "sodium" => Attribute::Salt,
    "soy" => Attribute::Soy,
    "spirit" => Attribute::Liquor,
    "squash" => Attribute::Pumpkin,
    "steak" => Attribute::Beef,
    "strawberry" => Attribute::Strawberry,
    "sugar" => Attribute::Sugar,
    "sulfate" => Attribute::Artificial,
    "sulfite" => Attribute::Artificial,
    "sunflower" => Attribute::Sunflower,
    "sweetener" => Attribute::Sugar,
    "swine" => Attribute::Pig,
    "tomatillos" => Attribute::Tomato,
    "tomato" => Attribute::Tomato,
    "tuna" => Attribute::Fish,
    "water" => Attribute::Water,
    "wheat" => Attribute::Wheat,
    "whey" => Attribute::Dairy,
};

impl Attribute {
    pub fn for_ingredient(ingredient: String) -> IndexSet<Attribute> {
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
            Attribute::Apple => "🍎",
            Attribute::Artificial => "🧪",
            Attribute::Avocado => "🥑",
            Attribute::Banana => "🍌",
            Attribute::Beef => "🐄",
            Attribute::Bone => "🦴",
            Attribute::Broccoli => "🥦",
            Attribute::Butter => "🧈",
            Attribute::Carrot => "🥕",
            Attribute::Cheese => "🧀",
            Attribute::Cherry => "🍒",
            Attribute::Chicken => "🐓",
            Attribute::Chocolate => "🍫",
            Attribute::Coconut => "🥥",
            Attribute::Corn => "🌽",
            Attribute::Cucumber => "🥒",
            Attribute::Dairy => "🥛",
            Attribute::Duck => "🦆",
            Attribute::Egg => "🥚",
            Attribute::Eggplant => "🍆",
            Attribute::Fish => "🐟",
            Attribute::Garlic => "🧄",
            Attribute::Grape => "🍇",
            Attribute::Honey => "🍯",
            Attribute::Lemon => "🍋",
            Attribute::Liquor => "🍸",
            Attribute::Lobster => "🦞",
            Attribute::Mango => "🥭",
            Attribute::Melon => "🍈",
            Attribute::Microbe => "🦠",
            Attribute::Mushroom => "🍄",
            Attribute::Onion => "🧅",
            Attribute::Orange => "🍊",
            Attribute::Palm => "🌴",
            Attribute::Peanut => "🥜",
            Attribute::Pear => "🍐",
            Attribute::Pepper => "🌶",
            Attribute::Pig => "🐖",
            Attribute::Pineapple => "🍍",
            Attribute::Potato => "🥔",
            Attribute::Pumpkin => "🎃",
            Attribute::Rice => "🍚",
            Attribute::Salt => "🧂",
            Attribute::Soy => "🍢",
            Attribute::Strawberry => "🍓",
            Attribute::Sugar => "🍬",
            Attribute::Sunflower => "🌻",
            Attribute::Tomato => "🍅",
            Attribute::Water => "💧",
            Attribute::Wheat => "🍞",
        }
        .trim()
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
