#![allow(dead_code, unused_variables)]
use futures::Future;
use movie_recommendation::*;
use std::io;
use std::sync::Arc;

const NUM_RESULTS: u8 = 5;

pub async fn get_movie_recommendations(
    tmdb: Arc<Tmdb>,
    genres: Vec<Genre>,
    watch_providers: Vec<WatchProvider>,
    runtime: Runtime,
    decade: Decade,
) -> Result<Vec<AsyncRecommendation>, Box<dyn std::error::Error>> {
    let recommendations = tmdb
        .get_recommendations(genres, watch_providers, runtime, decade)
        .await
        .expect("Error fetching recommendations");
    let mut index = 0;

    let mut movie_recommendations = vec![];

    for movie in recommendations.results {
        if index > 10 {
            break;
        }
        let temp_tmdb = Arc::clone(&tmdb);
        let handle = tokio::spawn(async move {
            let movie_id = movie.id.to_string();
            temp_tmdb
                .get_watch_providers_by_id(&movie_id)
                .await
                .expect("Unable to call tmdb")
        });
        movie_recommendations.push(AsyncRecommendation {
            movie,
            fut_prov: handle,
        });
        index += 1;
    }

    Ok(movie_recommendations)
}

pub async fn get_recommendations_from_criteria(
    tmdb: Arc<Tmdb>,
    criteria: RecommendationCriteria,
) -> Result<Vec<AsyncRecommendation>, Box<dyn std::error::Error>> {
    let recommendations = tmdb
        .get_recommendations(
            criteria.genres,
            criteria.watch_providers,
            criteria.runtime,
            criteria.decade,
        )
        .await
        .expect("Error fetching recommendations");
    let mut index = 0;

    let mut movie_recommendations = vec![];

    for movie in recommendations.results {
        if index > 10 {
            break;
        }
        let temp_tmdb = Arc::clone(&tmdb);
        let handle = tokio::spawn(async move {
            let movie_id = movie.id.to_string();
            temp_tmdb
                .get_watch_providers_by_id(&movie_id)
                .await
                .expect("Unable to call tmdb")
        });
        movie_recommendations.push(AsyncRecommendation {
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

pub async fn get_runtime() -> Result<Runtime, Box<dyn std::error::Error>> {
    println!("Choose a movie length: ");

    for runtime in Runtime::get_list() {
        println!("{} - {}", runtime.display_name(), runtime.description());
    }

    let mut cli_input = String::new();

    io::stdin()
        .read_line(&mut cli_input)
        .expect("Failed to read line");

    Ok(Runtime::from_string(cli_input.trim().to_string()))
}

pub async fn get_decades() -> Result<Decade, Box<dyn std::error::Error>> {
    println!("Choose a few decades: ");

    for decade in Decade::get_list() {
        println!("{}", decade.display_name());
    }

    let mut cli_input = String::new();

    io::stdin()
        .read_line(&mut cli_input)
        .expect("Failed to read line");

    /*

    //This code attempts to get a list of decades. This is tough with the TMDB, so for now will only support 1 decade choice

    let decade_string = cli_input.split(",");

    let mut chosen_decades = decade_string
        .into_iter()
        .map(|f| Decade::from_string(String::from(f)))
        .collect::<Vec<Decade>>();

    chosen_decades.sort_by(|a, b| {
        if a.sort_order() > b.sort_order() {
            Ordering::Less
        } else if a.sort_order() == b.sort_order() {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });

    */

    Ok(Decade::from_string(cli_input.trim().to_string()))
}

pub async fn get_movies_from_title(
    movie_title: String,
    tmdb: Arc<Tmdb>,
) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
    let search_result = tmdb
        .search_by_title(&movie_title)
        .await
        .expect("Something went wrong - unable to find movie_id");

    Ok(search_result.results)
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
