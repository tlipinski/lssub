use crate::osb::osb_request::osb_request;
use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

pub async fn login(credentials: &Credentials) -> Result<JwtToken> {
    info!("Logging in");

    let url = format!("{}/login", API_URL);

    let login = LoginRequest {
        username: &credentials.username,
        password: &credentials.password,
    };

    let request = reqwest::Client::new().post(url).json(&login);

    let response = osb_request::<LoginResponse>(request).await?;
    Ok(JwtToken(response.token))
}

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Credentials {{ username: {}, password: ******** }}",
            self.username
        )
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
