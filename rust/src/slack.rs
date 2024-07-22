use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::error::Error;
use dotenv::dotenv;

#[derive(Serialize)]
struct SlackMessage {
    channel: String,
    text: String,
}

#[derive(Deserialize)]
struct SlackResponse {
    ok: bool,
    #[serde(default)]
    upload_url: String,
    #[serde(default)]
    file_id: String,
}

pub fn send_slack_message(message: &str) -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let client = Client::new();
    let url = "https://slack.com/api/chat.postMessage";
    let token = std::env::var("TOKEN")?;
    let channel = std::env::var("CHANNEL")?;
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);
    headers.insert(CONTENT_TYPE, "application/json".parse()?);
    let data = SlackMessage {
        channel,
        text: message.to_string(),
    };
    let response = client.post(url)
        .headers(headers)
        .body(serde_json::to_string(&data).unwrap())
        .send()?;
    if response.status().is_success() {
        println!("Message sent successfully message: {}", message);
    } else {
        println!("Error: {}", response.status());
        let error_text = response.text()?;
        println!("{}", error_text);
    }
    Ok(())
}

pub fn upload_image_to_slack(token: &str, image: &[u8], filename: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let url = "https://slack.com/api/files.getUploadURLExternal";
    let params = [("filename", filename), ("length", &image.len().to_string())];
    
    let response_text = client.get(url)
        .bearer_auth(token)
        .query(&params)
        .send()?
        .text()?;

    let response: SlackResponse = serde_json::from_str(&response_text)?;

    if !response.ok {
        return Err("Failed to get upload URL".into());
    }

    let upload_url = response.upload_url;
    let file_id = response.file_id;

    client.post(upload_url)
        .header(CONTENT_TYPE, "application/octet-stream")
        .body(image.to_vec())
        .send()?;

    Ok(file_id)
}

pub fn send_single_image_to_slack(image: &[u8], filename: &str, title: &str) -> Result<(), Box<dyn Error>> {
    let token = std::env::var("TOKEN")?;
    let channel_id = std::env::var("CHANNEL_ID")?;
    let file_id = upload_image_to_slack(token.as_str(), image, filename)?;

    let client = Client::new();
    let url = "https://slack.com/api/files.completeUploadExternal";
    
    let data = serde_json::json!({
        "files": [{
            "id": file_id,
            "title": title
        }],
        "channel_id": channel_id
    });

    let response_text = client.post(url)
        .bearer_auth(token)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&data)?)
        .send()?
        .text()?;

    let response: SlackResponse = serde_json::from_str(&response_text)?;

    if response.ok {
        println!("Image sent successfully");
        Ok(())
    } else {
        Err("Failed to send image".into())
    }
}

pub fn send_multiple_images_to_slack(images: &[(&[u8], &str, &str)]) -> Result<(), Box<dyn Error>> {
    let token = std::env::var("TOKEN")?;
    let channel_id = std::env::var("CHANNEL_ID")?;
    let client = Client::new();
    let mut files = Vec::new();

    for (image, filename, title) in images {
        match upload_image_to_slack(token.as_str(), image, filename) {
            Ok(file_id) => files.push(serde_json::json!({
                "id": file_id,
                "title": title
            })),
            Err(e) => println!("Error uploading image {}: {}", filename, e),
        }
    }

    if files.is_empty() {
        return Err("No images were successfully prepared for upload".into());
    }

    let url = "https://slack.com/api/files.completeUploadExternal";
    let data = serde_json::json!({
        "files": files,
        "channel_id": channel_id
    });

    let response_text = client.post(url)
        .bearer_auth(token)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&data)?)
        .send()?
        .text()?;

    let response: SlackResponse = serde_json::from_str(&response_text)?;

    if response.ok {
        println!("Images sent successfully");
        Ok(())
    } else {
        Err("Failed to send images".into())
    }
}