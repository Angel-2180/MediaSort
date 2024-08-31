const TMDB_API_KEY: &'static str = "8c0878d2f669e85ac223e0680290fbf4";


use anyhow::{Error, Ok};
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{result::{MediaResult, MediaType}, strings::{accuracy, GETYEAR}};

#[derive(Serialize, Deserialize, Debug)]
struct MovieDBResult {
    id: i32,
    title: Option<String>,
    original_name: Option<String>,
    name: Option<String>,
    backdrop_path: Option<String>,
    first_air_date: Option<String>,
    genre_ids: Option<Vec<i32>>,
    original_language: Option<String>,
    overview: Option<String>,
    origin_country: Option<Vec<String>>,
    poster_path: Option<String>,
    popularity: Option<f64>,
    vote_average: Option<f64>,
    vote_count: Option<i32>,
    adult: Option<bool>,
    original_title: Option<String>,
    release_date: Option<String>,
    video: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MovieDBSearch {
    page: i32,
    results: Vec<MovieDBResult>,
    total_results: i32,
    total_pages: i32,
    status_code: Option<i32>,
    status_message: Option<String>,
}

impl MovieDBResult {
  fn to_result(&self, query: &str) -> Result<Option<MediaResult>, Error> {
    let mut result = MediaResult::new(
      self.title.clone().unwrap_or_default().to_string(),
      self.release_date.clone().unwrap_or_default(),
      MediaType::Movie,
      0,
    );
    let mut movie_title: String = "".to_string();
    if self.original_title.is_some() && self.original_title.clone().unwrap() != "".to_string() {
      movie_title = self.original_title.clone().unwrap();
    } else if self.title.is_some() && self.title.clone().unwrap() != "".to_string() {
      movie_title = self.title.clone().unwrap();
    }

    if movie_title != "".to_string() && self.release_date.is_some() && self.release_date.clone().unwrap() != "".to_string() {
      result.media_type = MediaType::Movie;
      result.title = movie_title.clone();
      if let Some(release_date) = self.release_date.clone() {
        if let Some(capture) = GETYEAR.captures(&release_date) {
          if let Some(year_match) = capture.get(1) {
            result.year = year_match.as_str().to_string();
            result.accuracy = accuracy(query, &movie_title);

          } else {
            return Err(anyhow::Error::msg(format!("movieDB error: No movie year: {}", release_date)));
          }
        }
      }
      return Ok(Some(result))
    }
    else if  self.name.is_some() && self.name.clone().unwrap() != "".to_string() && self.first_air_date.is_some() && self.first_air_date.clone().unwrap() != "".to_string()  {
      result.media_type = MediaType::Series;
      result.title = self.name.clone().unwrap();
      if let Some(first_air_date) = self.first_air_date.clone() {
        if let Some(capture) = GETYEAR.captures(&first_air_date) {
          if let Some(year_match) = capture.get(1) {
            result.year = year_match.as_str().to_string();
            result.accuracy = accuracy(query, &self.name.clone().unwrap());
          } else {
            return Err(anyhow::Error::msg(format!("movieDB error: No series year: {}", first_air_date)));
          }
        }
      }
      return Ok(Some(result))
    }
    Ok(None)
  }
}


pub(crate) fn search_movie_db(
  query: &str,
  year: Option<&str>,
  media_type: MediaType,
  debug_mode: bool,
) -> Result<Vec<MediaResult>, Error> {
  let client = Client::new();
  let mut path = "/search".to_string();

  let year_key = match media_type {
      MediaType::Movie => {
          path += "/movie";
          "year"
      }
      MediaType::Series => {
          path += "/tv";
          "first_air_date_year"
      }
  };

  let mut params = HashMap::new();
  params.insert("query", query);
  params.insert("api_key", TMDB_API_KEY);
  params.insert("page", "1");

  if let Some(year) = year {
    if !year.is_empty() {
      params.insert(year_key, year);
    }
  }

  if debug_mode {
      println!("Searching MovieDB API for '{}'", query);
  }

  let response = movie_db_request(&client, &path, &params)?;
  let search_data: MovieDBSearch = response.json()?;

  let results: Vec<MediaResult> = search_data
      .results
      .into_iter()
      .filter_map(|result| result.to_result(query).ok())
      .flatten()
      .collect();

  Ok(results)
}

fn movie_db_request(
  client: &Client,
  path: &str,
  params: &HashMap<&str, &str>,
) -> Result<Response, Error> {
  let url = format!(
      "https://api.themoviedb.org/3{}?{}",
      path,
      serde_urlencoded::to_string(params).unwrap(),
  );
  let response = client.get(&url).send()?;
  if !response.status().is_success() {
      return Err(Error::msg(format!("Error: {}", response.status())));
  }

  Ok(response)
}