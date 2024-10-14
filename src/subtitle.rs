use std::path::PathBuf;
use crate::{episode::Episode, search::strings::*};

#[derive(Clone)]
pub struct Subtitle {
  pub full_path: PathBuf,
  pub filename_clean: String,
  pub episode: Episode,
  pub language: Option<String>,
}


impl Subtitle {
  pub fn new(full_path: PathBuf) -> Self {
    let filename = full_path.file_name().unwrap().to_str().unwrap().to_string();
    let filename_clean = clean_filename(&filename);
    let extension = full_path.extension().unwrap().to_str().unwrap().to_string();

    let name = extract_series_name(&filename_clean).unwrap();

    Self {
      episode: Episode {
        full_path: full_path.clone(),
        filename: filename.clone(),
        filename_clean: filename_clean.clone(),
        extension: extension.clone(),
        name,
        season: 0,
        episode: 0,
        is_movie: false,
        year: None,
      },
        full_path,
        filename_clean,
        language: None,
    }
  }

  pub fn set_episode(&mut self, episode: Episode) {
    self.episode = episode;
  }
}