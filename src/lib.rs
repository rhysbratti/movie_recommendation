use std::fs;
use std::sync::Arc;

use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT}, Response
};
use serde::{Deserialize, Serialize};

// Messing with ideas around creating general 'Moods' to map to keywords and genres.
pub enum Mood {
    FamilyNight,
    Cinematic,
    Chill,
    Thrilling,
    Scary,
    FeelGood,
    Funny,
    Oddball,
    Adventurous,
    Goofy,
    Intense,
    Serious,
    Heartfelt,
    Dramatic,
}

#[derive(Debug, Deserialize)]
pub struct RecommendationCriteria {
    pub genres: Vec<Genre>,
    pub watch_providers: Vec<WatchProvider>,
    pub runtime: Runtime,
    pub decade: Decade,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TestRecommCritieria {
    pub genres: Option<Vec<Genre>>,
    pub watch_providers: Option<Vec<WatchProvider>>,
    pub runtime: Option<Runtime>,
    pub decade: Option<Decade>,
}

/*
    Runtime options
 */
#[derive(Debug, Deserialize, Serialize)]
pub enum Runtime {
    Quick,
    Average,
    MovieNight,
    MartinScorsese,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeResponse {
    pub runtime: Runtime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuntimeInfo {
    name: String,
    description: String,
}
impl Runtime {
    pub fn info(&self) -> RuntimeInfo {
        match self {
            Runtime::Quick => RuntimeInfo {
                name: String::from("Quick"),
                description: String::from("You're not looking for a commitment, but still want something awesome"),
            },
            Runtime::Average => RuntimeInfo {
                name: String::from("Average"),
                description: String::from("You've got some time, lets make it count"),
            },
            Runtime::MovieNight => RuntimeInfo {
                name: String::from("Movie Night"),
                description: String::from("Grab your popcorn, lets find a movie with that 'wow' factor"),
            },
            Runtime::MartinScorsese => RuntimeInfo {
                name: String::from("Martin Scorsese"),
                description: String::from("You refer to movies as 'films' and have a lot of time on your hands"),
            },
        }
    }

    pub fn runtime(&self) -> (i32, i32) {
        match self {
            Runtime::Quick => (60, 90),
            Runtime::Average => (90,120),
            Runtime::MovieNight => (120, 150),
            Runtime::MartinScorsese => (150, 500),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecadeInfo {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DecadeResponse {
    pub decade: String,
}

impl Decade {
    pub fn info(&self) -> DecadeInfo {
        match self {
            Decade::Classic => DecadeInfo {
                name: String::from("Classics")
            },
            Decade::Fifties => DecadeInfo {
                name: String::from("50s")
            },
            Decade::Sixties => DecadeInfo {
                name: String::from("60s")
            },
            Decade::Seventies => DecadeInfo {
                name: String::from("70s")
            },
            Decade::Eighties => DecadeInfo {
                name: String::from("80s")
            },
            Decade::Nineties => DecadeInfo {
                name: String::from("90s")
            },
            Decade::TwoThousands => DecadeInfo {
                name: String::from("2000s")
            },
            Decade::TwentyTens => DecadeInfo {
                name: String::from("2010s")
            },
            Decade::Recent => DecadeInfo {
                name: String::from("Recent")
            }, 
        }
    }

    pub fn from_string(decade_string: &str) -> Self{
        match decade_string {
            "Classics" => Decade::Classic,
            "50s" => Decade::Fifties,
            "60s" => Decade::Sixties,
            "70s" => Decade::Seventies,
            "80s" => Decade::Eighties,
            "90s" => Decade::Nineties,
            "2000s" => Decade::TwoThousands,
            "2010s" => Decade::TwentyTens,
            "Recent" => Decade::Recent,
            _ => Decade::Recent,
        }
    }
}

/*
    Decade enum for filtering by Decade
*/
#[derive(Debug, Deserialize, Serialize)]
pub enum Decade {
    Classic,
    Fifties,
    Sixties,
    Seventies,
    Eighties,
    Nineties,
    TwoThousands,
    TwentyTens,
    Recent,
}


impl Decade {
    // Map decade enum to a tuple date range. This is passed into the /discover endpoint to filter by release year
    pub fn year_range(&self) -> (String, String){
        match self{
            Decade::Classic => (String::from("1900"), String::from("1949")),
            Decade::Fifties => (String::from("1950"), String::from("1959")),
            Decade::Sixties => (String::from("1960"), String::from("1969")),
            Decade::Seventies => (String::from("1970"), String::from("1979")),
            Decade::Eighties => (String::from("1980"), String::from("1989")),
            Decade::Nineties => (String::from("1990"), String::from("1999")),
            Decade::TwoThousands => (String::from("2000"), String::from("2009")),
            Decade::TwentyTens => (String::from("2010"), String::from("2019")),
            Decade::Recent => (String::from("2020"), String::from("2024")),
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Movie {
    pub id: i64,
    pub overview: String,
    //popularity: f64,
    pub poster_path: Option<String>,
    pub release_date: String,
    pub title: String,
    //vote_average: f64,
    //vote_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchByTitleResponse {
    pub results: Vec<Movie>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct WatchProvider {
    pub logo_path: String,
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

#[derive(Debug, Deserialize, Clone, Serialize)]
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

pub struct AsyncRecommendation {
    pub movie: Movie,
    //pub providers: Vec<WatchProvider>,
    pub fut_prov: tokio::task::JoinHandle<GetWatchProvidersResponse>,
}

#[derive(Debug, Serialize)]
pub struct MovieRecommendation {
    pub movie: Movie,
    pub providers: Vec<WatchProvider>,
}

/* Struct for interacting with TMDB API */
#[derive(Clone)]
pub struct Tmdb {
    base_url: String,
    api_key: String,
}

/* Methods for TMDB API endpoints */
impl Tmdb {
    /* Constructor for building Tmdb object */
    pub fn new() -> Self {
        let api_key: String = fs::read_to_string("api.key").expect("Unable to read API Key!").trim().to_string();
        let base_url: String = String::from("https://api.themoviedb.org/3/");
        Self { api_key, base_url }
    }

    /* For building shared instance */
    pub fn shared_instance() -> Arc<Self> {
        Arc::new(Self::new())
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
        runtime: Runtime,
        decade: Decade,
    ) -> Result<GetRecommendationsResponse, Box<dyn std::error::Error>> {
        let genre_ids: String = genres.iter().map(|g| g.id.to_string()).collect::<Vec<_>>().join(",");

        let provider_ids: String = watch_providers.iter().map(|p| p.provider_id.to_string()).collect::<Vec<_>>().join("|");

        let start_date = decade.year_range().0;

        let end_date = decade.year_range().1;
        
        let url = 
            format!("{}/discover/movie?include_adult=false&include_video=false&language=en-US&page=1&primary_release_date.gte={}-01-01&primary_release_date.lte={}-12-31&with_runtime.gte={}&with_runtime.lte={}&sort_by=popularity.desc&watch_region=US&with_genres={}&with_watch_monetization_types=flatrate&with_watch_providers={}", 
            self.base_url, 
            start_date,
            end_date,
            runtime.runtime().0,
            runtime.runtime().1,
            genre_ids, 
            provider_ids);

        println!("{}", &url);

        let recommendation_response = self.make_tmdb_request(&url).await;

        let recommendations = recommendation_response
            .json::<GetRecommendationsResponse>()
            .await.
            expect("Error parsing JSON");

        Ok(recommendations)
    }
}
