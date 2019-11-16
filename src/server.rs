use ingredients_bot::*;

fn main() -> rusqlite::Result<()> {
    let server = simple_server::Server::new(|_, mut resp| {
        let config = TwitterClient::load();
        let food = get_food().unwrap();

        println!("{}", food.to_tweets().join("\n---\n"));

        config.post_food(&food);
        Ok(resp.body(food.to_tweets().join("\n---\n").as_bytes().to_vec())?)
    });

    let addr = "0.0.0.0";
    let port = &std::env::var("PORT").unwrap_or("9000".to_string());
    println!("Starting server at {}:{}", addr, port);
    server.listen(addr, port);
}
