use ffprobe::ffprobe;
use log::warn;
use regex::Regex;
use std::path::PathBuf;
use std::vec;

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
        };

        ep.fetch_infos();

        ep
    }

    pub fn to_string(&self) -> String {
        format!("{} - S{:02}E{:02}", self.name, self.season, self.episode)
    }

    fn fetch_infos(&mut self) {
        self.name = self.extract_series_name();
        self.season = self.extract_season();
        self.episode = self.extract_episode();
        self.extension = self.extract_extension();
        self.is_movie = self.is_movie();
    }

    fn clean_filename(filename_to_clean: &str) -> String {
        let mut cleaned = filename_to_clean.to_string();

        cleaned = cleaned.replace(&['.', '_', '-', '+'][..], " ");

        //remove unwanted patterns as [] and () content
        cleaned = Regex::new(r"\[.*?\]").unwrap().replace_all(&cleaned, "").to_string();
        cleaned = Regex::new(r"\(.*?\)").unwrap().replace_all(&cleaned, "").to_string();



        let unwanted_pattern = vec![
            "www", "com", "net", "org", "info", "mkv", "mp4", "avi", "wmv",
            "flv", "mov", "webm", "720p", "1080p", "x264", "x265", "HEVC",
            "MULTI", "AAC", "HD", "FRENCH", "VOSTFR", "VOSTA", "VF", "VO",
            "DL", "WEBRip", "WEB-DL", "WEB", "WEBRIP", "Rip", "RIP", "BluRay", "Blu-Ray", "Blu-ray",
            "WEB", "Film", "Movie", "TsundereRaws", "Tsundere", "Raws", "fit", "ws", "tv", "TV", "ec", "co", "vip",
            "vostfree", "boats", "uno", "Wawacity", "wawacity","H264", "cc", "red", "NanDesuKa", "FANSUB", "tokyo"
        ];

        for pattern in unwanted_pattern {
            cleaned = cleaned.replace(pattern, "");
        }

        cleaned.split_whitespace().collect::<Vec<&str>>().join(" ");

        cleaned = cleaned.trim().to_string();

        cleaned
    }

    fn extract_series_name(&self) -> String {
        let name_patterns = vec![
            r"(.+?)(S\d{1,2}E\d{1,2}|S\d{1,2})",
            r"(.+?)(E\d{1,2})",
            r"(.+?)(\d{1,3})",
            r"(.+?)(Film|Movie)",
            r"(.+)",
        ];

        for pattern in name_patterns {
            let re = Regex::new(pattern).unwrap();
            if let Some(captures) = re.captures(&self.filename_clean) {
                if let Some(name) = captures.get(1) {
                    return name.as_str().trim().to_string();
                }
            }
        }

        panic!("Name not found");
    }

    fn extract_season(&self) -> u32 {
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

    fn is_movie(&self) -> bool {
        if self.filename.contains("Film") || self.filename.contains("Movie") {
            return true;
        }
        if self.season == 0 && self.episode == 0 {
            return true;
        }

        match ffprobe(&self.full_path) {
            Ok(metadata) => {
                if let Some(duration) = metadata.format.duration {
                    if duration.parse::<f32>().unwrap_or(0.0) > 3000.0 {
                        return true;
                    }
                }
            }
            Err(e) => {
                warn!("Error while parsing file with ffprobe: {:?}", e);
                return false;
            }
        }

        false
    }
}
