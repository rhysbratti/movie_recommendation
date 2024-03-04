#![allow(dead_code, unused_variables)]
use futures::Future;
use movie_recommendation::*;
use std::sync::Arc;
use std::{env, io};

const NUM_RESULTS: u8 = 5;

pub async fn get_movie_recommendations(
    tmdb: Arc<Tmdb>,
    genres: Vec<Genre>,
    watch_providers: Vec<WatchProvider>,
) -> Result<Vec<MovieRecommendation>, Box<dyn std::error::Error>> {
    let recommendations = tmdb
        .get_recommendations(genres, watch_providers)
        .await
        .expect("Error fetching recommendations");
    let mut index = 0;

    let mut movie_recommendations = vec![];

    for movie in recommendations.results {
        if index > 10 {
            break;
        }
        let temp_tmdb = tmdb.clone();
        let handle = tokio::spawn(async move {
            let movie_id = movie.id.to_string();
            temp_tmdb
                .get_watch_providers_by_id(&movie_id)
                .await
                .expect("Unable to call tmdb")
        });
        movie_recommendations.push(MovieRecommendation {
            movie,
            fut_prov: handle,
        });
        index += 1;
    }

    Ok(movie_recommendations)
}

pub async fn get_provider_input(
    providers: impl Future<Output = Result<GetProvidersResponse, Box<dyn std::error::Error>>>,
) -> Vec<WatchProvider> {
    let supported_providers = vec![
        "Netflix",
        "Hulu",
        "Apple TV",
        "Peacock",
        "Amazon Prime Video",
        "Max",
        "Disney Plus",
        "Tubi",
        "Crunchyroll",
        "Paramount Plus",
    ];
    println!("Providers:");

    for provider in &supported_providers {
        println!("|{}", provider);
    }

    println!("Comma separated providers: ");

    let mut prov_input = String::new();

    io::stdin()
        .read_line(&mut prov_input)
        .expect("Failed to read line");

    let provider_string = prov_input.split(",");

    let chosen_providers = provider_string.collect::<Vec<&str>>();

    providers
        .await
        .expect("Uh oh")
        .results
        .into_iter()
        .filter(|p| chosen_providers.contains(&p.provider_name.as_str()))
        .collect()
}

pub async fn get_genre_input(
    genres: impl Future<Output = Result<GetGenresResponse, Box<dyn std::error::Error>>>,
) -> Vec<Genre> {
    println!("Genres:");

    let genres = genres.await.expect("Oh no").genres;

    for genre in &genres {
        println!("|{}", genre.name);
    }

    let mut cli_input = String::new();

    io::stdin()
        .read_line(&mut cli_input)
        .expect("Failed to read line");

    let genre_string = cli_input.split(",");

    let chosen_genres = genre_string.collect::<Vec<&str>>();

    genres
        .into_iter()
        .filter(|g| chosen_genres.contains(&g.name.as_str()))
        .collect()
}

pub async fn get_movie_from_title(tmdb: &Tmdb) -> Result<Movie, Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let movie_title = if args.len() < 2 {
        String::from("Spider-Man Homecoming")
    } else {
        args[1].clone()
    };

    let mut search_result = tmdb
        .search_by_title(&movie_title)
        .await
        .expect("Something went wrong - unable to find movie_id");

    for (i, movie) in search_result.results.iter().enumerate() {
        if i >= NUM_RESULTS.into() {
            break;
        }
        println!("{}", i);
        println!("Title: {}", movie.title);
        println!("Id: {}", movie.id);
        println!("Release date: {}", movie.release_date);
        println!("Overview: {}", movie.overview);
        println!("-----------------")
    }

    let mut cli_input = String::new();

    println!("Choose number from results: ");

    io::stdin()
        .read_line(&mut cli_input)
        .expect("Failed to read line");

    let index: usize = cli_input.trim().parse().expect("Input not an integer");

    Ok(search_result.results.remove(index))
}

pub async fn get_providers_from_id(
    tmdb: &Tmdb,
    movie_id: &i64,
) -> Result<Vec<WatchProvider>, Box<dyn std::error::Error>> {
    let provider_results = tmdb
        .get_watch_providers_by_id(&movie_id.to_string())
        .await
        .expect("Something went wrong - unable to find providers");

    Ok(provider_results.results.us.flatrate)
}

pub async fn get_genres_from_id(
    tmdb: &Tmdb,
    movie_id: &i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let genre_results = tmdb
        .get_movie_details(&movie_id.to_string())
        .await
        .expect("Error getting movie details");

    for genre in genre_results {
        println!("|{}", genre.name);
    }

    Ok(())
}
