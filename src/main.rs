use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Url,
};
use serde::Deserialize;
use std::env;

const API_KEY: &str = "Bearer API_KEY_HERE";

#[derive(Debug, Deserialize)]
struct Movie {
    id: i64,
    original_language: String,
    original_title: String,
    overview: String,
    popularity: f64,
    poster_path: Option<String>,
    release_date: String,
    title: String,
    video: bool,
    vote_average: f64,
    vote_count: i64,
}

#[derive(Debug, Deserialize)]
struct SearchByTitleResponse {
    results: Vec<Movie>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let movie_title = if args.len() < 2 {
        String::from("Superman")
    } else {
        args[1].clone()
    };

    let url = format!(
        "https://api.themoviedb.org/3/search/movie?query={}",
        movie_title
    );

    let client = reqwest::Client::new();

    let search_response = client
        .get(url)
        .header(AUTHORIZATION, API_KEY)
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "rust web-api demo")
        .send()
        .await?;

    if !search_response.status().is_success() {
        println!(
            "Error: API request failed with status code {}",
            search_response.status()
        );
        return Ok(());
    }

    let movie_results = search_response.json::<SearchByTitleResponse>().await?;

    for movie in movie_results.results {
        println!("Title: {}", movie.title);
        println!("Id: {}", movie.id);
        println!("Release date: {}", movie.release_date);
        println!("-----------------")
    }

    Ok(())
}
