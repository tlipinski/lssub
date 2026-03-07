use crate::values::{API_URL, AK, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use serde::Deserialize;
use std::collections::HashMap;

pub async fn guess(title: &str) -> Result<GuessResponse> {
    info!("Searching for {title}");
    let url = format!("{}/utilities/guessit", API_URL);

    let mut query_params = HashMap::new();
    query_params.insert("filename", title);

    let req = reqwest::Client::new()
        .get(url)
        .header("Api-Key", AK)
        .header("User-Agent", USER_AGENT)
        .query(&query_params);

    let response = req.send().await?;

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            debug!("Response {}", text_body);
            let json: Result<GuessResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(guess_response) => {
                    debug!("{:?}", guess_response);
                    Ok(guess_response)
                }
                Err(e) => {
                    error!("Failed decoding body {:?} {}", e, text_body);
                    Err(Error::from(e))
                }
            }
        }
        s if s.is_client_error() => {
            let error_response: crate::login::ErrorResponse = serde_json::from_str(&text_body)?;
            info!("Client error {:?}", error_response);
            if error_response.message.contains("invalid username/password") {
                Err(Error::msg("Invalid username or password"))
            } else {
                Err(Error::msg("Error calling OSB"))
            }
        }
        s => {
            error!("Server error [{}]: {}", s.as_u16(), text_body);
            Err(Error::msg("Server error"))
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GuessResponse {
    pub title: String,
    pub year: Option<i32>,
    pub language: Option<String>,
    pub subtitle_language: Option<String>,
    pub screen_size: Option<String>,
    pub streaming_service: Option<String>,
    pub other: Option<String>,
    pub audio_codec: Option<String>,
    pub audio_channels: Option<String>,
    pub video_codec: Option<String>,
    pub release_group: Option<String>,
    pub container: Option<String>,
    pub r#type: Option<String>,

    pub episode: Option<i32>,
    pub season: Option<i32>,
}
