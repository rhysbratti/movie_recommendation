use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Response,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Movie {
    id: i64,
    overview: String,
    //popularity: f64,
    //poster_path: Option<String>,
    release_date: String,
    title: String,
    //vote_average: f64,
    //vote_count: i64,
}

#[derive(Debug, Deserialize)]
struct SearchByTitleResponse {
    results: Vec<Movie>,
}

#[derive(Debug, Deserialize)]
struct WatchProvider {
    logo_path: String,
    //provider_id: i32,
    provider_name: String,
}

/* Represents a JSON object of a country/region - contains a list of movie providers broken down by type: */
/* flatrate - subscription based services like Netflix, HBO, etc. */
/* buy - services where movies can be bought like Vudu, Google Play Movies, etc */
/* rent - services where movies can be rented, like Vudu, Google Play Movies, etc */
#[derive(Debug, Deserialize)]
struct WatchProviderRegion {
    flatrate: Vec<WatchProvider>,
    //buy: Vec<WatchProvider>,
    //rent: Vec<WatchProvider>,
}

/* Represents a JSON object containing supported countries/regions */
#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct WatchProviderRegions {
    us: WatchProviderRegion,
}

#[derive(Debug, Deserialize)]
struct SearchForWatchProvidersResponse {
    results: WatchProviderRegions,
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

    /* Searches for movie by title - helpful for retrieving movie IDs */
    pub async fn search_by_title(
        &self,
        movie_title: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/search/movie?query={}", self.base_url, movie_title);

        let search_response = self.make_tmdb_request(&url).await;

        if !search_response.status().is_success() {
            println!(
                "Error: API request failed with status code {}",
                search_response.status()
            );
            return Ok(());
        }

        let movie_results = search_response.json::<SearchByTitleResponse>().await?;

        for (i, movie) in movie_results.results.iter().enumerate() {
            if i >= 10 {
                break;
            }
            println!("Title: {}", movie.title);
            println!("Id: {}", movie.id);
            println!("Release date: {}", movie.release_date);
            println!("Overview: {}", movie.overview);
            println!("-----------------")
        }
        Ok(())
    }

    /* Gets watch providers by movie ID */
    /* Watch providers are given by country, and by type: */
    /* For this application we are mostly interested in "flatrate" */
    pub async fn get_watch_providers_by_id(
        &self,
        movie_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/movie/{}/watch/providers", self.base_url, movie_id);

        let provider_response = self.make_tmdb_request(&url).await;

        if !provider_response.status().is_success() {
            println!(
                "Error: API request failed with status code {}",
                provider_response.status()
            );
            return Ok(());
        }

        // TODO: Improve error handling for things not available on streaming services
        let providers = provider_response
            .json::<SearchForWatchProvidersResponse>()
            .await
            .expect("Error Parsing JSON");

        for provider in providers.results.us.flatrate {
            println!("Name: {}", provider.provider_name);
            println!("Logo path: {}", provider.logo_path);
        }

        Ok(())
    }
}
