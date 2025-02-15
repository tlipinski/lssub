use crate::values::{KEY, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};
use crate::values::API_URL;

pub async fn login(credentials: &Credentials) -> Result<ApiToken> {
    info!("Loggin in");
    let url = format!("{}/login", API_URL);

    // let mut body = HashMap::new();
    // body.insert("username", &credentials.username);
    // body.insert("password", &credentials.password);

    let login = LoginRequest {
        username: &credentials.username,
        password: &credentials.password,
    };

    let req = reqwest::Client::new()
        .post(url)
        .header("Api-Key", KEY.clone())
        .header("User-Agent", USER_AGENT)
        .json(&login);

    let response = req.send().await?;

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            debug!("Response {}", text_body);
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
                Err(Error::msg("Error calling OSB"))
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
