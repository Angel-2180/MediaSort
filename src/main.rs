use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
use log::{trace, error, info, warn};
use env_logger::Env;
use std::{collections::HashSet, env, fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};
use std::time::Instant;
use rayon::prelude::*;

mod episode;
use crate::episode::Episode;

#[tokio::main]
async fn main() {
    let env: Env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);
    dotenv().ok(); // Load .env file

    let timer = Instant::now();
    let download_dir: String = env::var("DOWNLOAD_DIR").expect("DOWNLOAD_DIR is not set");
    let server_root_dir: String = env::var("SERVER_ROOT_DIR").expect("SERVER_ROOT_DIR is not set");
    let discord_webhook_url: String = env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL is not set");
    let client = Client::new();

    info!("Getting medias in {}", download_dir);
    let episodes = get_medias(&download_dir);
    let mut episodes_names = Vec::with_capacity(episodes.len());
    info!("Sorting medias...");
    let dir_set: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::new()));
    sort_medias_parralel(&episodes, &mut episodes_names, &download_dir, &server_root_dir, dir_set).await;
    episodes_names.sort_unstable();
    let mut content = episodes_names
        .iter()
        .map(|name| {
            let parts: Vec<&str> = name.split(" - ").collect();
            let media_name = parts[0];
            let episode_info = parts.get(1).unwrap_or(&"");
            format!("Added: *{}* - **{}** to the library!", media_name, episode_info)
        })
        .collect::<Vec<String>>()
        .join("\n");

    while !content.is_empty() {
        let message = if content.len() > 2000 {
            let mut chunk = content.split_off(2000);
            if let Some(index) = chunk.rfind('\n') {
                let truncated = chunk.split_off(index);
                content.push_str(chunk.as_str());
                truncated
            } else {
                let truncated = content.clone();
                content.clear();
                truncated
            }
        } else {
            let chunk = content.clone();
            content.clear();
            chunk
        };
        let message_payload = json!({
            "content": message,
            "username": "Media Bot"
        });

        send_message(&client, &discord_webhook_url, &message_payload).await;
    }


    info!("Total execution time: {:?}", timer.elapsed());
}

async fn send_message(client: &Client, url: &str, payload: &serde_json::Value) {
    let timer = Instant::now();
    let response = client.post(url)
        .json(&payload)
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => trace!("Message sent successfully in {:?}", timer.elapsed()),
        Ok(res) => {
            let status = res.status();
            let body = res.text().await.unwrap_or_else(|_| "Failed to get response body".to_string());
            error!("Failed to send message: {} in {:?} with body: {}", status, timer.elapsed(), body);
        },
        Err(e) => error!("Failed to send message: {} in {:?}", e, timer.elapsed()),
    }
}

async fn sort_medias_parralel(episodes: &Vec<Episode>, episodes_names: &mut Vec<String>, download_dir: &str, server_root_dir: &str, dir_set: Arc<Mutex<HashSet<String>>>) {
    let timer = Instant::now();
    info!("Timer started [sort_medias]: {:?}", timer.elapsed());

    let num_cpus = num_cpus::get() - 1;
    info!("Number of cpus: {}", num_cpus);

    let names: Vec<String> = batch_processing(episodes, &download_dir, &server_root_dir, num_cpus, dir_set).await;

    episodes_names.extend(names);

    info!("Timer ended [sort_medias]: {:?}", timer.elapsed());
}

async fn batch_processing(episodes: &Vec<Episode>, download_dir: &str, server_root_dir: &str, num_cpus: usize, dir_set: Arc<Mutex<HashSet<String>>>) -> Vec<String> {
    let timer = Instant::now();
    let episodes_chunks: Vec<_> = episodes.chunks(num_cpus).collect();

    let names: Vec<Vec<String>> = episodes_chunks.par_iter().map(|episodes_to_drain| {
        let mut local_names = Vec::new();
        parralel_iterator(episodes_to_drain, download_dir, server_root_dir, dir_set.clone());

        for episode in *episodes_to_drain {
            if episode.is_movie {
                local_names.push(episode.name.to_string());
                continue;
            }
            local_names.push(episode.to_string());
        }
        local_names
    }).collect();

    info!("Timer ended [batch_processing]: {:?}", timer.elapsed());

    names.into_iter().flatten().collect()
}

fn parralel_iterator(episodes: &[Episode], download_dir: &str, server_root_dir: &str, dir_set: Arc<Mutex<HashSet<String>>>) {
    episodes.par_iter().for_each(|episode| {
        let download_dir = download_dir.to_string();
        let server_root_dir = server_root_dir.to_string();
        let episode = episode.clone();
        let mut dest_dir = PathBuf::from(&server_root_dir).join("Films").to_str().unwrap().to_string();

        if !episode.is_movie {
            dest_dir = find_or_create_dir(&episode, &server_root_dir, dir_set.clone());
        }

        move_file_parralel(&episode, &download_dir, &dest_dir);
    });
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

fn move_file_parralel(episode: &Episode, source: &str, dest_dir: &str) {
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
        info!("timer ended [move_file]: {:?}", timer.elapsed());
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
    if source.as_path().starts_with(to.as_path()) {
        let rename_timer = Instant::now();
        match fs::rename(&source, &to) {
            Ok(_) => {
                info!("File renamed successfully");
                info!("timer checkpoint [move_file:rename]: {:?}", rename_timer.elapsed());
            }
            Err(e) => {
                panic!("Error while renaming file: {}", e);
            }
        }
        return true;
    }
    false
}

fn find_or_create_dir(episode: &Episode, location: &str, dir_set: Arc<Mutex<HashSet<String>>>) -> String {
    let timer = Instant::now();
    if episode.name == "unknown" {
        panic!("Episode name is unknown");
    }
    let location_dir = PathBuf::from(location).join("Series").to_str().unwrap().to_string();
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
