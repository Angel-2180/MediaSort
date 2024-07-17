use reqwest::Client;
use serde_json::json;
use std::time::Instant;
use log::{trace, error};

pub async fn send_message(client: &Client, url: &str, message_vec: &Vec<String>) {
  let mut content = message_vec
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