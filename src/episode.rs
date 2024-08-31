use std::path::PathBuf;
use std::vec;

use anyhow::{bail, Result};

use regex::Regex;
use ffprobe::ffprobe;

use crate::search;

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
        let filename_clean = Self::clean_filename(filename);

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
        let filename_clean = Self::clean_filename(filename);

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

        ep.name = ep.extract_series_name().unwrap();
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
        self.name = self.extract_series_name().unwrap();
        self.season = self.extract_season();
        self.episode = self.extract_episode();
        self.extension = self.extract_extension();
        self.is_movie = self.is_movie().unwrap();
    }

    fn clean_filename(filename_to_clean: &str) -> String {
        let mut cleaned = filename_to_clean.to_string();
        cleaned = cleaned[..cleaned.len() - 4].to_owned();
        cleaned = cleaned.replace(&['.', '_', '-', '+'][..], " ");

        //remove unwanted patterns as [] and () content
        cleaned = Regex::new(r"\[.*?\]").unwrap().replace_all(&cleaned, "").to_string();
        cleaned = Regex::new(r"\(.*?\)").unwrap().replace_all(&cleaned, "").to_string();
        cleaned = Regex::new(r"\b(net|fit|ws|tv|TV|ec|co|vip|cc|red|NanDesuKa|FANSUB|tokyo|WEBRip|DL|H264|Light|com|org|info|www|com|vostfree|VOSTFR|boats|uno|Wawacity|wawacity|WEB|TsundereRaws|1080p|720p|x264|AAC|Tsundere|Raws|fit|ws|tv|TV|ec)\b").unwrap().replace_all(&cleaned, "").to_string();
        cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");

        cleaned = cleaned.trim().to_string();

        cleaned
    }

    fn extract_series_name(&self) -> Result<String> {

        //use first string operation if possible to avoid regex
        let name: Vec<&str> = self.filename_clean.split_whitespace().collect();

        for i in 0..name.len() {
            if name[i].starts_with('S') && name[i].len() > 1 && name[i].chars().skip(1).all(char::is_numeric) {
                return Ok(name[..i].join(" ").trim().to_string());
            } else if name[i].starts_with('E') && name[i].len() > 1 && name[i].chars().skip(1).all(char::is_numeric) {
                return Ok(name[..i].join(" ").trim().to_string());
            }
        }

        let name_patterns = vec![
            r"(.+?)(S\d{1,2}E\d{1,2}|S\d{1,2})",
            r"(.+?)(S\d{1,2} \d{1,2})",
            r"(.+?)(E\d{1,2})",
            r"(.+?)(\d{1,3})",
            r"(.+?)(Film|Movie)",
            r"(.+)",
        ];

        for pattern in name_patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(&self.filename_clean) {
                if let Some(name) = captures.get(1) {
                    return Ok(name.as_str().trim().to_string());
                }
            }
        }

        bail!("Name not found")
    }

    fn extract_season(&self) -> u32 {

        //use first string operation if possible to avoid regex
        let season: Vec<&str> = self.filename_clean.split_whitespace().collect();
        for i in 0..season.len() {
            if season[i].starts_with('S') && season[i].len() > 1 && season[i].chars().skip(1).all(char::is_numeric) {
                return season[i].chars().skip(1).collect::<String>().parse::<u32>().unwrap_or(1);
            }
        }


        let season_pattern = r"S(\d{1,2})(?:E\d{1,2})?";
        let re = Regex::new(season_pattern).unwrap();
        if let Some(captures) = re.captures(&self.filename_clean) {
            if let Some(season) = captures.get(1) {
                return season.as_str().parse::<u32>().unwrap_or(1);
            }
        }

        0
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
        if self.filename.contains("Film") || self.filename.contains("Movie") {
            return Ok(true);
        }
        if self.season == 0 && self.episode == 0 {
            return Ok(true);
        }

        match ffprobe(&self.full_path) {
            Ok(metadata) => {
                if let Some(duration) = metadata.format.duration {
                    if duration.parse::<f32>().unwrap_or(0.0) > 4200.0 {
                        return Ok(true);
                    }
                }
            }
            Err(e) => {
                bail!("Error while parsing file with ffprobe: {:?}", e);
            }
        }

        Ok(false)
    }

    fn extract_year(&self) -> Option<u32> {
        search::strings::YEAR.captures(&self.filename_clean).map(|year| year.get(1).unwrap().as_str().parse::<u32>().unwrap())
    }
}
