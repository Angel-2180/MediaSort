use std::vec;

use regex::Regex;

pub struct Episode {
  pub filename: String,
  pub filename_clean: String,
  pub extension: String,

  pub name: String,
  pub season: u32,
  pub episode: u32,
}

impl Episode {
  pub fn new(filename: &str) -> Self {
    let filename_clean = Self::clean_filename(filename);

    let mut ep = Episode {
      filename: filename.to_string(),
      filename_clean: filename_clean.clone(),

      name: "unknown".to_string(),
      season: 0,
      episode: 0,

      extension: "unknown".to_string(),
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
  }

  fn clean_filename(filename_to_clean: &str) -> String {
    let mut cleaned = filename_to_clean.to_string();

    // replace '-._+' with ' '
    cleaned = cleaned.replace("-", " ");
    cleaned = cleaned.replace(".", " ");
    cleaned = cleaned.replace("_", " ");
    cleaned = cleaned.replace("+", " ");

    let patterns = vec![
      r"(www\..*?\..{2,3})",
      r"\(.*?\)",
      r"\[.*?\]",
      r"(mkv|mp4|avi|wmv|flv|mov|webm)",
      r"\b\d{3,4}p\b",
      r"(x264|x265|HEVC|MULTI|AAC|HD)",
      r"(FRENCH|VOSTFR|VOSTA|VF|VO)",
      r"(www|com|vostfree|boats|uno|Wawacity|WEB|TsundereRaws|Tsundere|Raws|fit|ws|tv|TV)"
    ];

    for pattern in patterns {
      let re = Regex::new(pattern).unwrap();
      cleaned = re.replace_all(&cleaned, "").to_string();
    }

    let re = Regex::new(r"(?m)^ +| +$| +( )").unwrap();
    cleaned = re.replace_all(&cleaned, " ").to_string();

    // // remove consecutive spaces
    // cleaned = cleaned.replace("  ", " ");

    // remove leading and trailing spaces
    cleaned = cleaned.trim().to_string();

    cleaned
  }

  fn extract_series_name(&self) -> String {
    let name_patterns = vec![
      r"(.+?)(S\d{1,2}E\d{1,2}|S\d{1,2})",
      r"(.+?)(\d{1,3})",
      r"(.+?)(Film|Movie)",
      r"(.+)"
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
    let season_pattern = r"S(\d{1,2})E\d{1,2}";
    let re = Regex::new(season_pattern).unwrap();

    if let Some(captures) = re.captures(&self.filename_clean) {
      if let Some(season) = captures.get(1) {
        return season.as_str().parse::<u32>().unwrap_or(1);
      }
    }

    1
  }

  fn extract_episode(&self) -> u32 {
    let episode_patterns = vec![
      r"S\d{1,2}E(\d{1,2})",
      r"\b(\d{1,4})\b"
    ];

    for pattern in episode_patterns {
      let re = Regex::new(pattern).unwrap();
      if let Some(captures) = re.captures(&self.filename_clean) {
        if let Some(episode) = captures.get(1) {
          return episode.as_str().parse::<u32>().unwrap_or(1);
        }
      }
    }

    1
  }

  fn extract_extension(&self) -> String {
    let extension_pattern = r"\.(mkv|mp4|avi|wmv|flv|mov|webm)";
    let re = Regex::new(extension_pattern).unwrap();

    if let Some(captures) = re.captures(&self.filename) {
      if let Some(extension) = captures.get(1) {
        return extension.as_str().to_string();
      }
    }

    panic!("Extension not found");
  }
}