use crate::config::Config;
use crate::USER_AGENT;
use anyhow::Result;
use log::debug;
use secrecy::ExposeSecret;
use serde::Deserialize;
use crate::login::ApiToken;

pub async fn get_user_info(config: &Config, token: &ApiToken) -> Result<()> {
    let url = format!("{}/infos/user", config.api.url);
    let resp = reqwest::Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", token.token.expose_secret()),
        )
        .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", USER_AGENT) // Replace with actual header and value
        .send()
        .await?;

    let user_info_response: UserInfoResponse = resp.json().await?;
    debug!("Body {:?}", user_info_response);

    Ok(())
}

#[derive(Deserialize, Debug)]
struct UserInfoResponse {
    data: UserData
}

#[derive(Deserialize, Debug)]
struct UserData {
    username: String,
    allowed_downloads: i32,
    remaining_downloads: i32,
}
