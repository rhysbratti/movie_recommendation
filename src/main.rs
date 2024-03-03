use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Response,
};
use serde::Deserialize;
use std::{env, fs, io};

#[derive(Debug, Deserialize)]
struct Movie {
    id: i64,
    overview: String,
    //popularity: f64,
    //poster_path: Option<String>,
    release_date: String,
    title: String,
    //vote_average: f64,
    //vote_count: i64,
}

#[derive(Debug, Deserialize)]
struct SearchByTitleResponse {
    results: Vec<Movie>,
}

#[derive(Debug, Deserialize)]
struct WatchProvider {
    logo_path: String,
    //provider_id: i32,
    provider_name: String,
}

#[derive(Debug, Deserialize)]
struct WatchProviderRegion {
    flatrate: Vec<WatchProvider>,
    //buy: Vec<WatchProvider>,
    //rent: Vec<WatchProvider>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct WatchProviderRegions {
    us: WatchProviderRegion,
}

#[derive(Debug, Deserialize)]
struct SearchForWatchProvidersResponse {
    results: WatchProviderRegions,
}

async fn make_tmdb_request(url: &String) -> Response {
    let api_key: String = fs::read_to_string("api.key").expect("Unable to read API Key!");
    let client = reqwest::Client::new();
    client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "rust web-api demo")
        .send()
        .await
        .expect("Failed to make call")
}

async fn search_by_title(movie_title: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.themoviedb.org/3/search/movie?query={}",
        movie_title
    );

    let search_response = make_tmdb_request(&url).await;

    if !search_response.status().is_success() {
        println!(
            "Error: API request failed with status code {}",
            search_response.status()
        );
        return Ok(());
    }

    let movie_results = search_response.json::<SearchByTitleResponse>().await?;

    for (i, movie) in movie_results.results.iter().enumerate() {
        if i >= 10 {
            break;
        }
        println!("Title: {}", movie.title);
        println!("Id: {}", movie.id);
        println!("Release date: {}", movie.release_date);
        println!("Overview: {}", movie.overview);
        println!("-----------------")
    }
    Ok(())
}

async fn get_watch_providers_by_id(movie_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.themoviedb.org/3/movie/{}/watch/providers",
        movie_id
    );

    let provider_response = make_tmdb_request(&url).await;

    if !provider_response.status().is_success() {
        println!(
            "Error: API request failed with status code {}",
            provider_response.status()
        );
        return Ok(());
    }

    let providers = provider_response
        .json::<SearchForWatchProvidersResponse>()
        .await
        .expect("Error Parsing JSON");

    for provider in providers.results.us.flatrate {
        println!("Name: {}", provider.provider_name);
        println!("Logo path: {}", provider.logo_path);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut movie_id = String::new();
    let args: Vec<String> = env::args().collect();

    let movie_title = if args.len() < 2 {
        String::from("Superman")
    } else {
        args[1].clone()
    };

    let search_result = search_by_title(movie_title).await;

    println!("Choose movie ID: ");

    io::stdin()
        .read_line(&mut movie_id)
        .expect("Failed to read line");

    let provider_result = get_watch_providers_by_id(movie_id).await;

    /*
    let client = reqwest::Client::new();

    let provider_response = client
        .get(url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, "rust web-api demo")
        .send()
        .await?;*/

    Ok(())
}
