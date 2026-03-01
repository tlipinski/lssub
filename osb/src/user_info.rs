use crate::login::JwtToken;
use crate::values::API_URL;
use crate::values::{KEY, USER_AGENT};
use anyhow::Result;
use log::{debug, error};
use secrecy::ExposeSecret;
use serde::Deserialize;

pub async fn get_user_info(token: &JwtToken) -> Result<UserInfo> {
    let url = format!("{}/infos/user", API_URL);
    let resp = reqwest::Client::new()
        .get(url)
        .bearer_auth(token.0.expose_secret())
        .header("Api-Key", KEY.clone())
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;

    let text_body = resp.text().await?;

    let json: Result<UserInfo, _> = serde_json::from_str(&text_body);

    match json {
        Ok(user_info_response) => {
            debug!("{:?}", user_info_response);
            Ok(user_info_response)
        }
        Err(e) => {
            error!("Failed decoding body {:?} {}", e, text_body);
            Err(e.into())
        }
    }

}

#[derive(Deserialize, Debug)]
pub struct UserInfo {
    data: UserData,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    username: String,
    allowed_downloads: i32,
    remaining_downloads: i32,
}
