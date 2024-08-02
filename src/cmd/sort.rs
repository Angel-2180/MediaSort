use core::num;
use std::collections::HashSet;
use std::fs;
use std::hash::Hash;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{bail, Ok, Result};

use once_cell::sync::Lazy;
use rayon::{prelude::*, ThreadPoolBuilder};

use serde_json::json;

use crate::cmd::profile::get_profile_by_name;
use crate::cmd::{profile, Run, Sort};
use crate::episode::Episode;

impl Run for Sort {
    fn run(&mut self) -> Result<()> {
        if self.profile.is_some() {
            //TODO: implement profiles
            let profile = get_profile_by_name(self.profile.as_ref().unwrap())?;
            let (input, output, flags) = profile::get_profile_properties(&profile)?;
            self.input = Some(PathBuf::from(input));
            println!("input: {:?}", self.input.clone().unwrap());
            self.output = Some(PathBuf::from(output));
            println!("output: {:?}", self.output.clone().unwrap());

            self.verbose = flags["verbose"].as_bool().unwrap_or(false);
            self.threads = flags["threads"].as_u64().map(|n| n as usize);
            self.recursive = flags["recursive"].as_bool().unwrap_or(false);
            self.webhook = flags["webhook"].as_str().map(|s| s.to_string());
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

    fn visit_dirs(&self, dir: &PathBuf, cb: &dyn Fn(&PathBuf) -> Result<()>) -> Result<()> {
        let paths: fs::ReadDir = fs::read_dir(dir.clone()).unwrap();
        println!("dir: {:?}", dir);

        for path in paths {
            let path: PathBuf = path.unwrap().path();
            if path.is_dir() {
                println!("path: {:?}", path);
                //search if directory name is S[0-999]+ without regex

                self.visit_dirs(&path, cb)?;
            } else {
                cb(&path)?;
            }
        }

        Ok(())
    }

    fn register_media(&self, path: &PathBuf, episodes: &mut Vec<Episode>, start_instant: &Instant) -> Result<()> {
        if self.is_media(path) {
            let episode: Episode = Episode::new(path);
            episodes.push(episode.clone());

            self.verbose(&format!(
                "Found media file {:?} in {:?}",
                episode.filename_clean,
                start_instant.elapsed()
            ));
        }
        Ok(())
    }


    fn get_medias_from_input(&self) -> Result<Vec<Episode>> {
        let timer = Instant::now();

        let input_path = self.input.clone();
        let paths: fs::ReadDir = fs::read_dir(input_path.unwrap()).unwrap();
        let episodes: Mutex<Vec<Episode>> = Vec::new().into();
        let has_media = Mutex::new(false);
        for path in paths {
            let start_instant: Instant = Instant::now();
            let path: PathBuf = path.unwrap().path();

            if self.recursive && path.is_dir() {
                //we visit directories recursively to find all media files
                //we want once all media files to be found and sorted/moved to delete the empty directories
                self.visit_dirs(&path,  &|path| {

                    *has_media.lock().unwrap() = true;

                    let register_timer = Instant::now();
                    let mut episodes_guard = episodes.lock().unwrap();
                    //move the file to the source directory

                    if path.is_file() && self.is_media(path) {
                        let episode: Episode = Episode::new(path);
                        if episode.season == 0 {

                            let series_folder: PathBuf = path.parent().unwrap().to_path_buf().parent().unwrap().to_path_buf();
                            let to = self.output.clone().unwrap().join("Series").join(&episode.name);

                            if is_on_same_drive(&series_folder, &to.clone()) {
                                move_by_rename_recursive(&series_folder, &to)?;
                            }
                            else {
                               move_by_copy_recursive(&series_folder, &to)?;
                            }

                        }
                        else {
                            //we move the file to the source directory
                            let to = self.input.clone().unwrap().join(&episode.filename);
                            fs::rename(&path, to.clone()).unwrap();
                            self.register_media(&to, &mut episodes_guard, &register_timer).unwrap();
                        }
                    }
                    Ok(())
                }).unwrap();


            }

            self.register_media(&path, &mut episodes.lock().unwrap(), &start_instant).unwrap();
        }

        let episodes = episodes.into_inner().unwrap();
        let has_media = has_media.into_inner().unwrap();

        if episodes.is_empty() && !has_media {
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
        self.verbose(&format!("Sorting medias in {:?}", self.input.clone().unwrap()));

        let mut episodes: Vec<Episode> = self.get_medias_from_input()?;

        if episodes.is_empty() {
            return Ok(());
        }
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

        episodes.par_iter_mut().try_for_each(|episode| {
            let dest_dir: PathBuf = self.find_or_create_dir(&episode, dir_set.clone()).unwrap();
            self.move_media(&episode, &dest_dir)
        })?;

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


        self.verbose(&format!(
            "Moving {:?} to {:?}",
            episode.name.clone() + "." + &episode.extension.clone(),
            to_dir
        ));

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
            self.verbose(&format!(
                "File already exists: {:?} in {:?}",
                to_path, timer.elapsed()
            ));
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

        if self.webhook.is_some() && !self.webhook.as_ref().unwrap().is_empty() && self.webhook.as_ref().unwrap() != "" {
            println!("Sending webhook");
            let mut message = format!(
                "Added: `{} - S{:02}E{:02}` to the library",
                episode.name, episode.season, episode.episode
            );
            if episode.is_movie {
                message = format!("Added: `{}` to the library", episode.name);
            }

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

pub fn move_by_copy_recursive<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), anyhow::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        bail!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    fs::remove_dir_all(from)?;

    Ok(())
}


pub fn move_by_rename_recursive<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), anyhow::Error> {
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  rename: {:?} -> {:?}", &path, &dest_path);
                        fs::rename(&path, &dest_path)?;
                    }
                    None => {
                        bail!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    fs::remove_dir_all(from)?;

    Ok(())
}
