use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use log::{error, trace};

pub async fn send_message(client: &Client, url: &str, message_vec: &mut Vec<String>) {
    message_vec.sort();
    let content = message_vec
    .iter()
    .map(|name| {
        let parts: Vec<&str> = name.split(" - ").collect();
        let media_name = parts[0];
        let episode_info = parts.get(1).unwrap_or(&"");
        format!("Added: *{}* - **{}** to the library!", media_name, episode_info)
    })
    .collect::<Vec<String>>();

    for chunk in content.chunks(10) {
        let message = chunk.join("\n");
        let message_payload = json!({
            "content": message,
            "username": "Media Bot"
        });
        send_discord_message(&client, &url, &message_payload).await;
    }


}

async fn send_discord_message(client: &Client, url: &str, payload: &serde_json::Value) {
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