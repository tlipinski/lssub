use crate::values::{API_URL, KEY, USER_AGENT};
use anyhow::{Error, Result};
use log::{error, info, trace};
use serde::{Deserialize, Serialize};

pub async fn get_download_link(file_id: i64) -> Result<DownloadResponse> {
    let url = format!("{}/download", API_URL);

    let req = DownloadRequest { file_id };

    let req = reqwest::Client::new()
        .post(url)
        .header("Api-Key", KEY.clone())
        .header("User-Agent", USER_AGENT)
        .json(&req);

    // debug!("{:?}", req);

    let response = req.send().await?;

    // debug!("{:?}", response);

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            let json: Result<DownloadResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(features_response) => {
                    trace!("{}", serde_json::to_string_pretty(&features_response)?);
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

#[derive(Serialize, Debug)]
struct DownloadRequest {
    file_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DownloadResponse {
    pub link: String,
    pub file_name: String,
}
