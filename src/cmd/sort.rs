use core::time;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anyhow::{bail, Ok, Result};

use indicatif::{ProgressBar,ProgressStyle, MultiProgress};
use once_cell::sync::Lazy;
use rayon::{prelude::*, ThreadPoolBuilder};

use serde_json::json;

use crate::cmd::{profile, Run, Sort};
use crate::episode::Episode;
use crate::search::result::MediaResult;
use crate::search::{self};

static MULTI_PROGRESS: Lazy<MultiProgress> = Lazy::new(|| MultiProgress::new());

impl Run for Sort {
    fn run(&mut self) -> Result<()> {
        self.setup_profile()?;
        self.validate_io()?;
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

    fn setup_profile(&mut self) -> Result<()> {
        if let Some(profile_name) = &self.profile {
            let profile = profile::get_profile_by_name(profile_name)?;
            let (input, output, flags) = profile::get_profile_properties(&profile)?;

            self.input = Some(PathBuf::from(input));
            self.output = Some(PathBuf::from(output));
            self.verbose = flags["verbose"].as_bool().unwrap_or(false);
            self.threads = flags["threads"].as_u64().map(|n| n as usize);
            self.recursive = flags["recursive"].as_bool().unwrap_or(false);
            self.webhook = flags["webhook"].as_str().map(|s| s.to_string());
            self.dry_run = flags["dry-run"].as_bool().unwrap_or(false);
            self.tv_template = flags["tv-template"].as_str().map(|s| s.to_string());
            self.movie_template = flags["movie-template"].as_str().map(|s| s.to_string());
            self.search = flags["search"].as_bool().unwrap_or(true);
        }
        Ok(())
    }

    fn validate_io(&self) -> Result<()> {
        if self.input.is_none() {
            bail!("Input directory is required");
        }

        if self.output.is_none() {
            bail!("Output directory is required");
        }


        if !self.input.clone().unwrap().exists() {
            bail!("Input directory does not exist");
        }
        Ok(())
    }

    fn verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    fn visit_dirs(&self, dir: &PathBuf, cb: &dyn Fn(&PathBuf) -> Result<()>) -> Result<()> {
        let paths: fs::ReadDir = fs::read_dir(dir.clone()).unwrap();

        for path in paths {
            let path: PathBuf = path.unwrap().path();
            if path.is_dir() {
                self.visit_dirs(&path, cb)?;
            } else {
                cb(&path)?;
            }
        }

        Ok(())
    }

    fn search_database(&self, episode: &mut Episode) -> Result<()> {
        let mut results;
        let name: String = episode.name.clone();
        if episode.is_movie {
            results = search::search_tmdb::search_movie_db(&episode.name, None, search::result::MediaType::Movie, false)?;
        } else {
            results = search::search_tvmaze::search_tvmaze(&episode.name, None, search::result::MediaType::Series)?;
            if results.is_empty() {
                results = search::search_tmdb::search_movie_db(&episode.name, None, search::result::MediaType::Series, false)?;
            }
        }
        let closest_result: Option<MediaResult> = search::result::get_highest_accuracy(results);
        if closest_result.is_some() {
            let best_result = closest_result.unwrap();
            let sanitized_name = sanitize_filename(&best_result.title);
            episode.set_name(sanitized_name.as_str());
            episode.year = best_result.year.parse::<u32>().ok();

        }
        else {
            episode.set_name(&name);
        }
        Ok(())
    }

    fn register_media(&self, path: &PathBuf, episodes: &mut Vec<Episode>, start_instant: &Instant) -> Result<()> {
        let mut episode: Episode = Episode::new(path);
        if self.search {
            self.search_database(&mut episode)?;
        }
        episodes.push(episode.clone());

        self.verbose(&format!(
            "Found media file {:?} in {:?}",
            episode.filename_clean,
            start_instant.elapsed()
        ));
        Ok(())
    }

    fn process_directory(&self, path: &PathBuf, episodes: &Mutex<Vec<Episode>> , has_media : &Mutex<bool>) -> Result<()> {
        self.visit_dirs(path, &|file_path| {
            *has_media.lock().unwrap() = true;
            let mut episodes_guard = episodes.lock().unwrap();
            self.register_and_move_media(&file_path, &mut episodes_guard)?;
            Ok(())
        })?;
        Ok(())
    }

    fn process_file(&self, path: &PathBuf, episodes: &Mutex<Vec<Episode>>) -> Result<()> {
        let mut episodes_guard = episodes.lock().unwrap();
        if self.is_media(path) {
            self.register_media(&path, &mut episodes_guard, &Instant::now())?;
        }
        Ok(())
    }

    fn move_series_folder(&self, path: &PathBuf, episode_name: String) -> Result<()> {
        let series_folder: PathBuf = path.parent().unwrap().parent().unwrap().to_path_buf();
        let dest_folder: PathBuf = self.output.clone().unwrap().join(self.tv_template.clone().unwrap()).join(episode_name);
        println!("Moving series folder {:?} to {:?}", series_folder, dest_folder);
        if is_on_same_drive(&series_folder, &dest_folder) {
            move_by_rename_recursive(&series_folder, &dest_folder)?;
        } else {
            move_by_copy_recursive(&series_folder, &dest_folder)?;
        }
        Ok(())
    }

    fn move_individual_file_to_root(&self, path: &PathBuf, episode: &Episode) -> Result<()> {
        let to = self.input.clone().unwrap().join(&episode.filename);
        fs::rename(&path, to.clone())?;
        Ok(())
    }

    fn register_and_move_media(&self, path: &PathBuf, episodes: &mut Vec<Episode>) -> Result<()> {
        let register_timer = Instant::now();
        if path.is_file() && self.is_media(path) {
            let mut episode: Episode = Episode::new(path);
            self.search_database(&mut episode)?;
            if episode.season == 0 {
                self.move_series_folder(path, episode.name)?;
            }
            else {
                self.move_individual_file_to_root(path, &episode)?;
                self.register_media(&path, episodes, &register_timer)?;
            }
        }
        Ok(())
    }

    fn check_media_status(&self, episodes: &Mutex<Vec<Episode>>, has_media : &Mutex<bool>) -> Result<()> {
        if episodes.lock().unwrap().is_empty() && !*has_media.lock().unwrap() {
            bail!("No media files found in the input directory");
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
            let path: PathBuf = path?.path();
            if self.recursive && path.is_dir() {
                self.process_directory(&path, &episodes, &has_media)?;

            } else {
                self.process_file(&path, &episodes)?;
            }
        }
        self.check_media_status(&episodes, &has_media)?;

        self.verbose(&format!(
            "Found {} media files in {:?}",
            episodes.lock().unwrap().len(),
            timer.elapsed()
        ));
        Ok(episodes.into_inner().unwrap())
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

    fn setup_thread_pool(&self) -> Result<()> {
        let max_cpu_count: usize = num_cpus::get() - 1;
        let num_threads: usize = self.threads.unwrap_or(max_cpu_count).min(max_cpu_count);

        if num_threads == 0 {
            bail!("Number of threads must be greater than 0");
        }

        // Configure global thread pool
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap();

        Ok(())
    }

    fn move_episodes(&self, episodes: &Vec<Episode>) -> Result<()> {
        let timer = Instant::now();
        let dir_set: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let pb = get_progress_bar(episodes.len());
        pb.set_message("Moving files");

        episodes.par_iter().try_for_each(|episode| -> Result<()> {
            let dest_dir: PathBuf = self.find_or_create_dir(&episode, dir_set.clone())?;
            pb.set_message(format!("Moving files - {}", episode.name));
            self.move_media(&episode, &dest_dir, &pb)?;
            pb.inc(1);
            Ok(())
        })?;

        pb.finish_with_message("Moving completed");

        self.verbose(&format!("Moved {} media files in {:?}", episodes.len(), timer.elapsed()));

        Ok(())
    }

    fn sort_medias_threaded(&self) -> Result<()> {
        self.verbose(&format!("Sorting medias in {:?}", self.input.clone().unwrap()));

        self.setup_thread_pool()?;

        let episodes: Vec<Episode> = self.get_medias_from_input()?;

        if episodes.is_empty() {
            return Ok(());
        }
        if self.dry_run {
            dry_run_sort(&episodes, self.tv_template.clone().unwrap(), self.movie_template.clone().unwrap())?;
            return Ok(());

        }
        self.move_episodes(&episodes)?;

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


        let dest_dir: PathBuf = self.get_dir_name(&episode);

        {
            let mut dir_set_guard = dir_set.lock().unwrap();

            if !dir_set_guard.contains(&dest_dir) && !dest_dir.exists() {
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
                PathBuf::from(serie_dir.clone()).join(format!("S{:02}", if episode.season == 0 { 1 } else { episode.season }));


            if dir_set.lock().unwrap().contains(&season_dir) {
                return Ok(season_dir);
            }

            fs::create_dir_all(&season_dir).expect("Failed to create directory");
            dir_set.lock().unwrap().insert(season_dir.clone());

            return Ok(season_dir);
        }

        Ok(dest_dir)
    }

    fn get_dir_name(&self, episode: &Episode) -> PathBuf {
        let mut dest_dir: PathBuf =
        PathBuf::from(&<Option<PathBuf> as Clone>::clone(&self.output).unwrap());
        if episode.is_movie {
            dest_dir.push(self.movie_template.clone().unwrap());
        } else {
            dest_dir.push(self.tv_template.clone().unwrap());
        }

        dest_dir
    }

    fn validate_move_paths(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        if !from.exists() {
            bail!("Source path does not exist: {:?}", from);
        } else if !from.is_file() {
            bail!("Source path is not a file: {:?}", from);
        } else if from.parent() == to.parent() {
            bail!("Source and destination directories are the same");
        } else if to.exists() {
            self.verbose(&format!("Destination path already exists: {:?}", to));
            return Ok(());
        }

        Ok(())
    }

    fn execute_file_move(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        if is_on_same_drive(&from, &to) {
            move_by_rename(&from, &to)?;
        } else {
            move_by_copy(&from, &to)?;
        }
        Ok(())
    }

    fn create_webhook_payload(&self, episode: &Episode) -> String {
        if episode.is_movie {
            format!("Added: `{}` to the library", episode.name)
        } else {
            if episode.episode >= 100 {
                format!("Added: `{} - S{:02}E{:03}` to the library", episode.name, episode.season, episode.episode)
            } else {
                format!("Added: `{} - S{:02}E{:02}` to the library", episode.name, episode.season, episode.episode)
            }
        }
    }

    fn send_webhook(&self, episode: &Episode, pb: &ProgressBar) -> Result<()> {
        if let Some(webhook) = self.webhook.as_ref() {
            if !webhook.is_empty() && webhook != "default" {
                let pb_msg = pb.message().to_string();
                if self.verbose {
                    pb.set_message( format!("{} - Sending webhook", pb_msg));
                }
                let message = self.create_webhook_payload(episode);

                let payload = json!({
                    "content": message,
                });

                let res = reqwest::blocking::Client::new().post(webhook).json(&payload).send()?;
                if !res.status().is_success() {
                    bail!("Failed to send webhook: {:?}", res);
                }
                if self.verbose {
                    pb.set_message( format!("{} - Sent webhook", pb_msg));
                }
            }
        }

        Ok(())
    }

    fn get_new_filename(&self, episode: &Episode) -> String {
        if episode.is_movie {
            format!("{}.{}", episode.name, episode.extension)
        } else {
            if episode.episode >= 100 {
                format!("{} - E{:03}.{}", episode.name, episode.episode, episode.extension)
            } else {
                format!("{} - E{:02}.{}", episode.name, episode.episode, episode.extension)
            }
        }
    }

    fn move_media(&self, episode: &Episode, dest_dir: &PathBuf, pb : &ProgressBar) -> Result<()> {
        let timer = Instant::now();
        let from_path = self.input.clone().unwrap().join(&episode.filename);
        let to_path = dest_dir.join(self.get_new_filename(episode));

        self.validate_move_paths(&from_path, &to_path)?;
        self.execute_file_move(&from_path, &to_path)?;
        if self.verbose {
            pb.set_message( format!("Moved {} to {} in {:?}", episode.filename_clean, to_path.to_str().unwrap(), timer.elapsed()));
        }
        self.send_webhook(&episode, pb)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn is_on_same_drive<P: AsRef<Path>>(path1: P, path2: P) -> bool {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    let path1_root = path1.components().next().unwrap();
    let path2_root = path2.components().next().unwrap();

    path1_root == path2_root
}

#[cfg(target_os = "linux")]
fn is_on_same_drive<P: AsRef<Path>, Q: AsRef<Path>>(path1: P, path2: Q) -> bool {
    use std::fs;

    let fs1 = fs::metadata(path1).expect("Unable to read metadata").dev();
    let fs2 = fs::metadata(path2).expect("Unable to read metadata").dev();

    fs1 == fs2
}

fn move_by_rename<P: AsRef<Path>>(from: P, to: P) -> Result<()> {
    fs::rename(from.as_ref(), to)?;
    Ok(())
}

fn move_by_copy<P: AsRef<Path> + Send + Sync>(from: P, to: P ) -> Result<()> {
    fs::copy(from.as_ref(), to)?;
    fs::remove_file(from)?;
    Ok(())
}

pub fn count_files<P: AsRef<Path>>(path: P) -> Result<usize> {
    let mut count = 0;
    let mut stack = std::collections::VecDeque::new();
    stack.push_back(PathBuf::from(path.as_ref()));

    while let Some(working_path) = stack.pop_front() {
        for entry in fs::read_dir(&working_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push_back(path);
            } else {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn move_by_copy_recursive<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), anyhow::Error> {
    let total_files = count_files(from.as_ref())?;
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    let pb = get_progress_bar(total_files);
    pb.set_message("Copying files");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    while let Some(working_path) = stack.pop() {
        pb.set_message("Copying files".to_owned() + " - Processing directory");
        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            pb.set_message("Copying files".to_owned() + " - Creating directory");
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
                        pb.set_message("Copying files".to_owned() + " -" + filename.to_str().unwrap());
                        fs::copy(&path, &dest_path)?;
                        pb.inc(1);
                    }
                    None => {
                        bail!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    fs::remove_dir_all(from)?;

    pb.finish_with_message("Copying completed");

    Ok(())
}

pub fn move_by_rename_recursive<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), anyhow::Error> {
    let total_files = count_files(from.as_ref())?;
    let mut stack = Vec::new();
    stack.push(PathBuf::from(from.as_ref()));

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();
    let pb = get_progress_bar(total_files);
    pb.set_message("Renaming files".to_owned() + " - Processing directory");

    while let Some(working_path) = stack.pop() {

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            pb.set_message("Renaming files".to_owned() + " - Creating directory");
            fs::create_dir_all(&dest)?;
        }



        for entry in fs::read_dir(working_path.clone())? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        pb.set_message("Renaming files".to_owned() + " - " + filename.to_str().unwrap());
                        fs::rename(&path, &dest_path)?;
                        pb.inc(1);

                    }
                    None => {
                        bail!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    fs::remove_dir_all(from)?;

    pb.finish_and_clear();
    Ok(())
}

fn get_progress_bar(len : usize) -> ProgressBar {
    let pb = MULTI_PROGRESS.add(indicatif::ProgressBar::new(len as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .progress_chars("#>-")
            .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {msg} {pos}/{len}")
            .unwrap()
    );
    pb.enable_steady_tick(time::Duration::from_millis(100));
    pb
}

fn sanitize_filename(filename: &str) -> String {
    let invalid_chars = ['<', '>', '"', '/', '|', '?', '*', ':'];
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    // Skip drive letter (e.g., "C:") when sanitizing
    let (drive, rest) = if filename.len() > 2 && &filename[1..2] == ":" {
        filename.split_at(2)
    } else {
        ("", filename)
    };

    // Sanitize the rest of the path, ignoring invalid characters
    let sanitized: String = rest.chars()
        .filter(|c| !invalid_chars.contains(c))
        .collect();
    let sanitized = sanitized.trim().to_string();

    // Ensure the sanitized filename is not a reserved name
    let sanitized = if reserved_names.contains(&sanitized.as_str()) {
        format!("{}_", sanitized)
    } else {
        sanitized
    };

    // Reattach the drive letter
    format!("{}{}", drive, sanitized)
}




 // Optimized printing function
 fn print_tree(
    dry_map: &HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>,
    prefix: &str,
    is_last: bool,
) {
    let connector = if is_last { "└─" } else { "├─" };

    for (i, (media_key, series_map)) in dry_map.iter().enumerate() {
        let is_last_media = i == dry_map.len() - 1;
        println!("{}{} {}/", prefix, connector, media_key);
        let new_prefix = format!("{}{}", prefix, if is_last_media { "   " } else { "│  " });

        for (j, (series_key, season_map)) in series_map.iter().enumerate() {
            let is_last_series = j == series_map.len() - 1;
            print_tree_inner(&season_map, series_key, &new_prefix, is_last_series);
        }
    }
}

fn print_tree_inner(
    season_map: &HashMap<String, Vec<String>>,
    series_key: &str,
    prefix: &str,
    is_last: bool,
) {
    let connector = if is_last { "└─" } else { "├─" };

    // Handle empty `series_key` for movies
    if !series_key.is_empty() {
        println!("{}{} {}/", prefix, connector, series_key);
    }
    let new_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });

    for (k, (season_key, episodes)) in season_map.iter().enumerate() {
        let is_last_season = k == season_map.len() - 1;

        if !season_key.is_empty() {
            println!("{}{} {}/", new_prefix, if is_last_season { "└─" } else { "├─" }, season_key);
        }
        let episode_prefix = if is_last_season {
            format!("{}   ", new_prefix)
        } else {
            format!("{}│  ", new_prefix)
        };

        for (l, episode) in episodes.iter().enumerate() {
            let is_last_episode = l == episodes.len() - 1;
            println!("{}{} {}", episode_prefix, if is_last_episode { "└─" } else { "├─" }, episode);
        }
    }
}

pub fn dry_run_sort(episodes: &Vec<Episode>, tv_template : String, movie_template : String) -> Result<()> {
    if episodes.is_empty() {
        bail!("No media files found in the input directory");
    }
    let dry_map: Arc<Mutex<HashMap<String, HashMap<String, HashMap<String, Vec<String>>>>>> = Arc::new(Mutex::new(HashMap::new()));

    episodes.par_iter().for_each(|episode| {
        let media_name = if episode.is_movie {
            movie_template.clone()
        } else {
            tv_template.clone()
        };
        if episode.is_movie {
            // Directly add the movie file under the media name (e.g., "Films")
            let movie_file = format!("{}.{}", episode.name, episode.extension);
            dry_map
                .lock()
                .unwrap()
                .entry(media_name.clone())
                .or_insert_with(HashMap::new)  // Insert a new HashMap<String, HashMap<String, Vec<String>>>
                .entry("".to_string())  // No series folder for movies, use empty string
                .or_insert_with(HashMap::new)  // Insert a new HashMap<String, Vec<String>>
                .entry("".to_string())  // No season folder for movies, use empty string
                .or_insert_with(Vec::new)  // Insert a new Vec<String> for the movie files
                .push(movie_file);
        } else {
            // Series logic
            let series_name = &episode.name;
            let season_key = format!("S{:02}", if episode.season == 0 { 1 } else { episode.season });
            let episode_key = if episode.episode > 100 {
                format!("{} - E{:03}.{}", episode.name, episode.episode, episode.extension)
            } else {
                format!("{} - E{:02}.{}", episode.name, episode.episode, episode.extension)
            };

            dry_map
                .lock()
                .unwrap()
                .entry(media_name.clone())
                .or_insert_with(HashMap::new)
                .entry(series_name.clone())
                .or_insert_with(HashMap::new)
                .entry(season_key)
                .or_insert_with(Vec::new)
                .push(episode_key);
        }
    });


    //print like the tree command
    print_tree(&dry_map.lock().unwrap(), "", true);
    Ok(())
}