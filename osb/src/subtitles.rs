use crate::values::{API_URL, KEY, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn subtitles(filename: &str, languages: Vec<String>) -> Result<SubtitlesResponse> {
    let url = format!("{}/subtitles", API_URL);

    let mut params = HashMap::new();
    params.insert("query", filename);
    let langs = languages.join(",");
    params.insert("languages", langs.as_str());

    let req = reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(5))
        .header("Api-Key", KEY.clone())
        .header("User-Agent", USER_AGENT)
        .query(&params);

    debug!("{:?}", req);

    let response = req.send().await?;

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            trace!("Response {}", text_body);
            let json: Result<SubtitlesResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(subtitles_response) => {
                    debug!("{}", serde_json::to_string_pretty(&subtitles_response)?);
                    Ok(subtitles_response)
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

#[derive(Deserialize, Serialize, Debug)]
pub struct SubtitlesResponse {
    pub data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Data {
    pub r#type: String,
    pub attributes: Attributes,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureDetails {
    pub movie_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Attributes {
    pub feature_details: FeatureDetails,
    pub language: String,
    pub download_count: i32,
    pub upload_date: String,
    pub release: String,
}
