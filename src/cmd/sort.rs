use core::num;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{bail, Ok, Result};

use once_cell::sync::Lazy;
use rayon::{prelude::*, ThreadPoolBuilder};

use serde_json::json;

use crate::cmd::{Run, Sort};
use crate::episode::Episode;

impl Run for Sort {
    fn run(&self) -> Result<()> {
        if self.profile.is_some() {
            //TODO: implement profiles
            bail!("Profiles are not implemented yet");
        }

        if self.input.is_none() {
            bail!("Input directory is required");
        }

        if self.output.is_none() {
            bail!("Output directory is required");
        }

        let global_timer = Instant::now();

        self.sort_medias_threaded()?;

        println!(
            "\nMedias sorted successfully in {:?}",
            global_timer.elapsed()
        );

        Ok(())
    }
}

impl Sort {
    fn verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    fn get_medias_from_input(&self) -> Result<Vec<Episode>> {
        let timer = Instant::now();

        let input_path = self.input.clone();
        let paths: fs::ReadDir = fs::read_dir(input_path.unwrap()).unwrap();
        let mut episodes: Vec<Episode> = Vec::new();

        for path in paths {
            let temp: Instant = Instant::now();
            let path: PathBuf = path.unwrap().path();

            if self.is_media(&path) {
                let episode: Episode = Episode::new(&path);
                episodes.push(episode.clone());

                self.verbose(&format!(
                    "Found media file {:?} in {:?}",
                    episode.filename_clean,
                    temp.elapsed()
                ));
            }
        }

        if episodes.is_empty() {
            bail!("No media files found in the input directory");
        }

        self.verbose(&format!(
            "Found {} media files in {:?}",
            episodes.len(),
            timer.elapsed()
        ));

        Ok(episodes)
    }

    fn is_media(&self, path: &PathBuf) -> bool {
        static MEDIA_EXTENSIONS: Lazy<HashSet<&str>> = Lazy::new(|| {
            ["mp4", "mkv", "avi", "mov", "flv", "wmv", "webm"]
                .iter()
                .cloned()
                .collect()
        });

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext_str| MEDIA_EXTENSIONS.contains(&ext_str))
            .unwrap_or(false)
    }

    fn sort_medias_threaded(&self) -> Result<()> {
        self.verbose(&format!("Sorting medias in {:?}", self.input));

        let mut episodes: Vec<Episode> = self.get_medias_from_input()?;
        let dir_set: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));

        let max_cpu_count: usize = num_cpus::get() - 1;
        let mut num_threads: usize = self.threads.unwrap_or(max_cpu_count);

        if num_threads > max_cpu_count {
            num_threads = max_cpu_count;

            println!(
                "Number of threads is greater than the number of CPUs. Unlimited PAAAAWAAAAR! ({} threads)",
                num_threads
            );
        }

        if num_threads == 0 {
            bail!("Number of threads must be greater than 0");
        }

        // Configure global thread pool
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap();

        episodes.par_iter_mut().for_each(|episode| {
            let dest_dir: PathBuf = self.find_or_create_dir(&episode, dir_set.clone()).unwrap();

            let _ = self.move_media(&episode, &dest_dir);
        });

        Ok(())
    }

    fn find_or_create_dir(
        &self,
        episode: &Episode,
        dir_set: Arc<Mutex<HashSet<PathBuf>>>,
    ) -> Result<PathBuf> {
        if episode.name == "unknow" {
            bail!("Episode name is unknow");
        }

        let mut dest_dir: PathBuf =
            PathBuf::from(&<Option<PathBuf> as Clone>::clone(&self.output).unwrap());

        if episode.is_movie {
            dest_dir.push("Films");
        } else {
            dest_dir.push("Series");
        }

        {
            let mut dir_set_guard = dir_set.lock().unwrap();

            if !dir_set_guard.contains(&dest_dir) {
                fs::create_dir_all(&dest_dir).expect("Failed to create directory");
                dir_set_guard.insert(dest_dir.clone());
            }
        }

        if !episode.is_movie {
            let serie_dir = PathBuf::from(dest_dir.clone()).join(&episode.name);
            if dir_set.lock().unwrap().contains(&serie_dir) {
                return Ok(serie_dir);
            }

            let season_dir: PathBuf =
                PathBuf::from(serie_dir.clone()).join(format!("S{:02}", &episode.season));

            if dir_set.lock().unwrap().contains(&season_dir) {
                return Ok(season_dir);
            }

            fs::create_dir_all(&season_dir).expect("Failed to create directory");
            dir_set.lock().unwrap().insert(season_dir.clone());

            return Ok(season_dir);
        }

        Ok(dest_dir)
    }

    fn move_media(&self, episode: &Episode, dest_dir: &PathBuf) -> Result<()> {
        let timer = Instant::now();
        let from_dir: PathBuf = <Option<PathBuf> as Clone>::clone(&self.input)
            .unwrap()
            .clone();
        let to_dir: PathBuf = dest_dir.clone();

        let from_path: PathBuf = from_dir.join(&episode.filename);
        if !from_path.exists() {
            bail!("File does not exist: {:?}", from_path);
        } else if !from_path.is_file() {
            bail!("Path is not a file: {:?}", from_path);
        }

        let new_filename: String;

        if episode.is_movie {
            new_filename = format!("{}.{}", episode.name, episode.extension);
        } else {
            if episode.episode >= 100 {
                new_filename = format!(
                    "{} - E{:03}.{}",
                    episode.name, episode.episode, episode.extension
                );
            } else {
                new_filename = format!(
                    "{} - E{:02}.{}",
                    episode.name, episode.episode, episode.extension
                );
            }
        }

        let to_path: PathBuf = to_dir.join(new_filename.clone());
        if from_dir == to_dir {
            bail!("Source and destination directories are the same");
        } else if to_path.exists() {
            //if already exists, skip
            return Ok(());
        }

        if is_on_same_drive(&from_path.clone(), &to_path.clone()) {
            move_by_rename(&from_path, &to_path)?;
        } else {
            move_by_copy(&from_path, &to_path)?;
        }

        self.verbose(&format!(
            "Moved {:?} to {:?} in {:?}",
            episode.filename_clean,
            to_path.to_str().unwrap(),
            timer.elapsed()
        ));

        if self.webhook.is_some() {
            let message = format!(
                "Added: `{} - S{:02}E{:02}` to the library",
                episode.name, episode.season, episode.episode
            );

            let payload = json!({
                "content": message,
            });

            let client = reqwest::blocking::Client::new();

            let res = client
                .post(self.webhook.as_ref().unwrap())
                .json(&payload)
                .send()?;

            if !res.status().is_success() {
                bail!("Failed to send webhook: {:?}", res);
            }

            self.verbose(&format!("Sent webhook: {:?}", payload));
        }

        Ok(())
    }
}

fn is_on_same_drive(source: &PathBuf, dest: &PathBuf) -> bool {
    let drive1: Component = source.components().next().unwrap();
    let drive2: Component = dest.components().next().unwrap();

    drive1 == drive2
}

fn move_by_copy(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let copy_res = fs::copy(from, to);
    if let Err(e) = copy_res {
        bail!("Failed to move file: {:?}", e);
    }

    let del_res = fs::remove_file(from);
    if let Err(e) = del_res {
        bail!("Failed to delete source media: {:?}", e);
    }

    Ok(())
}

fn move_by_rename(from: &PathBuf, to: &PathBuf) -> Result<()> {
    let rename_res = fs::rename(from, to);
    if let Err(e) = rename_res {
        bail!("Failed to move file: {:?}", e);
    }

    Ok(())
}
