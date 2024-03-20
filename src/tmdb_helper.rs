#![allow(dead_code, unused_variables)]
use movie_recommendation::*;
use std::{collections::HashMap, sync::Arc};

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
            criteria.feedback,
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

struct AsyncFeedback {
    movie_id: i64,
    keyword_future: tokio::task::JoinHandle<KeywordResponse>,
}

async fn get_keyword_futures(tmdb: &Arc<Tmdb>, id_list: Vec<i64>) -> Vec<AsyncFeedback> {
    let mut futures: Vec<AsyncFeedback> = vec![];

    for id in id_list {
        let temp_tmdb = Arc::clone(&tmdb);
        let handle = tokio::spawn(async move {
            temp_tmdb
                .get_keywords_for_id(&id)
                .await
                .expect("Unable to call tmdb")
        });
        futures.push(AsyncFeedback {
            movie_id: id,
            keyword_future: handle,
        });
    }

    futures
}

async fn get_keyword_list(feedback: Vec<AsyncFeedback>) -> Vec<Keyword> {
    let mut keywords_list: Vec<Keyword> = vec![];
    for keyword_future in feedback {
        let keywords = keyword_future.keyword_future.await;
        match keywords {
            Ok(mut keyword_response) => keywords_list.append(&mut keyword_response.keywords),
            Err(err) => println!("{}", err),
        };
    }

    keywords_list
}

async fn get_keyword_votes(keywords: Vec<Keyword>) -> HashMap<i64, i16> {
    let mut votes: HashMap<i64, i16> = HashMap::new();
    for keyword in keywords {
        if votes.contains_key(&keyword.id) {
            votes.insert(keyword.id, votes.get(&keyword.id).unwrap() + 1);
        } else {
            votes.insert(keyword.id, 1);
        }
    }

    votes
}

async fn refine_keywords(
    mut upvotes: HashMap<i64, i16>,
    mut downvotes: HashMap<i64, i16>,
) -> (HashMap<i64, i16>, HashMap<i64, i16>) {
    let mut remove_upvotes: Vec<i64> = vec![];

    for (id, count) in &mut upvotes {
        if downvotes.contains_key(&id) {
            if downvotes.get(&id).unwrap() >= count {
                downvotes.remove(&id);
            } else {
                // Can't modify a collection we're iterating over
                remove_upvotes.push(*id);
            }
        }
    }

    for id in remove_upvotes {
        upvotes.remove(&id);
    }

    (upvotes, downvotes)
}

pub async fn process_feedback(
    tmdb: Arc<Tmdb>,
    thumbs_up_ids: Vec<i64>,
    thumbs_down_ids: Vec<i64>,
) -> (Vec<i64>, Vec<i64>) {
    let thumbs_up_future = get_keyword_futures(&tmdb, thumbs_up_ids);
    let thumbs_down_future = get_keyword_futures(&tmdb, thumbs_down_ids);

    let thumbs_up_keywords = get_keyword_list(thumbs_up_future.await);
    let thumbs_down_keywords = get_keyword_list(thumbs_down_future.await);

    let thumbs_up_votes = get_keyword_votes(thumbs_up_keywords.await).await;
    let thumbs_down_votes = get_keyword_votes(thumbs_down_keywords.await).await;

    let (refined_upvotes, refined_downvotes) =
        refine_keywords(thumbs_up_votes, thumbs_down_votes).await;

    let mut sorted_up_votes: Vec<_> = refined_upvotes.iter().collect();

    sorted_up_votes.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

    let mut sorted_down_votes: Vec<_> = refined_downvotes.iter().collect();

    sorted_down_votes.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

    let mut criteria_upvotes: Vec<i64> = vec![];
    let mut criteria_downvotes: Vec<i64> = vec![];

    println!("Upvotes: ");
    for (&id, &count) in sorted_up_votes.iter().take(5) {
        criteria_upvotes.push(id);
    }

    println!("Downvotes: ");
    for (&id, &count) in sorted_down_votes.iter().take(5) {
        criteria_downvotes.push(id)
    }

    (criteria_upvotes, criteria_downvotes)
}

#[cfg(test)]
mod tests {
    use futures::future;

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
            feedback: None,
        }
    }

    #[tokio::test]
    async fn test_keyword_votes() {
        let keywords = vec![
            Keyword {
                id: 111,
                name: "testing".to_string(),
            },
            Keyword {
                id: 111,
                name: "testing".to_string(),
            },
            Keyword {
                id: 111,
                name: "testing".to_string(),
            },
            Keyword {
                id: 222,
                name: "foo".to_string(),
            },
            Keyword {
                id: 222,
                name: "foo".to_string(),
            },
            Keyword {
                id: 333,
                name: "bar".to_string(),
            },
        ];
        let keyword_map: HashMap<i64, i16> = get_keyword_votes(keywords).await;

        assert_eq!(keyword_map.get(&111).unwrap(), &3i16);
        assert_eq!(keyword_map.get(&222).unwrap(), &2i16);
        assert_eq!(keyword_map.get(&333).unwrap(), &1i16);
    }

    #[tokio::test]
    async fn test_keyword_list() {
        let tmdb = Tmdb::shared_instance();

        let movie_ids = vec![82702, 62177];

        let future_response: Vec<AsyncFeedback> =
            get_keyword_futures(&tmdb, movie_ids.clone()).await;

        assert!(!future_response.is_empty());

        let keyword_response: Vec<Keyword> = get_keyword_list(future_response).await;

        assert!(!keyword_response.is_empty());
    }

    #[tokio::test]
    async fn test_keyword_futures() {
        let tmdb = Tmdb::shared_instance();

        let movie_ids = vec![82702, 62177];

        let future_response = get_keyword_futures(&tmdb, movie_ids.clone()).await;

        assert!(!future_response.is_empty());

        for future in future_response {
            let id = future.movie_id;
            assert!(&movie_ids.contains(&id));

            let keywords = future.keyword_future.await;

            assert!(keywords.is_ok());

            assert!(!keywords.unwrap().keywords.is_empty());
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
