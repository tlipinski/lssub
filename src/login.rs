use crate::config::Config;
use anyhow::Result;
use log::debug;
use secrecy::ExposeSecret;
use serde::Deserialize;

pub async fn login(config: &Config, credentials: &Credentials) -> Result<ApiToken> {
    let url = format!("{}/login", config.api.url);

    let req = reqwest::Client::new()
        .post(url)
        .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", "subster v0.1.0") // Replace with actual header and value
        .json(&serde_json::json!({
            "username": credentials.username,
            "password": credentials.password
        }));
    debug!("Request {:?}", req);
    let response = req.send().await?;

    let login_response: LoginResponse = response.json().await?;

    debug!("{:?}", login_response);

    Ok(ApiToken(login_response.token.to_owned()))
}
pub struct Credentials {
    pub username: String,
    pub password: String
}

pub struct ApiToken(String);

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: String,
    user: User,
}

#[derive(Deserialize, Debug)]
struct User {
    allowed_downloads: i32,
}
