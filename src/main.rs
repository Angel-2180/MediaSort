use dotenv::dotenv;
// use reqwest::{Client};
// use serde_json::json;
use log::{info, warn};
use std::{ env, fs, path::{Path, PathBuf}};

mod episode;
use crate::episode::Episode;

//  -> Result<(), reqwest::Error>

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file

    let download_dir: String = env::var("DOWNLOAD_DIR").expect("DOWNLOAD_DIR is not set");
    let server_root_dir: String = env::var("SERVER_ROOT_DIR").expect("SERVER_ROOT_DIR is not set");
    // let discord_webhook_url: String = env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL is not set");

    // let client: Client = Client::new();


    let episodes = get_medias(&download_dir);
    episodes[0].to_string();
    sort_medias(episodes, &download_dir, &server_root_dir);

    // let payload = json!({
    //     "content": ep.to_string()
    // });

    // let response = client.post(discord_webhook_url.clone())
    //     .json(&payload)
    //     .send()
    //     .await?;

    // if response.status().is_success() {
    //     println!("Message sent successfully");
    // } else {
    //     println!("Failed to send message: {}", response.status());
    // }

    // Ok(())
}




fn sort_medias(episodes: Vec<Episode>, download_dir: &str, server_root_dir: &str) {
    for episode in episodes {
        let dest_dir = find_or_create_dir(&episode, server_root_dir);
        move_file(&episode, download_dir, &dest_dir);
    }
}

fn get_medias(dir: &str) -> Vec<Episode> {
    info!("Getting medias in {}", dir);

    let paths = fs::read_dir(dir).unwrap();
    let mut episodes: Vec<Episode> = Vec::new();

    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension != "part" { // skip incomplet downloads
                // skip non-video files
                if extension == "mkv" || extension == "mp4" || extension == "avi" || extension == "wmv" || extension == "flv" || extension == "mov" || extension == "webm" {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let episode = Episode::new(filename);

                    episodes.push(episode);
                }
            }
        }
    }

    episodes
}

// helper functions to move file
fn move_file(episode: &Episode, source: &str, dest_dir: &str) {
    let source = Path::new(&source).join(&episode.filename);

    if !source.is_file() {
        panic!("File not found");
    }

    let dest = Path::new(&dest_dir).join(&episode.filename);
    let new_filename = format!("{} - {:02}.{}", episode.name, episode.episode, episode.extension).to_string();
    let to = Path::new(&dest_dir).join(&new_filename);

    if dest == to || to.is_file() {
        warn!("File already exists");
        return; //TODO: add a way to handle this like maybe destroying it ?
    }

    match fs::copy(&source, &dest) {
        Ok(_) => {
            info!("File moved successfully");
            match fs::rename(&dest, &to){
                Ok(_) => {
                    info!("File renamed successfully");

                }
                Err(e) => {
                    panic!("Error while renaming file: {}", e);
                }
            }
            match fs::remove_file(&source) {
                Ok(_) => {
                    info!("File removed successfully");

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

}

fn find_or_create_dir(episode: &Episode, location: &str) -> String {
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

    season_dir
}