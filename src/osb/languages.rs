use crate::osb::osb_client::OsbClient;
use anyhow::Result;
use reqwest::Method;
use serde::Deserialize;

pub async fn get_languages(osb_client: OsbClient) -> Result<Vec<Language>> {
    let response: LanguagesResponse = osb_client
        .call(Method::GET, "/infos/languages", |rq| rq)
        .await?;

    Ok(response.data)
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
