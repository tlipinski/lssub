use crate::osb::login::JwtToken;
use crate::osb::osb_client::OsbClient;
use crate::osb::values::{AK, API_URL, USER_AGENT, VIP_API_URL};
use anyhow::{Error, Result};
use log::{error, info, trace};
use reqwest::Method;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

pub async fn get_download_link(
    osb_client: OsbClient,
    token_opt: Option<JwtToken>,
    file_id: i64,
) -> Result<DownloadLinkResponse> {
    let response: DownloadLinkResponse = if let Some(token) = token_opt {
        osb_client.call(Method::POST, "/download", |rq| {
            rq.bearer_auth(token.0.expose_secret())
                .json(&DownloadRequest { file_id })
        }).await?
    } else {
        osb_client.call(Method::POST, "/download", |rq| {
            rq.json(&DownloadRequest { file_id })
        }).await?
    };

    Ok(response)
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
