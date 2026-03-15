use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub async fn login(credentials: &Credentials) -> Result<JwtToken> {
    info!("Logging in");

    let url = format!("{}/login", API_URL);

    let login = LoginRequest {
        username: &credentials.username,
        password: &credentials.password,
    };

    let req = reqwest::Client::new()
        .post(url)
        .header("Api-Key", AK)
        .header("User-Agent", USER_AGENT)
        .json(&login);

    let response = req.send().await?;

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => {
            // debug!("Response {}", text_body);
            let json: Result<LoginResponse, _> = serde_json::from_str(&text_body);
            match json {
                Ok(login_response) => {
                    debug!("Login response: {login_response:?}");
                    Ok(JwtToken(login_response.token))
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
            Err(Error::msg(error_response))
        }
        s => {
            error!("Server error [{}]: {}", s.as_u16(), text_body);
            Err(Error::msg("Server error"))
        }
    }
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct JwtToken(pub SecretBox<String>);

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
pub(crate) struct ErrorResponse {
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
    pub error: Option<String>,
    pub status: Option<u32>,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.message {
            write!(f, "{}", msg)?;
        } else if let Some(errors) = &self.errors {
            write!(f, "{}", errors.join(", "))?;
        } else if let Some(error) = &self.error {
            write!(f, "{}", error)?;
        }

        if let Some(status) = self.status {
            write!(f, " (status: {})", status)?;
        }

        Ok(())
    }
}
