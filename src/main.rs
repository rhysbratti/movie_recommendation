use movie_recommendation::*;
use std::{env, fs, io};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut movie_id = String::new();
    let args: Vec<String> = env::args().collect();

    let api_key: String = fs::read_to_string("api.key").expect("Unable to read API Key!");

    let tmdb = Tmdb::new(api_key, String::from("https://api.themoviedb.org/3/"));

    let movie_title = if args.len() < 2 {
        String::from("Spider-Man Homecoming")
    } else {
        args[1].clone()
    };

    let _search_result = tmdb.search_by_title(movie_title).await;

    println!("Choose movie ID: ");

    io::stdin()
        .read_line(&mut movie_id)
        .expect("Failed to read line");

    let _provider_result = tmdb.get_watch_providers_by_id(movie_id).await;

    Ok(())
}
