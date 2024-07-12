use dotenv::dotenv;
use reqwest::{Client};
use serde_json::json;
use log::{info, warn};
use std::{ env, fs, path::{Path, PathBuf}};

mod episode;
use crate::episode::Episode;

//  -> Result<(), reqwest::Error>

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file
    env_logger::init();
    info!("Starting...");
    let timer = std::time::Instant::now();
    let download_dir: String = env::var("DOWNLOAD_DIR").expect("DOWNLOAD_DIR is not set");
    let server_root_dir: String = env::var("SERVER_ROOT_DIR").expect("SERVER_ROOT_DIR is not set");
    let discord_webhook_url: String = env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL is not set");

    let client: Client = Client::new();
    info!("Getting medias in {}", download_dir);
    let episodes = get_medias(&download_dir);
    info!("Sorting medias...");
    sort_medias(&episodes, &download_dir, &server_root_dir);
    for episode in episodes {
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
    }


    info!("Execution time: {:?}", timer.elapsed());
}


async fn send_message(client: &Client, url: &str, payload: &serde_json::Value) {
    let response = client.post(url)
        .json(&payload)
        .send()
        .await
        .expect("Failed to send message");

    if response.status().is_success() {
        println!("Message sent successfully");
    } else {
        println!("Failed to send message: {}", response.status());
    }
}


fn sort_medias(episodes: &Vec<Episode>, download_dir: &str, server_root_dir: &str) {
    let timer = std::time::Instant::now();
    info!("Timer started [sort_medias]: {:?}", timer.elapsed());
    for episode in episodes {
        let mut dest_dir = (PathBuf::from(server_root_dir).join("Films")).to_str().unwrap().to_string();
        if !episode.is_movie {
            dest_dir = find_or_create_dir(&episode, server_root_dir);
        }
        move_file(&episode, download_dir, &dest_dir);
    }
    info!("Timer ended [sort_medias]: {:?}", timer.elapsed());
}

fn get_medias(dir: &str) -> Vec<Episode> {
    let timer = std::time::Instant::now();
    let paths = fs::read_dir(dir).unwrap();
    let mut episodes: Vec<Episode> = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension != "part" { // skip incomplet downloads
                // skip non-video files
                if extension == "mkv" || extension == "mp4" || extension == "avi" || extension == "wmv" || extension == "flv" || extension == "mov" || extension == "webm" {
                    let episode = Episode::new(&path);
                    episodes.push(episode);
                }
            }
        }
    }
    info!("Found {} medias", episodes.len());
    info!("timer ended [get_medias]: {:?}", timer.elapsed());

    episodes
}

// helper functions to move file
fn move_file(episode: &Episode, source: &str, dest_dir: &str) {
    let timer = std::time::Instant::now();
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

    if dest == to || to.is_file() {
        warn!("File already exists");
        return; //TODO: add a way to handle this like maybe destroying it ?
    }
    let copy_timer = std::time::Instant::now();
    match fs::copy(&source, &dest) {
        Ok(_) => {
            info!("File moved successfully");
            info!("timer checkpoint [move_file:copy]: {:?}", copy_timer.elapsed());
            let rename_timer = std::time::Instant::now();
            match fs::rename(&dest, &to){
                Ok(_) => {
                    info!("File renamed successfully");
                    info!("timer checkpoint [move_file:rename]: {:?}", rename_timer.elapsed());

                }
                Err(e) => {
                    panic!("Error while renaming file: {}", e);
                }
            }
            let remove_timer = std::time::Instant::now();

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

fn find_or_create_dir(episode: &Episode, location: &str) -> String {
    let timer = std::time::Instant::now();
    if episode.name == "unknown" {
        panic!("Episode name is unknown");
    }
    let location_dir = (PathBuf::from(location).join("Series")).to_str().unwrap().to_string();
    let series_dir = format!("{}/{}", location_dir, episode.name);
    let season_dir = format!("{}/S{:02}", series_dir, episode.season);

    if !Path::new(&series_dir).exists() {
        fs::create_dir(&series_dir).unwrap();
    }

    if !Path::new(&season_dir).exists() {
        fs::create_dir(&season_dir).unwrap();
    }

    info!("timer ended [find_or_create_dir]: {:?}", timer.elapsed());

    season_dir
}
