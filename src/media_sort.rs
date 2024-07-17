use reqwest::Client;
use log::{trace, error, info, warn};
use std::{collections::HashSet, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};
use std::time::Instant;
use rayon::{prelude::*, ThreadPoolBuilder};


use crate::episode::Episode;
use crate::messager::send_message;

pub async fn sort_medias(download_dir: &str, server_root_dir: &str, discord_webhook_url: &str, client : &Client) {

    info!("Getting medias in {}", download_dir);
    let mut episodes = get_medias(&download_dir);
    let episodes_names: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    info!("Sorting medias...");
    let dir_set: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    sort_medias_parallel(&mut episodes, &episodes_names, &download_dir, &server_root_dir, dir_set).await;
    let episodes_names_guard = episodes_names.lock().unwrap();
    let mut episodes_names = episodes_names_guard.clone();
    drop(episodes_names_guard);

    send_message(&client, &discord_webhook_url, &mut episodes_names).await;
}



async fn sort_medias_parallel(episodes: &mut Vec<Episode>, episodes_names: &Arc<Mutex<Vec<String>>>, download_dir: &str, server_root_dir: &str, dir_set: Arc<Mutex<HashSet<String>>>) {
    let timer = Instant::now();
    info!("Timer started [sort_medias]: {:?}", timer.elapsed());

    let num_cpus = num_cpus::get() - 1;
    info!("Number of cpus: {}", num_cpus);
    let download_dir_str = download_dir.to_string();
    let server_root_dir_str = server_root_dir.to_string();

    ThreadPoolBuilder::new().num_threads(num_cpus).build_global().unwrap();
    episodes.par_iter_mut().for_each(|episode| {
        let mut dest_dir = format!("{}/Films", server_root_dir_str);

        {
            let mut dir_set_guard = dir_set.lock().unwrap();
            if !dir_set_guard.contains(&dest_dir) {
                fs::create_dir_all(&dest_dir).expect("Failed to create directory");
                dir_set_guard.insert(dest_dir.clone());
            }
        }

        if !episode.is_movie {
            dest_dir = find_or_create_dir(&episode, &server_root_dir_str, dir_set.clone());
            let mut episodes_names_guard = episodes_names.lock().unwrap();
            episodes_names_guard.push(episode.to_string());
        } else {
            let mut episodes_names_guard = episodes_names.lock().unwrap();
            episodes_names_guard.push(episode.name.to_string());
        }

        move_file_parallel(&episode, &download_dir_str, &dest_dir);
    });


    info!("Timer ended [sort_medias]: {:?}", timer.elapsed());
}


fn get_medias(dir: &str) -> Vec<Episode> {
    let timer = Instant::now();
    let paths = fs::read_dir(dir).unwrap();
    let mut episodes: Vec<Episode> = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension != "part" && ["mkv", "mp4", "avi", "wmv", "flv", "mov", "webm"].contains(&extension.to_str().unwrap()) {
                let episode = Episode::new(&path);
                episodes.push(episode);
            }
        }
    }

    info!("Found {} medias in {:?}", episodes.len(), timer.elapsed());

    episodes
}

fn move_file_parallel(episode: &Episode, source: &str, dest_dir: &str) {
    let timer = Instant::now();
    let source = Path::new(&source).join(&episode.filename);

    if !source.is_file() {
        panic!("File not found");
    }

    let dest = Path::new(&dest_dir).join(&episode.filename);
    let mut new_filename = format!("{} - E{:02}.{}", episode.name, episode.episode, episode.extension);

    if episode.is_movie {
        new_filename = format!("{}.{}", episode.name, episode.extension);
    }

    let to = Path::new(&dest_dir).join(&new_filename);

    if dest == to || to.is_file() {
        warn!("File already exists");
        return;
    }

    let copy_timer = Instant::now();

    if check_if_on_same_drive(&source, &to) {
        info!("timer ended [move_file:check_same_drive]: {:?}", timer.elapsed());
        return;
    }

    match fs::copy(&source, &dest) {
        Ok(_) => {
            info!("File moved successfully");
            info!("timer checkpoint [move_file:copy]: {:?}", copy_timer.elapsed());
            let rename_timer = Instant::now();
            match fs::rename(&dest, &to) {
                Ok(_) => {
                    info!("File renamed successfully");
                    info!("timer checkpoint [move_file:rename]: {:?}", rename_timer.elapsed());
                }
                Err(e) => {
                    panic!("Error while renaming file: {}", e);
                }
            }
            let remove_timer = Instant::now();
            match fs::remove_file(&source) {
                Ok(_) => {
                    info!("File removed successfully");
                    info!("timer checkpoint [move_file:remove]: {:?}", remove_timer.elapsed());
                }
                Err(e) => {
                    panic!("Error while removing file: {}", e);
                }
            };
        }
        Err(e) => {
            panic!("Error while moving file: {}", e);
        }
    }

    info!("timer ended [move_file]: {:?}", timer.elapsed());
}

fn check_if_on_same_drive(source: &PathBuf, to: &PathBuf) -> bool {
    fn get_drive_letter(path: &PathBuf) -> String {
        path.components().next().map(|c| c.as_os_str().to_string_lossy().to_string()).unwrap_or_default()
    }

    let source_drive = get_drive_letter(&source);
    let to_drive = get_drive_letter(&to);

    if source_drive == to_drive {
        let rename_timer = Instant::now();
        match fs::rename(&source, &to) {
            Ok(_) => {
                info!("File renamed successfully");
                info!("Timer checkpoint [move_file:rename]: {:?}", rename_timer.elapsed());
            }
            Err(e) => {
                error!("Error while renaming file: {}", e);
                return false;
            }
        }
        return true;
    }
    info!("Source drive: {}, is not the same as Destination drive: {}", source_drive, to_drive);
    false
}

fn find_or_create_dir(episode: &Episode, location: &str, dir_set: Arc<Mutex<HashSet<String>>>) -> String {
    let timer = Instant::now();
    if episode.name == "unknown" {
        panic!("Episode name is unknown");
    }
    let location_dir = PathBuf::from(location).join("Series").to_str().unwrap().to_string();
    if !dir_set.lock().unwrap().contains(&location_dir) {
        fs::create_dir_all(&location_dir).expect("Failed to create directory");
        dir_set.lock().unwrap().insert(location_dir.clone());
    }
    let series_dir = format!("{}/{}", location_dir, episode.name);
    let season_dir = format!("{}/S{:02}", series_dir, episode.season);

    let mut dir_set_lock = dir_set.lock().unwrap();

    if dir_set_lock.contains(&season_dir) {
        warn!("Directory already exists");
        return season_dir;
    }

    create_dir_if_not_exists(&mut dir_set_lock, &series_dir);
    create_dir_if_not_exists(&mut dir_set_lock, &season_dir);

    info!("timer ended [find_or_create_dir]: {:?}", timer.elapsed());
    season_dir
}

fn create_dir_if_not_exists(dir_set: &mut HashSet<String>, dir: &str) {
    let timer = Instant::now();
    if !dir_set.contains(&dir.to_string()) {
        if let Err(e) = fs::create_dir(dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                panic!("Error creating directory {}: {:?}", dir, e);
            }
        }
        dir_set.insert(dir.to_string());
        trace!("Created directory {} in {:?}", dir, timer.elapsed());
        return;
    }
    trace!("Found directory {} in {:?}", dir, timer.elapsed());
}