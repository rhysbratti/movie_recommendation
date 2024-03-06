use std::io;

use movie_recommendation::*;

pub enum WorkflowSteps {
    ProviderStep,
    FilterStep,
    GenreStep,
}

pub struct ProviderStep {
    //next: &impl ,
    providers_list: Vec<WatchProvider>,
    supported_providers: Vec<String>,
}

pub struct GenreStep {
    genre_list: Vec<Genre>,
}

pub trait WorkflowStep<T> {
    async fn preload(tmdb: &Tmdb) -> Self
    where
        Self: Sized;

    fn execute(&self) -> Vec<T>;
}

impl WorkflowStep<Genre> for GenreStep {
    async fn preload(tmdb: &Tmdb) -> Self {
        let genre_list = tmdb.get_genre_list().await.expect("Oh no").genres;

        GenreStep { genre_list }
    }

    fn execute(&self) -> Vec<Genre> {
        println!("Genres:");

        for genre in &self.genre_list {
            println!("|{}", genre.name);
        }

        let mut cli_input = String::new();

        io::stdin()
            .read_line(&mut cli_input)
            .expect("Failed to read line");

        let genre_string = cli_input.split(",");

        let chosen_genres = genre_string.collect::<Vec<&str>>();

        <Vec<Genre> as Clone>::clone(&self.genre_list)
            .into_iter()
            .filter(|g| chosen_genres.contains(&g.name.as_str()))
            .collect()
    }
}

impl WorkflowStep<WatchProvider> for ProviderStep {
    async fn preload(tmdb: &Tmdb) -> Self {
        let supported_providers = vec![
            "Netflix".to_string(),
            "Hulu".to_string(),
            "Apple TV".to_string(),
            "Peacock".to_string(),
            "Amazon Prime Video".to_string(),
            "Max".to_string(),
            "Disney Plus".to_string(),
            "Tubi".to_string(),
            "Crunchyroll".to_string(),
            "Paramount Plus".to_string(),
        ];

        let future_providers = tmdb.get_providers_list().await.expect("Error").results;

        ProviderStep {
            supported_providers,
            providers_list: future_providers,
        }
    }

    fn execute(&self) -> Vec<WatchProvider> {
        println!("Providers:");

        for provider in &self.supported_providers {
            println!("|{}", provider);
        }

        println!("Comma separated providers: ");

        let mut prov_input = String::new();

        io::stdin()
            .read_line(&mut prov_input)
            .expect("Failed to read line");

        let provider_string = prov_input.split(",");

        let chosen_providers = provider_string.collect::<Vec<&str>>();

        <Vec<WatchProvider> as Clone>::clone(&self.providers_list)
            .into_iter()
            .filter(|p| chosen_providers.contains(&p.provider_name.as_str()))
            .collect()
    }
}
