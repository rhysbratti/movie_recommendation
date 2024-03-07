#![allow(dead_code, unused_variables)]
use actix_cors::Cors;
use actix_web::{
    get, post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use movie_recommendation::*;
use std::sync::Arc;

mod tmdb_helper;
mod workflow;
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
    })
    .bind("127.0.0.1:8000")?
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
    web::Json(supported_providers)
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

#[post("/runtime")]
async fn post_runtime(runtime: web::Json<Runtime>) -> impl Responder {
    println!("Received a runtime: {:#?}", runtime);

    println!(
        "Runtime info: {} - {}",
        runtime.runtime().0,
        runtime.runtime().1
    );

    HttpResponse::Ok().body("Got it")
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
