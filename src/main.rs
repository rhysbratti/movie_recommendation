#![allow(dead_code, unused_variables)]
use movie_recommendation::*;
use std::sync::Arc;

mod tmdb_helper;

// TODO: Data structure for mapping general "Vibes" to genres and keywords

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmdb = Tmdb::shared_instance();

    /*

    let movie = get_movie_from_title(&tmdb).await.expect("Error ID");
    */
    recommendation_flow(tmdb).await.expect("Uh oh");

    Ok(())
}

async fn recommendation_flow(tmdb: Arc<Tmdb>) -> Result<(), Box<dyn std::error::Error>> {
    let filtered_providers = tmdb_helper::get_provider_input(tmdb.get_providers_list()).await;

    let filtered_genres = tmdb_helper::get_genre_input(tmdb.get_genre_list()).await;

    let movie_recommendations =
        tmdb_helper::get_movie_recommendations(tmdb, filtered_genres, filtered_providers)
            .await
            .expect("Error getting recommendations");

    println!("Getting movie recommendations...");

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
