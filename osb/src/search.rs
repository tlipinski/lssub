use crate::guess::GuessResponse;
use crate::login::ApiToken;
use crate::values::{API_URL, KEY, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn search(filename: &str, languages: Vec<&str>) -> Result<()> {
    let url = format!("{}/subtitles", API_URL);

    let mut params = HashMap::new();
    params.insert("query", filename);
    let langs = languages.join(",");
    params.insert("languages", langs.as_str());

    let req = reqwest::Client::new()
        .get(url)
        .header("Api-Key", KEY.clone())
        .header("User-Agent", USER_AGENT)
        .query(&params);

    println!("{:?}", req);

    let response = req.send().await?;

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            trace!("Response {}", text_body);
            let json: Result<SearchResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(search_response) => {
                    debug!("{}", serde_json::to_string_pretty(&search_response)?);
                    Ok(())
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
struct SearchResponse {
    total_pages: i32,
    data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Data {
    r#type: String,
    attributes: Attributes,
}

#[derive(Deserialize, Serialize, Debug)]
struct FeatureDetails {
    movie_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Attributes {
    feature_details: FeatureDetails,
    download_count: i32,
    upload_date: String,
    release: String,
}
