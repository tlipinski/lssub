use crate::osb::login::JwtToken;
use crate::osb::osb_request::osb_request;
use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::Result;
use log::{debug, error};
use secrecy::ExposeSecret;
use serde::Deserialize;

pub async fn get_user_info(token: &JwtToken) -> Result<User> {
    let request = reqwest::Client::new()
        .get(format!("{}/infos/user", API_URL))
        .bearer_auth(token.0.expose_secret());

    let user_info: UserInfo = osb_request(request).await?;

    Ok(user_info.data)
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
