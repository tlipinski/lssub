use crate::osb::login::JwtToken;
use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::Result;
use log::{debug, error};
use secrecy::ExposeSecret;
use serde::Deserialize;
use crate::osb::osb_request::osb_request;

pub async fn get_user_info(token: &JwtToken) -> Result<UserInfo> {
    let request = reqwest::Client::new()
        .get(format!("{}/infos/user", API_URL))
        .bearer_auth(token.0.expose_secret());

    osb_request(request).await
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct UserInfo {
    pub data: UserData,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct UserData {
    pub username: String,
    pub downloads_count: i32,
    pub remaining_downloads: i32,
    pub level: String,
    pub allowed_translations: i32,
    pub allowed_downloads: i32,
}
