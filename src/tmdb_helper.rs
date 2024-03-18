#![allow(dead_code, unused_variables)]
use movie_recommendation::*;
use std::sync::Arc;

use crate::redis_helper;

const NUM_RESULTS: u8 = 5;

pub async fn get_recommendations_for_session(
    tmdb: Arc<Tmdb>,
    session_id: String,
) -> Result<Vec<AsyncRecommendation>, Box<dyn std::error::Error>> {
    let criteria = redis_helper::criteria_from_cache(&session_id).await?;

    let recommendations = tmdb
        .get_recommendations(
            criteria.genres.expect("No genres for ID"),
            criteria.watch_providers.expect("No watch providers for ID"),
            criteria.runtime.expect("No runtime for ID"),
            criteria.decade.expect("No decade for ID"),
        )
        .await?;
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
            async_providers: handle,
        });
        index += 1;
    }

    Ok(movie_recommendations)
}

pub async fn get_movies_from_title(
    movie_title: String,
    tmdb: Arc<Tmdb>,
) -> Result<Vec<Movie>, Box<dyn std::error::Error>> {
    let search_result = tmdb.search_by_title(&movie_title).await?;

    Ok(search_result.results)
}

pub async fn get_providers_from_id(
    tmdb: &Tmdb,
    movie_id: i64,
) -> Result<Vec<WatchProvider>, Box<dyn std::error::Error>> {
    let provider_results = tmdb
        .get_watch_providers_by_id(&movie_id.to_string())
        .await?;

    Ok(provider_results.results.us.flatrate)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_criteria() -> RecommendationCriteria {
        RecommendationCriteria {
            genres: Some(vec![
                Genre {
                    id: 28,
                    name: "Action".to_string(),
                },
                Genre {
                    id: 12,
                    name: "Adventure".to_string(),
                },
            ]),
            watch_providers: Some(vec![WatchProvider {
                logo_path: "/".to_string(),
                provider_id: 8,
                provider_name: "Netflix".to_string(),
            }]),
            runtime: Some(Runtime::from_string("Average")),
            decade: Some(Decade::from_string("Recent")),
        }
    }

    #[tokio::test]
    async fn test_recommendations() {
        let session_id = String::from("123-456-789");
        let tmdb = Tmdb::shared_instance();

        let criteria = get_criteria();

        redis_helper::criteria_to_cache(&session_id, criteria)
            .await
            .expect("Error interacting with redis");

        let recommendations = get_recommendations_for_session(tmdb, session_id.clone()).await;

        assert!(recommendations.is_ok());
        let recommendations = recommendations.unwrap();

        assert!(!recommendations.is_empty());

        redis_helper::end_session(session_id).await;
    }

    #[tokio::test]
    #[should_panic(expected = "No genres for ID")]
    async fn test_recommendations_no_genre() {
        let session_id = String::from("987-654-321");
        let tmdb = Tmdb::shared_instance();

        let mut criteria = get_criteria();

        criteria.genres = None;

        redis_helper::criteria_to_cache(&session_id, criteria)
            .await
            .expect("Error interacting with redis");

        let recommendations = get_recommendations_for_session(tmdb, session_id.clone()).await;

        redis_helper::end_session(session_id).await;
    }

    #[tokio::test]
    #[should_panic(expected = "No watch providers for ID")]
    async fn test_recommendations_no_providers() {
        let session_id = String::from("555-555-555");
        let tmdb = Tmdb::shared_instance();

        let mut criteria = get_criteria();

        criteria.watch_providers = None;

        redis_helper::criteria_to_cache(&session_id, criteria)
            .await
            .expect("Error interacting with redis");

        let recommendations = get_recommendations_for_session(tmdb, session_id.clone()).await;
        redis_helper::end_session(session_id).await;
    }

    #[tokio::test]
    #[should_panic(expected = "No runtime for ID")]
    async fn test_recommendations_no_runtime() {
        let session_id = String::from("545-789-123");
        let tmdb = Tmdb::shared_instance();

        let mut criteria = get_criteria();

        criteria.runtime = None;

        redis_helper::criteria_to_cache(&session_id, criteria)
            .await
            .expect("Error interacting with redis");

        let recommendations = get_recommendations_for_session(tmdb, session_id.clone()).await;
        redis_helper::end_session(session_id).await;
    }

    #[tokio::test]
    #[should_panic(expected = "No decade for ID")]
    async fn test_recommendations_no_decade() {
        let session_id = String::from("222-222-222");
        let tmdb = Tmdb::shared_instance();

        let mut criteria = get_criteria();

        criteria.decade = None;

        redis_helper::criteria_to_cache(&session_id, criteria)
            .await
            .expect("Error interacting with redis");

        let recommendations = get_recommendations_for_session(tmdb, session_id.clone()).await;
        redis_helper::end_session(session_id).await;
    }

    #[tokio::test]
    async fn test_providers() {
        let tmdb = Tmdb::shared_instance();
        let movie_id: i64 = 438631;

        let providers = get_providers_from_id(&tmdb, movie_id).await;

        assert!(providers.is_ok());
        let providers = providers.unwrap();

        assert!(!providers.is_empty());
    }
}
