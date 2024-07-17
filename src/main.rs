use dotenv::dotenv;
use reqwest::Client;
use env_logger::Env;
use std::time::Instant;
use std::env;
use log::info;

mod episode;
mod media_sort;
mod messager;

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

    media_sort::sort_medias(&download_dir, &server_root_dir, &discord_webhook_url, &client).await;


    info!("Total execution time: {:?}", timer.elapsed());
}
