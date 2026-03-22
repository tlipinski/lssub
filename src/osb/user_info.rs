use crate::osb::login::JwtToken;
use crate::osb::osb_client::OsbClient;
use crate::osb::osb_request::osb_request;
use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::Result;
use log::{debug, error, info};
use reqwest::Method;
use secrecy::ExposeSecret;
use serde::Deserialize;

pub async fn get_user_info(token: &JwtToken) -> Result<User> {
    info!("Getting user info");

    let client = OsbClient::new(API_URL);

    let response: UserInfo = client
        .call(Method::GET, "/infos/user", |rq| {
            rq.bearer_auth(token.0.expose_secret())
        })
        .await?;

    Ok(response.data)
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct UserInfo {
    pub data: User,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct User {
    pub username: String,
    pub downloads_count: i32,
    pub remaining_downloads: i32,
    pub level: String,
    pub allowed_translations: i32,
    pub allowed_downloads: i32,
}
