use movie_recommendation::*;
use std::{collections::HashMap, env, fs, io};

// TODO: Data structure for mapping general "Vibes" to genres and keywords

const NUM_RESULTS: u8 = 5;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key: String = fs::read_to_string("api.key").expect("Unable to read API Key!");

    let tmdb = Tmdb::new(api_key, String::from("https://api.themoviedb.org/3/"));

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
) -> Result<(), Box<dyn std::error::Error>> {
    let provider_results = tmdb
        .get_watch_providers_by_id(&movie_id.to_string())
        .await
        .expect("Something went wrong - unable to find providers");

    for provider in provider_results.results.us.flatrate {
        println!("|{}", provider.provider_name);
    }

    Ok(())
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
    let genres = tmdb
        .get_genre_list()
        .await
        .expect("Unable to request genre list")
        .genres;

    println!("Provide comma separated list of genres");

    for genre in genres {
        println!("|{}", genre.name);
    }

    let mut cli_input = String::new();

    io::stdin()
        .read_line(&mut cli_input)
        .expect("Failed to read line");

    let genre_string = cli_input.split(",");

    //let collected_genres: Vec<&str> = genre_string.collect();

    let chosen_genres = genre_string.map(|genre| genre.trim().parse::<String>().expect("Uh oh "));
    //let chosen_genres = genre_string.collect::<Vec<&str>>();

    for genre in chosen_genres {
        println!("{}", genre);
    }

    //println!("{:#?}", chosen_genres);

    Ok(())
}
