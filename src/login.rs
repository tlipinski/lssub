use crate::config::Config;
use anyhow::Result;
use log::debug;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;
use crate::USER_AGENT;

pub async fn login(config: &Config, credentials: &Credentials) -> Result<ApiToken> {
    let url = format!("{}/login", config.api.url);

    // let mut body = HashMap::new();
    // body.insert("username", &credentials.username);
    // body.insert("password", &credentials.password);

    let req = reqwest::Client::new()
        .post(url)
        .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", USER_AGENT) // Replace with actual header and value
        .json(&serde_json::json!({
            "username": credentials.username,
            "password": credentials.password
        }));
    debug!("Request {:?}", req);
    let response = req.send().await?;

    let login_response: LoginResponse = response.json().await?;

    debug!("{:?}", login_response);

    Ok(ApiToken(login_response.token))
}
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub struct ApiToken(SecretBox<String>);

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: SecretBox<String>,
    user: User,
}

#[derive(Deserialize, Debug)]
struct User {
    allowed_downloads: i32,
}
