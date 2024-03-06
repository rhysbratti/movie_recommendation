#![allow(dead_code, unused_variables)]
use movie_recommendation::*;
use std::sync::Arc;
use workflow::WorkflowStep;

use crate::workflow::{ProviderStep, WorkflowSteps};

mod tmdb_helper;
mod workflow;
// TODO: Data structure for mapping general "Vibes" to genres and keywords

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmdb = Tmdb::shared_instance();

    /*

    let movie = get_movie_from_title(&tmdb).await.expect("Error ID");
    */
    //workflow(tmdb).await.expect("Error starting workflow");
    recommendation_flow(tmdb).await.expect("Uh oh");

    Ok(())
}

/*
async fn workflow(tmdb: Arc<Tmdb>) -> Result<(), Box<dyn std::error::Error>> {
    let provider_step = workflow::ProviderStep::preload(&tmdb);
    println!("Welcome! With this tool you can combine your Streaming Services and your moods to get movie recommendations!");
    println!("Powered by the TMDB API");
    let genre_step = workflow::GenreStep::preload(&tmdb);

    let filtered_providers = provider_step.await.execute();

    let filtered_genres = genre_step.await.execute();

    println!("Gathering movie recommendations...");

    let movie_recommendations =
        tmdb_helper::get_movie_recommendations(tmdb, filtered_genres, filtered_providers)
            .await
            .expect("Error getting recommendations");

    println!("Based on your input, here is a list of recommended movies!: ");

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
*/

async fn recommendation_flow(tmdb: Arc<Tmdb>) -> Result<(), Box<dyn std::error::Error>> {
    let providers = tmdb.get_providers_list();
    let genres = tmdb.get_genre_list();

    let filtered_providers = tmdb_helper::get_provider_input(providers).await;

    let runtime = tmdb_helper::get_runtime().await.expect("Oh oh");

    let decade = tmdb_helper::get_decades().await.expect("Uh oh");

    let filtered_genres = tmdb_helper::get_genre_input(genres).await;

    let movie_recommendations = tmdb_helper::get_movie_recommendations(
        tmdb,
        filtered_genres,
        filtered_providers,
        runtime,
        decade,
    )
    .await
    .expect("Error getting recommendations");

    println!("Getting movie recommendations...");

    for movie_rec in movie_recommendations {
        println!("|{}", movie_rec.movie.title);
        println!("Desc: {}", movie_rec.movie.overview);
        println!("Date: {}", movie_rec.movie.release_date);
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
