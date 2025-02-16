use std::collections::HashMap;
use anyhow::{Error, Result};
use log::{debug, error, info};
use serde::Deserialize;
use crate::login::ApiToken;
use crate::values::{API_URL, KEY, USER_AGENT};

pub async fn guess(title: &str) -> Result<GuessResponse> {
  info!("Searching for {title}");
  let url = format!("{}/utilities/guessit", API_URL);

  let mut query_params = HashMap::new();
  query_params.insert("filename", title);

  let req = reqwest::Client::new()
      .post(url)
      .header("Api-Key", KEY.clone())
      .header("User-Agent", USER_AGENT)
      .query(&query_params);

  println!("{:?}", req);

  let response = req.send().await?;

  let status = response.status();

  println!("{}", status);

  let text_body = response.text().await?;

  println!("{}", text_body);

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
  title: String,
  container: String,
  episode: i32,
}
