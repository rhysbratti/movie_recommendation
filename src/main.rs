use futures::executor::block_on;
use movie_recommendation::*;
use std::{collections::HashMap, env, fs, io, iter::Filter, thread};

// TODO: Data structure for mapping general "Vibes" to genres and keywords

const NUM_RESULTS: u8 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmdb = Tmdb::new();

    /*

    let movie = get_movie_from_title(&tmdb).await.expect("Error ID");

    println!("Can be found at the following providers: ");

    get_providers_from_id(&tmdb, &movie.id)
        .await
        .expect("Error getting providers");

    println!("Has the following genres: ");

    get_genres_from_id(&tmdb, &movie.id)
        .await
        .expect("Error getting genres");
    */
    recommendation_flow(&tmdb).await;

    Ok(())
}

async fn get_movie_from_title(tmdb: &Tmdb) -> Result<Movie, Box<dyn std::error::Error>> {
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

async fn get_providers_from_id(
    tmdb: &Tmdb,
    movie_id: &i64,
) -> Result<Vec<WatchProvider>, Box<dyn std::error::Error>> {
    let provider_results = tmdb
        .get_watch_providers_by_id(&movie_id.to_string())
        .await
        .expect("Something went wrong - unable to find providers");

    Ok(provider_results.results.us.flatrate)
}

async fn get_genres_from_id(tmdb: &Tmdb, movie_id: &i64) -> Result<(), Box<dyn std::error::Error>> {
    let genre_results = tmdb
        .get_movie_details(&movie_id.to_string())
        .await
        .expect("Error getting movie details");

    for genre in genre_results {
        println!("|{}", genre.name);
    }

    Ok(())
}

async fn recommendation_flow(tmdb: &Tmdb) -> Result<(), Box<dyn std::error::Error>> {
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

    let genres = tmdb
        .get_genre_list()
        .await
        .expect("Unable to request genre list")
        .genres;

    let providers = tmdb
        .get_providers_list()
        .await
        .expect("Unable to get providers list")
        .results;

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

    let filtered_providers: Vec<WatchProvider> = providers
        .into_iter()
        .filter(|p| chosen_providers.contains(&p.provider_name.as_str()))
        .collect();

    println!("Genres:");

    for genre in &genres {
        println!("|{}", genre.name);
    }

    let mut cli_input = String::new();

    io::stdin()
        .read_line(&mut cli_input)
        .expect("Failed to read line");

    let genre_string = cli_input.split(",");

    //let collected_genres: Vec<&str> = genre_string.collect();

    let chosen_genres = genre_string.collect::<Vec<&str>>();

    let filtered_genres: Vec<Genre> = genres
        .into_iter()
        .filter(|g| chosen_genres.contains(&g.name.as_str()))
        .collect();

    let movies = tmdb
        .get_recommendations(filtered_genres, filtered_providers)
        .await
        .expect("Error getting recommendations")
        .results;

    let mut movie_recommendations = vec![];
    let mut index = 0;
    for movie in movies {
        if index > 10 {
            break;
        }
        let handle = tokio::spawn(async move {
            let temp_tmdb = Tmdb::new();
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

    for movie_rec in movie_recommendations {
        println!("|{}", movie_rec.movie.title);
        println!("Desc: {}", movie_rec.movie.overview);
        println!("Providers: ");
        let providers: Vec<WatchProvider> = movie_rec
            .fut_prov
            .await
            .expect("Uh oh ")
            .results
            .us
            .flatrate;
        for provider in providers {
            println!("{}", provider.provider_name);
        }
        println!("-------------");
    }

    Ok(())
}
