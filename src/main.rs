use movie_recommendation::*;
use std::{env, fs, io};

const NUM_RESULTS: u8 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key: String = fs::read_to_string("api.key").expect("Unable to read API Key!");

    let tmdb = Tmdb::new(api_key, String::from("https://api.themoviedb.org/3/"));

    let movie_id = get_id_from_title(&tmdb).await.expect("Error ID");

    get_providers_from_id(&tmdb, &movie_id)
        .await
        .expect("Error getting providers");

    get_genres_from_id(&tmdb, &movie_id)
        .await
        .expect("Error getting genres");

    Ok(())
}

async fn get_id_from_title(tmdb: &Tmdb) -> Result<String, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let movie_title = if args.len() < 2 {
        String::from("Spider-Man Homecoming")
    } else {
        args[1].clone()
    };

    let search_result = tmdb
        .search_by_title(&movie_title)
        .await
        .expect("Something went wrong - unable to find movie_id");

    for (i, movie) in search_result.results.iter().enumerate() {
        if i >= NUM_RESULTS.into() {
            break;
        }
        println!("Title: {}", movie.title);
        println!("Id: {}", movie.id);
        println!("Release date: {}", movie.release_date);
        println!("Overview: {}", movie.overview);
        println!("-----------------")
    }

    let mut movie_id = String::new();

    println!("Choose movie ID: ");

    io::stdin()
        .read_line(&mut movie_id)
        .expect("Failed to read line");

    Ok(movie_id)
}

async fn get_providers_from_id(
    tmdb: &Tmdb,
    movie_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let provider_results = tmdb
        .get_watch_providers_by_id(&movie_id)
        .await
        .expect("Something went wrong - unable to find providers");

    for provider in provider_results.results.us.flatrate {
        println!("{}", provider.provider_name);
    }

    Ok(())
}

async fn get_genres_from_id(
    tmbd: &Tmdb,
    movie_id: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let genre_results = tmbd
        .get_movie_details(movie_id)
        .await
        .expect("Error getting movie details");

    for genre in genre_results {
        println!("{}", genre.name);
    }

    Ok(())
}
