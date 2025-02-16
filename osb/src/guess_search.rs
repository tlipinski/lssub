use crate::guess::GuessResponse;
use crate::login::ApiToken;
use crate::values::{API_URL, KEY, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use serde::Deserialize;
use std::collections::HashMap;

pub async fn guess_search(guess_response: GuessResponse) -> Result<()> {
    let url = format!("{}/subtitles", API_URL);

    let mut params = HashMap::new();
    params.insert("query", guess_response.title);
    insert_if_some(
        &mut params,
        guess_response.episode.map(|v| v.to_string()),
        "episode_number",
    );
    insert_if_some(
        &mut params,
        guess_response.season.map(|v| v.to_string()),
        "season_number",
    );
    insert_if_some(&mut params, Some("pl".to_string()), "languages");

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
            debug!("Response {}", text_body);
            let json: Result<GuessResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(guess_response) => {
                    debug!("{:?}", guess_response);
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

// a lifetime - same for hash map keys and for name
// both live only inside search function
fn insert_if_some<'a>(params: &mut HashMap<&'a str, String>, value: Option<String>, name: &'a str) {
    match value {
        None => {}
        Some(v) => {
            params.insert(&name, v);
        }
    }
}
