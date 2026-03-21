use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
use crate::osb::osb_request::osb_request;

pub async fn login(credentials: &Credentials) -> Result<JwtToken> {
    info!("Logging in");

    let url = format!("{}/login", API_URL);

    let login = LoginRequest {
        username: &credentials.username,
        password: &credentials.password,
    };

    let req = reqwest::Client::new()
        .post(url)
        .json(&login);

    osb_request(req).await
}

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Credentials {{ username: {}, password: ******** }}", self.username)
    }
}

#[derive(Debug, Deserialize)]
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
