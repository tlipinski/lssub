use crate::values::{API_URL, KEY, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn features(query: &str) -> Result<FeaturesResponse> {
    let url = format!("{}/features", API_URL);

    let mut params = HashMap::new();
    params.insert("query", query);

    let req = reqwest::Client::new()
        .get(url)
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
            let json: Result<FeaturesResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(features_response) => {
                    debug!("{}", serde_json::to_string_pretty(&features_response)?);
                    Ok(features_response)
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
            Err(Error::msg("Error calling OSB"))
        }
        s => {
            error!("Server error [{}]: {}", s.as_u16(), text_body);
            Err(Error::msg("Server error"))
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeaturesResponse {
    data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Data {
    id: String,
    r#type: String,
    attributes: Attributes
}

#[derive(Deserialize, Serialize, Debug)]
struct Attributes {
    title: String,
    year: String,
    subtitles_count: i32
}