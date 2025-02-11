use crate::config::Config;
use crate::USER_AGENT;
use anyhow::{Context, Error, Result};
use log::{debug, error, info, warn};
use reqwest::StatusCode;
use secrecy::{ExposeSecret, SecretBox};
use serde::{Deserialize, Serialize};

pub async fn login(config: &Config, credentials: &Credentials) -> Result<ApiToken> {
    info!("Loggin in");
    let url = format!("{}/login", config.api.url);

    // let mut body = HashMap::new();
    // body.insert("username", &credentials.username);
    // body.insert("password", &credentials.password);

    let login = LoginRequest {
        username: &credentials.username,
        password: &credentials.password,
    };

    let req = reqwest::Client::new()
        .post(url)
        // .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", USER_AGENT) // Replace with actual header and value
        .json(&login);

    let response = req.send().await?;

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            let json: Result<LoginResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(login_response) => {
                    debug!("{:?}", login_response);
                    Ok(ApiToken(login_response.token))
                }
                Err(e) => {
                    error!("Failed decoding body {:?} {}", e, text_body);
                    Err(Error::from(e))
                }
            }
        }
        s if s.is_client_error() => {
            let error_response: ErrorResponse = serde_json::from_str(&text_body)?;
            info!("Client error {:?}", error_response);
            if error_response.message.contains("invalid username/password") {
                Err(Error::msg("Invalid username or password"))
            } else {
                Err(Error::msg("Unknown error"))
            }
        }
        s => {
            error!("Server error [{}]: {}", s.as_u16(), text_body);
            Err(Error::msg("Server error"))
        }
    }
}
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct ApiToken(pub SecretBox<String>);

#[derive(Serialize, Debug)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: SecretBox<String>,
    user: User,
}

#[derive(Deserialize, Debug)]
struct User {
    allowed_downloads: i32,
}

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    message: String,
    status: Option<u32>,
}
