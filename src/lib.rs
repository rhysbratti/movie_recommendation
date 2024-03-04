use std::{fmt::format, fs, future::Future, process::Output};

use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Response,
};
use serde::Deserialize;
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize)]
pub struct Movie {
    pub id: i64,
    pub overview: String,
    //popularity: f64,
    //poster_path: Option<String>,
    pub release_date: String,
    pub title: String,
    //vote_average: f64,
    //vote_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchByTitleResponse {
    pub results: Vec<Movie>,
}

#[derive(Debug, Deserialize)]
pub struct WatchProvider {
    //logo_path: String,
    pub provider_id: i32,
    pub provider_name: String,
}

/* Represents a JSON object of a country/region - contains a list of movie providers broken down by type: */
/* flatrate - subscription based services like Netflix, HBO, etc. */
/* buy - services where movies can be bought like Vudu, Google Play Movies, etc */
/* rent - services where movies can be rented, like Vudu, Google Play Movies, etc */
#[derive(Debug, Deserialize)]
pub struct WatchProviderRegion {
    pub flatrate: Vec<WatchProvider>,
    //buy: Vec<WatchProvider>,
    //rent: Vec<WatchProvider>,
}

/* Represents a JSON object containing supported countries/regions */
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct WatchProviderRegions {
    pub us: WatchProviderRegion,
}

#[derive(Debug, Deserialize)]
pub struct GetWatchProvidersResponse {
    pub results: WatchProviderRegions,
}

#[derive(Debug, Deserialize)]
pub struct GetMoveDetailsResponse {
    genres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
pub struct Genre {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GetGenresResponse {
    pub genres: Vec<Genre>,
}

#[derive(Debug, Deserialize)]
pub struct GetProvidersResponse {
    pub results: Vec<WatchProvider>,
}

#[derive(Debug, Deserialize)]
pub struct GetRecommendationsResponse {
    pub results: Vec<Movie>,
}

pub struct MovieRecommendation {
    pub movie: Movie,
    //pub providers: Vec<WatchProvider>,
    pub fut_prov: tokio::task::JoinHandle<GetWatchProvidersResponse>,
}

/* Struct for interacting with TMDB API */
pub struct Tmdb {
    base_url: String,
    api_key: String,
}

/* Methods for TMDB API endpoints */
impl Tmdb {
    /* Constructor for building Tmdb object */
    pub fn new() -> Self {
        let api_key: String = fs::read_to_string("api.key").expect("Unable to read API Key!");
        let base_url: String = String::from("https://api.themoviedb.org/3/");
        Self { api_key, base_url }
    }

    /* Private function to make TMDB API call */
    async fn make_tmdb_request(&self, url: &String) -> Response {
        let client = reqwest::Client::new();
        client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {0}", self.api_key))
            .header(ACCEPT, "application/json")
            .header(USER_AGENT, "rust web-api demo")
            .send()
            .await
            .expect("Failed to make call")
    }

    pub async fn get_movie_details(
        &self,
        movie_id: &String,
    ) -> Result<Vec<Genre>, Box<dyn std::error::Error>> {
        // https://api.themoviedb.org/3/movie/49521?language=en-US

        let url = format!("{}/movie/{}?language=en-US", self.base_url, movie_id);

        let details_response = self.make_tmdb_request(&url).await;

        let genres = details_response.json::<GetMoveDetailsResponse>().await?;

        Ok(genres.genres)
    }

    /* Searches for movie by title - helpful for retrieving movie IDs */
    pub async fn search_by_title(
        &self,
        movie_title: &String,
    ) -> Result<SearchByTitleResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/search/movie?query={}", self.base_url, movie_title);

        let search_response = self.make_tmdb_request(&url).await;

        let movie_results = search_response.json::<SearchByTitleResponse>().await?;

        Ok(movie_results)
    }

    /* Gets watch providers by movie ID */
    /* Watch providers are given by country, and by type: */
    /* For this application we are mostly interested in "flatrate" */
    pub async fn get_watch_providers_by_id(
        &self,
        movie_id: &String,
    ) -> Result<GetWatchProvidersResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/movie/{}/watch/providers", self.base_url, movie_id);

        let provider_response = self.make_tmdb_request(&url).await;

        // TODO: Improve error handling for things not available on streaming services
        let providers = provider_response
            .json::<GetWatchProvidersResponse>()
            .await
            .expect(format!("Error parsing {}", movie_id).as_str());

        Ok(providers)
    }

    pub async fn get_genre_list(&self) -> Result<GetGenresResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/genre/movie/list?language=en", self.base_url);

        let genre_response = self.make_tmdb_request(&url).await;

        let genres = genre_response
            .json::<GetGenresResponse>()
            .await
            .expect("Error parsing JSON");

        Ok(genres)
    }

    pub async fn get_providers_list(
        &self,
    ) -> Result<GetProvidersResponse, Box<dyn std::error::Error>> {
        let url = format!(
            "{}/watch/providers/movie?language=en-US&watch_region=US",
            self.base_url
        );

        let providers_response = self.make_tmdb_request(&url).await;

        let providers = providers_response
            .json::<GetProvidersResponse>()
            .await
            .expect("Error parsing JSON");

        Ok(providers)
    }

    pub async fn get_recommendations(
        &self,
        genres: Vec<Genre>,
        watch_providers: Vec<WatchProvider>,
    ) -> Result<GetRecommendationsResponse, Box<dyn std::error::Error>> {
        let genre_ids: String = genres.iter().map(|g| g.id.to_string()).collect::<Vec<_>>().join(",");

        let provider_ids: String = watch_providers.iter().map(|p| p.provider_id.to_string()).collect::<Vec<_>>().join("|");
        
        let url = 
            format!("{}/discover/movie?include_adult=false&include_video=false&language=en-US&page=1&sort_by=popularity.desc&watch_region=US&with_genres={}&with_watch_monetization_types=flatrate&with_watch_providers={}", 
            self.base_url, 
            genre_ids, 
            provider_ids);

        let recommendation_response = self.make_tmdb_request(&url).await;

        let recommendations = recommendation_response
            .json::<GetRecommendationsResponse>()
            .await.
            expect("Error parsing JSON");

        Ok(recommendations)
    }
}
