use movie_recommendation::RecommendationCriteria;
use redis::{Commands, Connection};
use uuid::Uuid;

const CONNECTION_STRING: &str = "redis://localhost:6379";

pub async fn criteria_from_cache(
    session_id: &String,
) -> Result<RecommendationCriteria, redis::RedisError> {
    match get_connection() {
        Ok(mut con) => {
            let redis_result: String = con.get(session_id).expect("Error reading from redis");

            let criteria: RecommendationCriteria =
                serde_json::from_str(&redis_result).expect("Error parsing result");

            Ok(criteria)
        }
        Err(err) => Err(err),
    }
}

pub async fn criteria_to_cache(
    session_id: &String,
    criteria: RecommendationCriteria,
) -> Result<bool, redis::RedisError> {
    match get_connection() {
        Ok(mut con) => {
            let json_string = serde_json::to_string(&criteria).expect("Unable to parse criteria");
            Ok(con
                .set(session_id, json_string)
                .expect("Error writing to redis cache"))
        }
        Err(err) => Err(err),
    }
}

pub async fn start_recommendation_session() -> Result<String, redis::RedisError> {
    match get_connection() {
        Ok(mut con) => {
            let session_id = Uuid::new_v4().to_string();

            let criteria = RecommendationCriteria {
                genres: None,
                watch_providers: None,
                runtime: None,
                decade: None,
            };

            let json_string = serde_json::to_string(&criteria).expect("Unable to parse criteria");

            let _: () = con.set(&session_id, json_string).unwrap();

            Ok(session_id)
        }
        Err(err) => Err(err),
    }
}

fn get_connection() -> Result<Connection, redis::RedisError> {
    let client = redis::Client::open(CONNECTION_STRING);
    match redis::Client::open(CONNECTION_STRING) {
        Ok(client) => client.get_connection(),
        Err(err) => Err(err),
    }
}

pub async fn end_session(session_id: String) {
    let mut con = get_connection().expect("Error connecting to redis");

    let _: () = con.del(session_id).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session() {}
}
