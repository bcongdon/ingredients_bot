extern crate ingredients_bot;
use ingredients_bot::*;

fn main() {
    let food = get_food().unwrap();
    println!("{}", food.to_tweets().join("\n---\n"));
}
