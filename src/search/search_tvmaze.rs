use super::result::*;
use super::strings::accuracy;
use super::strings::GETYEAR;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use anyhow::Error;

pub fn search_tvmaze(query: &str, year: Option<&str>, media_type: MediaType) -> Result<Vec<MediaResult>, Error>{
    let _ = year;
    let url = format!("http://api.tvmaze.com/search/shows?q={}", query);
    if cfg!(debug_assertions) {
        println!("Searching TVMaze for '{}'", query);
    }

    let client = Client::new();
    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Err(Error::msg(format!("Error: {}", response.status())));
        }
    let body = response.text()?;
    // println!("{}", body);

    let tv_maze_results: Vec<TvMazeResult> = serde_json::from_str(&body)?;
    let mut results = Vec::new();
    for tv_maze_result in tv_maze_results {
        if let Some(captures) = GETYEAR.captures(&tv_maze_result.show.premiered) {
            if let Some(year_match) = captures.get(1) {
                let accuracy = accuracy(query, &tv_maze_result.show.name);
                let media_type = match media_type {
                    MediaType::Series => MediaType::Series,
                    MediaType::Movie => MediaType::Movie,

                };
                results.push(
                    MediaResult::new(
                    tv_maze_result.show.name.clone(),
                    year_match.as_str().to_string(),
                    media_type,
                    false,
                    accuracy,
                ));
            }
        }
    }
    if results.is_empty() {
        println!("No results found for '{}'", query);
    }
  Ok(results)
}

#[derive(Deserialize, Serialize)]
pub struct Links {
    pub previousepisode: Previousepisode,
    pub self_: SelfLink,
}

#[derive(Deserialize, Serialize)]
pub struct Previousepisode {
    pub href: String,
}

#[derive(Deserialize, Serialize)]
pub struct SelfLink {
    pub href: String,
}

#[derive(Deserialize, Serialize)]
pub struct Externals {
    pub thetvdb: Option<i32>,
    pub tvrage: Option<i32>,
}

#[derive(Deserialize, Serialize)]
pub struct Image {
    pub medium: String,
    pub original: String,
}

#[derive(Deserialize, Serialize)]
pub struct Network {
    pub country: Country,
    pub id: Option<i32>,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct Country {
    pub code: String,
    pub name: String,
    pub timezone: String,
}

#[derive(Deserialize, Serialize)]
pub struct Rating {
    pub average: Option<f64>,
}


#[derive(Deserialize, Serialize)]
pub struct Schedule {
    pub days: Vec<serde_json::Value>,
    pub time: String,
}

#[derive(Deserialize, Serialize)]
pub struct Show {
    pub links: Option<Links>,
    pub externals: Externals,
    pub genres: Vec<String>,
    pub id: Option<i32>,
    pub image: Image,
    pub language: String,
    pub name: String,
    pub network: Network,
    pub premiered: String,
    pub rating: Rating,
    pub runtime: Option<i32>,
    pub schedule: Schedule,
    pub status: String,
    pub summary: String,
    pub r#type: String,
    pub updated: Option<i32>,
    pub url: String,
    pub web_channel: Option<serde_json::Value>,
    pub weight: Option<i32>,
}


#[derive(Deserialize ,Serialize)]
pub struct TvMazeResult {
    pub score: Option<f64>,
    pub show: Show,
}