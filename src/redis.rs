use movie_recommendation::TestRecommCritieria;
use redis::{Commands, Connection};
use uuid::Uuid;

const CONNECTION_STRING: &str = "redis://redis-server:6379";

pub async fn to_cache(session_id: &String, json_id: String, json_string: String) {
    let mut con = get_connection();
    let key = format!("{}-{}", session_id, json_id);

    let _: () = con.set(key, json_string).unwrap();
}

pub async fn from_cache(session_id: &String, json_id: String) -> redis::RedisResult<String> {
    let mut con = get_connection();
    let key = format!("{}-{}", session_id, json_id);

    let result: String = con.get(key).unwrap();

    Ok(result)
}

pub async fn criteria_from_cache(
    session_id: &String,
) -> Result<TestRecommCritieria, Box<dyn std::error::Error>> {
    let mut con = get_connection();

    let redis_result: String = con.get(session_id).expect("Error getting redis stuff");

    let criteria: TestRecommCritieria =
        serde_json::from_str(&redis_result).expect("Error parsing result");

    Ok(criteria)
}

pub async fn criteria_to_cache(session_id: &String, criteria: TestRecommCritieria) {
    let mut con = get_connection();

    let json_string = serde_json::to_string(&criteria).expect("Unable to parse criteria");

    let _: () = con.set(session_id, json_string).unwrap();
}

pub async fn start_session() -> String {
    let mut con = get_connection();

    let session_id = Uuid::new_v4().to_string();

    let session_key = format!("session:{}", session_id);

    let _: () = con.set(&session_id, true).unwrap();

    session_id
}

pub async fn start_recommendation_session() -> String {
    let mut con = get_connection();

    let session_id = Uuid::new_v4().to_string();

    let criteria = TestRecommCritieria {
        genres: None,
        watch_providers: None,
        runtime: None,
        decade: None,
    };

    let json_string = serde_json::to_string(&criteria).expect("Unable to parse criteria");

    let _: () = con.set(&session_id, json_string).unwrap();

    session_id
}

pub async fn get_session(session_id: String) -> bool {
    let mut con = get_connection();

    let output = con.get(session_id);

    match output {
        Ok(return_val) => {
            println!("{}", &return_val);
            return_val
        }
        Err(msg) => {
            println!("{}", msg);
            false
        }
    }
}

fn get_connection() -> Connection {
    let client = redis::Client::open(CONNECTION_STRING).expect("Uh oh ");
    client.get_connection().expect("uh oh ")
}
