use crate::osb::login::JwtToken;
use crate::osb::osb_request::osb_request;
use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::Result;
use log::{debug, error};
use secrecy::ExposeSecret;
use serde::Deserialize;

pub async fn get_languages() -> Result<Vec<Language>> {
    let request = reqwest::Client::new().get(format!("{}/infos/languages", API_URL));

    let languages: LanguagesResponse = osb_request(request).await?;

    Ok(languages.data)
}

#[derive(Deserialize)]
struct LanguagesResponse {
    data: Vec<Language>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Language {
    pub language_name: String,
    pub language_code: String,
}
