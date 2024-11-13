use std::path::PathBuf;

use anyhow::{bail, Result};

use regex::Regex;
use ffprobe::ffprobe;

use crate::search::{self, strings::*};

#[derive(Clone)]
pub struct Episode {
    pub full_path: PathBuf,
    pub filename: String,
    pub filename_clean: String,
    pub extension: String,

    pub name: String,
    pub season: u32,
    pub episode: u32,
    pub is_movie: bool,
    pub year: Option<u32>,
}

impl Episode {
    pub fn new(full_path: &PathBuf) -> Self {
        let filename = full_path.file_name().unwrap().to_str().unwrap();
        let filename_clean = clean_filename(filename).unwrap_or_default();

        let mut ep = Episode {
            full_path: full_path.clone(),
            filename: filename.to_string(),
            filename_clean: filename_clean.clone(),
            extension: "unknown".to_string(),

            name: "unknown".to_string(),
            season: 0,
            episode: 0,
            is_movie: false,
            year: None,
        };

        ep.fetch_infos();

        ep
    }

    #[cfg(test)]
    pub fn new_test(filename: &str, is_movie: bool) -> Self {
        let filename_clean = clean_filename(filename).unwrap_or_default();

        let mut ep = Episode {
            full_path: filename.into(),
            filename: filename.to_string(),
            filename_clean: filename_clean.clone(),
            extension: "unknown".to_string(),

            name: "unknown".to_string(),
            season: 0,
            episode: 0,
            is_movie: false,
            year: None,
        };

        ep.name = extract_series_name(&ep.filename_clean).unwrap();
        ep.season = ep.extract_season();
        ep.episode = ep.extract_episode();
        ep.extension = ep.extract_extension();
        ep.is_movie = is_movie;

        ep
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    fn fetch_infos(&mut self) {
        self.name = extract_series_name(&self.filename_clean).unwrap();
        self.season = self.extract_season();
        self.episode = self.extract_episode();
        self.extension = self.extract_extension();
        self.is_movie = self.is_movie().unwrap();
        self.year = self.extract_year().is_none().then(|| 0);
    }





    fn extract_season(&self) -> u32 {
        // First attempt: check for season indicators using simple string operations
        let season_parts: Vec<&str> = self.filename_clean.split_whitespace().collect();

        for i in 0..season_parts.len() {
            // Check for the traditional 'S' format
            if season_parts[i].starts_with('S') && season_parts[i].len() > 1
                && season_parts[i][1..].chars().all(char::is_numeric) {
                // Parse and return the season number
                return season_parts[i][1..].parse::<u32>().unwrap_or(1);
            }

            // Check for patterns like "2nd Season", "3rd Season", etc.
            if let Some(digit) = season_parts[i].chars().next() {
                // Check if the first character is a digit
                if digit.is_digit(10) && season_parts[i].len() > 2 {
                    // Ensure it ends with a valid suffix followed by "Season"
                    if (season_parts[i].ends_with("st") ||
                        season_parts[i].ends_with("nd") ||
                        season_parts[i].ends_with("rd") ||
                        season_parts[i].ends_with("th")) &&
                        (i + 1 < season_parts.len() && season_parts[i + 1].eq_ignore_ascii_case("season")) {
                            // Parse and return the season number
                            if let Ok(season_num) = season_parts[i][..season_parts[i].len()-2].parse::<u32>() {
                                return season_num;
                            }
                    }
                }
            }
        }

        // Second attempt: use regex to find the season number
        let season_pattern = r"S(\d{1,2})(?:E\d{1,2})?"; // Match 'S' followed by digits, optional 'E' with digits
        let re = Regex::new(season_pattern).unwrap();

        if let Some(captures) = re.captures(&self.filename_clean) {
            if let Some(season) = captures.get(1) {
                // Parse and return the captured season number
                return season.as_str().parse::<u32>().unwrap_or(1);
            }
        }

        0 // Return 0 if no season number was found
    }



    fn extract_episode(&self) -> u32 {

        //use first string operation if possible to avoid regex
        let episode: Vec<&str> = self.filename_clean.split_whitespace().collect();
        for i in 0..episode.len() {
            if episode[i].starts_with('E') && episode[i].len() > 1 && episode[i].chars().skip(1).all(char::is_numeric) {
                return episode[i].chars().skip(1).collect::<String>().parse::<u32>().unwrap_or(1);
            }
        }


        let episode_pattern = r"(?:S\d{1,2}E(\d{1,2}))|(?:E(\d{1,2}))|(?:\b(\d{1,3})\b)";
        let re = Regex::new(episode_pattern).unwrap();
        if let Some(captures) = re.captures(&self.filename_clean) {
            if let Some(episode) = captures.get(1) {
                return episode.as_str().parse::<u32>().unwrap_or(1);
            } else if let Some(episode) = captures.get(2) {
                return episode.as_str().parse::<u32>().unwrap_or(1);
            } else if let Some(episode) = captures.get(3) {
                return episode.as_str().parse::<u32>().unwrap_or(1);
            }
        }

        0
    }

    fn extract_extension(&self) -> String {
        let extension = self
            .full_path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        extension
    }

    fn is_movie(&self) -> Result<bool> {
        // Check if the filename explicitly indicates a movie
        if self.filename.contains("Film") || self.filename.contains("Movie") {
            return Ok(true);
        }

        // Check for the absence of season and episode (which could indicate a movie)
        if self.season == 0 && self.episode == 0 {
            return Ok(true);
        }

        // If it's part of a season but has no episode info, treat it with suspicion
        if self.season > 0 && self.episode == 0 {
            return Ok(true);
        }

        // Regex for season-episode patterns like "S01E01"
        let series_pattern = Regex::new(r"S\d{1,2}E\d{1,2}").unwrap();

        // Try to determine based on file duration
        match ffprobe(&self.full_path) {
            Ok(metadata) => {
                if let Some(duration_str) = metadata.format.duration {
                    let duration = duration_str.parse::<f32>().unwrap_or(0.0);

                    // If duration exceeds 4200 seconds, check further
                    if duration > 4200.0 {
                        // Check if the filename has a series-like pattern
                        if series_pattern.is_match(&self.filename) {
                            // If it matches a season-episode pattern, it's likely a series
                            return Ok(false);
                        } else {
                            // If no series pattern is found and duration is long, treat it as a movie
                            return Ok(true);
                        }
                    }
                }
            }
            Err(e) => {
                bail!("Error while parsing file with ffprobe: {:?}", e);
            }
        }

        // Default case: if it didn't match other criteria, assume it's a series
        Ok(false)
    }


    fn extract_year(&self) -> Option<u32> {
        search::strings::YEAR.captures(&self.filename_clean).map(|year| year[0].parse::<u32>().unwrap_or(0))
    }

}
