use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;

mod episode;
use crate::episode::Episode;

//  -> Result<(), reqwest::Error>

#[tokio::main]
async fn main() {
    dotenv().ok(); // Load .env file

    let download_dir: String = std::env::var("DOWNLOAD_DIR").expect("DOWNLOAD_DIR is not set");
    let server_root_dir: String = std::env::var("SERVER_ROOT_DIR").expect("SERVER_ROOT_DIR is not set");
    let discord_webhook_url: String = std::env::var("DISCORD_WEBHOOK_URL").expect("DISCORD_WEBHOOK_URL is not set");

    let client: Client = Client::new();

    // array of filenames testcase
    let filenames: [&str; 13] = [
        "A.Sign.of.Affection.S99E01.VOSTFR.1080p.WEB.x264-TsundereRaws-Wawacity.boats.mkv",
        "My Deer Friend Nokotan S01E01 VOSTFR 1080p WEB x264 AAC -Tsundere-Raws (CR).mp4",
        "Edens.Zero.S02E01.FRENCH.1080p.WEB.x264-TsundereRaws-Wawacity.uno.mkv",
        "Dragon Ball 001 Bulma et Son Goku.mkv",
        "[Mixouille] Bleach Kai - 049 - Les gardiens de nos âmes - 720p.MULTI.x264.mkv",
        "Komi-san.wa.Komyushou.Desu.01.VOSTFR.1080p.www.vostfree.tv.mp4",
        "Kaguya-sama.wa.Kokurasetai.Ultra.Romantic.01.VOSTFR.1080p.www.vostfree.tv.mp4",
        "Chou.Kadou.Girl.⅙.Amazing.Stranger.11.VOSTFR.720p.www.vostfree.com.mp4",
        "DanMachi.S4.22.VOSTFR.1080p.www.vostfree.ws.mp4",
        "Dr.Stone.S2.11.FiNAL.VF.1080p.www.vostfree.com.mp4",
        "My.Love.Story.with.Yamada-kun.at.Lv999.S01E02.VOSTFR.1080p.WEB.x264-TsundereRaws-Wawacity.uno",
        "The.100.Girlfriends.Who.Really.Really.Really.Really.Really.Love.You.S01E12.VOSTFR.1080p.WEB.x264-TsundereRaws-Wawacity.fit",
        "Goblin.Slayer.Goblin.s.Crown.2020.MULTi.1080p.BluRay.DTS.x264-Wawacity.tv.mkv"

    ];

    for filename in filenames.iter() {
        let ep: Episode = Episode::new(filename);

        println!("{} - {}", ep.to_string(), ep.filename_clean.to_string());
    }

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