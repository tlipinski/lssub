use crate::config::Config;
use crate::login::ApiToken;
use crate::values::{KEY, USER_AGENT};
use anyhow::Result;
use log::{debug, error};
use secrecy::ExposeSecret;
use serde::Deserialize;
use crate::values::API_URL;

pub async fn get_user_info(config: &Config, token: &ApiToken) -> Result<()> {
    let url = format!("{}/infos/user", API_URL);
    let resp = reqwest::Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", token.0.expose_secret()),
        )
        .header("Api-Key", KEY)
        .header("User-Agent", USER_AGENT) // Replace with actual header and value
        .send()
        .await?;

    let text_body = resp.text().await?;

    let json: Result<UserInfoResponse, _> = serde_json::from_str(&text_body);

    match json {
        Ok(user_info_response) => {
            debug!("{:?}", user_info_response);
        }
        Err(e) => {
            error!("Failed decoding body {:?} {}", e, text_body);
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct UserInfoResponse {
    data: UserData,
}

#[derive(Deserialize, Debug)]
struct UserData {
    username: String,
    allowed_downloads: i32,
    remaining_downloads: i32,
}
