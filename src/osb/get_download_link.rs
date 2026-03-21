use crate::osb::login::JwtToken;
use crate::osb::osb_request::osb_request;
use crate::osb::values::{AK, API_URL, USER_AGENT, VIP_API_URL};
use anyhow::{Error, Result};
use log::{error, info, trace};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

pub async fn get_download_link(
    token_opt: Option<JwtToken>,
    file_id: i64,
) -> Result<DownloadLinkResponse> {
    let url = if token_opt.is_some() {
        format!("{}/download", VIP_API_URL)
    } else {
        format!("{}/download", API_URL)
    };

    let request = if let Some(token) = token_opt {
        reqwest::Client::new()
            .post(url)
            .bearer_auth(token.0.expose_secret())
            .json(&DownloadRequest { file_id })
    } else {
        reqwest::Client::new()
            .post(url)
            .json(&DownloadRequest { file_id })
    };

    osb_request(request).await
}

#[derive(Serialize, Debug)]
struct DownloadRequest {
    file_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DownloadLinkResponse {
    pub link: String,
    pub file_name: String,
    pub requests: i32,
    pub remaining: i32,
}
