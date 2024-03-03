use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Response,
};
use serde::Deserialize;
use std::io;

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
    //provider_id: i32,
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
    id: i32,
    pub name: String,
}

/* Struct for interacting with TMDB API */
pub struct Tmdb {
    base_url: String,
    api_key: String,
}

/* Methods for TMDB API endpoints */
impl Tmdb {
    /* Constructor for building Tmdb object */
    pub fn new(api_key: String, base_url: String) -> Self {
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
            .expect("Error Parsing JSON");

        Ok(providers)
    }
}
