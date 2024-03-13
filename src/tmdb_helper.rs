#![allow(dead_code, unused_variables)]
use movie_recommendation::*;
use std::sync::Arc;

use crate::redis;

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

pub async fn get_recommendations_for_session(
    tmdb: Arc<Tmdb>,
    session_id: String,
) -> Result<Vec<AsyncRecommendation>, Box<dyn std::error::Error>> {
    let criteria = redis::criteria_from_cache(&session_id)
        .await
        .expect("Uh oh");

    let recommendations = tmdb
        .get_recommendations(
            criteria.genres.expect("No genres for ID"),
            criteria.watch_providers.expect("No watch providers for ID"),
            criteria.runtime.expect("No runtime for ID"),
            criteria.decade.expect("No decade for ID"),
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
