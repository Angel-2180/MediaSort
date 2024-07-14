use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
use log::{trace, error, info, warn};
use env_logger::Env;
use std::{env, fs, path::{Path, PathBuf}};
use tokio::task;
use futures::future::join_all;
use std::time::Instant;

mod episode;
use crate::episode::Episode;

#[tokio::main]
async fn main() {
    let env = Env::default()
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

    info!("Sorting medias...");
    sort_medias(&episodes, &download_dir, &server_root_dir).await;

    for episode in &episodes {
        let mut payload = json!({
            "content": format!("Added: {} to the library!", episode.to_string()),
            "username": "Media Bot"
        });
        if episode.is_movie {
            payload = json!({
                "content": format!("Added: {} to the library!", episode.name),
                "username": "Media Bot"
            });
        }

        send_message(&client, &discord_webhook_url, &payload).await;
        // Sleep for 1 second to avoid rate limiting
        std::thread::sleep(std::time::Duration::from_secs(1));
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
        Ok(res) => error!("Failed to send message: {} in {:?}", res.status(), timer.elapsed()),
        Err(e) => error!("Failed to send message: {} in {:?}", e, timer.elapsed()),
    }
}

async fn sort_medias(episodes: &Vec<Episode>, download_dir: &str, server_root_dir: &str) {
    let timer = Instant::now();
    info!("Sorting medias started");

    let mut tasks = vec![];

    for episode in episodes {
        let download_dir = download_dir.to_string();
        let server_root_dir = server_root_dir.to_string();
        let episode = episode.clone();

        tasks.push(task::spawn(async move {
            let timer = Instant::now();
            let mut dest_dir = PathBuf::from(&server_root_dir).join("Films").to_str().unwrap().to_string();
            if !episode.is_movie {
                dest_dir = find_or_create_dir(&episode, &server_root_dir);
            }
            move_file(&episode, &download_dir, &dest_dir).await;
            trace!("Processed file in {:?}", timer.elapsed());
        }));
    }

    join_all(tasks).await;

    info!("Sorting medias completed in {:?}", timer.elapsed());
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

async fn move_file(episode: &Episode, source: &str, dest_dir: &str) {
    let timer = Instant::now();
    let source = Path::new(&source).join(&episode.filename);

    if !source.is_file() {
        panic!("File not found");
    }

    let dest = Path::new(&dest_dir).join(&episode.filename);
    let mut new_filename = format!("{} - E{:02}.{}", episode.name, episode.episode, episode.extension).to_string();
    if episode.is_movie {
        new_filename = format!("{}.{}", episode.name, episode.extension).to_string();
    }

    let to = Path::new(&dest_dir).join(&new_filename);

    if dest == to || to.exists() {
        warn!("File already exists");
        return;
    }

    let dest_clone = dest.clone();
    let source_clone = source.clone();
    let copy_result = task::spawn_blocking(move || fs::copy(&source_clone, &dest_clone)).await.unwrap();
    match copy_result {
        Ok(_) => {
            info!("File copied successfully in {:?}", timer.elapsed());
            let rename_result = task::spawn_blocking(move || fs::rename(&dest, &to)).await.unwrap();
            match rename_result {
                Ok(_) => {
                    let source_clone = source.clone();
                    let remove_result = task::spawn_blocking(move || fs::remove_file(&source_clone)).await.unwrap();
                    match remove_result {
                        Ok(_) => info!("File removed successfully in {:?}", timer.elapsed()),
                        Err(e) => panic!("Error while removing file: {}", e),
                    }
                }
                Err(e) => panic!("Error while renaming file: {}", e),
            }
        }
        Err(e) => panic!("Error while moving file: {}", e),
    }
}

fn find_or_create_dir(episode: &Episode, location: &str) -> String {
    let timer = Instant::now();
    if episode.name == "unknown" {
        panic!("Episode name is unknown");
    }
    let location_dir = PathBuf::from(location).join("Series").to_str().unwrap().to_string();
    let series_dir = format!("{}/{}", location_dir, episode.name);
    let season_dir = format!("{}/S{:02}", series_dir, episode.season);

    create_dir_if_not_exists(&series_dir);
    create_dir_if_not_exists(&season_dir);

    info!("Directory created in {:?}", timer.elapsed());

    season_dir
}

fn create_dir_if_not_exists(dir: &str) {
    let timer = Instant::now();
    if !Path::new(dir).is_dir() {
        if let Err(e) = fs::create_dir(dir) {
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                panic!("Error creating directory {}: {:?}", dir, e);
            }
        }
        trace!("Created directory {} in {:?}", dir, timer.elapsed());
        return;
    }
    trace!("Checked/created directory {} in {:?}", dir, timer.elapsed());
}
