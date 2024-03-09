#![allow(dead_code, unused_variables)]
use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use movie_recommendation::*;
mod redis;
mod tmdb_helper;
// TODO: Data structure for mapping general "Vibes" to genres and keywords

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let tmdb = Tmdb::shared_instance();

    HttpServer::new(|| {
        let cors = Cors::permissive();

        App::new()
            .wrap(cors)
            .service(get_runtimes)
            .service(get_decades)
            .service(get_simple_watch_providers)
            .service(get_movies_by_title)
            .service(post_movies)
            .service(post_runtime)
            .service(get_recommendations)
            .service(get_genres)
            .service(start_session)
            .service(get_session)
            .service(post_providers)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[get("/runtimes")]
async fn get_runtimes() -> impl Responder {
    let runtimes = vec![
        Runtime::Quick.info(),
        Runtime::Average.info(),
        Runtime::MovieNight.info(),
        Runtime::MartinScorsese.info(),
    ];

    web::Json(runtimes)
}

#[get("/decades")]
async fn get_decades() -> impl Responder {
    let decades = vec![
        Decade::Classic.info(),
        Decade::Fifties.info(),
        Decade::Sixties.info(),
        Decade::Seventies.info(),
        Decade::Eighties.info(),
        Decade::Nineties.info(),
        Decade::TwoThousands.info(),
        Decade::TwentyTens.info(),
        Decade::Recent.info(),
    ];

    web::Json(decades)
}

#[get("/simplewatchproviders")]
async fn get_simple_watch_providers() -> impl Responder {
    let tmdb = Tmdb::shared_instance();
    let providers = tmdb.get_providers_list();
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

    let mut provider_output: Vec<WatchProvider> = providers
        .await
        .expect("Uh oh")
        .results
        .into_iter()
        .filter(|p| supported_providers.contains(&p.provider_name.as_str()))
        .collect();

    for provider in &mut provider_output {
        provider.logo_path = str::replace(provider.logo_path.as_str(), "jpg", "svg");
    }

    web::Json(provider_output)
}

#[get("/movies/{movie_title}")]
async fn get_movies_by_title(movie_title: web::Path<String>) -> impl Responder {
    println!("Got a request for {}", movie_title);
    let tmdb = Tmdb::shared_instance();
    let movies = tmdb_helper::get_movies_from_title(movie_title.into_inner(), tmdb)
        .await
        .expect("Uh oh ");

    HttpResponse::Ok().json(movies)
}

#[post("/movies")]
async fn post_movies(movies: web::Json<Vec<Movie>>) -> impl Responder {
    println!("I got: {:#?}", movies);

    HttpResponse::Ok().body("Got the movies")
}

#[post("/watch_providers/{session_id}")]
async fn post_providers(
    session_id: web::Path<String>,
    providers: web::Json<Vec<WatchProvider>>,
) -> impl Responder {
    let id = session_id.clone();
    let json_string =
        serde_json::to_string(&providers.into_inner()).expect("Unable to parse providers");

    redis::to_cache(
        &session_id.into_inner(),
        String::from("providers"),
        json_string,
    )
    .await;

    let response = format!("Posted providers for {}", id);

    println!("{}", &response);

    HttpResponse::Ok().body(response)
}

#[post("/runtime/{session_id}")]
async fn post_runtime(
    session_id: web::Path<String>,
    runtime: web::Json<RuntimeResponse>,
) -> impl Responder {
    let id = session_id.clone();
    println!("Received a runtime: {:#?}", runtime);

    let json_string =
        serde_json::to_string(&runtime.into_inner()).expect("Unable to parse runtime");

    redis::to_cache(
        &session_id.into_inner(),
        String::from("runtime"),
        json_string,
    )
    .await;

    let response = format!("Posted runtime for {}", &id);

    HttpResponse::Ok().body(response)
}

#[post("/recommendations")]
async fn get_recommendations(criteria: web::Json<RecommendationCriteria>) -> impl Responder {
    let tmdb = Tmdb::shared_instance();

    let recs = tmdb_helper::get_recommendations_from_criteria(tmdb, criteria.into_inner())
        .await
        .expect("Error getting movie results");

    let mut movie_recommendations: Vec<MovieRecommendation> = vec![];

    for rec in recs {
        let providers: Vec<WatchProvider> = rec.fut_prov.await.expect("Uh oh ").results.us.flatrate;
        movie_recommendations.push(MovieRecommendation {
            movie: rec.movie,
            providers: providers,
        })
    }

    HttpResponse::Ok().json(movie_recommendations)
}

#[get("/genres")]
async fn get_genres() -> impl Responder {
    let tmdb = Tmdb::shared_instance();

    HttpResponse::Ok().json(tmdb.get_genre_list().await.expect("Uh oh").genres)
}

#[get{"/get_session/{session_id}"}]
async fn get_session(session_id: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body(
        redis::get_session(session_id.into_inner())
            .await
            .to_string(),
    )
}

#[get{"/start_session"}]
async fn start_session() -> impl Responder {
    println!("Got request to start session");
    HttpResponse::Ok().body(redis::start_session().await)
}
