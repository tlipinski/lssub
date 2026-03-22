use crate::osb::osb_client::OsbClient;
use crate::osb::values::{AK, API_URL, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn subtitles(osb_client: OsbClient, request: SubtitlesRequest) -> Result<Vec<Subtitle>> {
    let mut params: HashMap<&'static str, String> = HashMap::new();
    let langs = request.languages.join(",");
    params.insert("languages", langs);

    params.insert("ai_translated", request.ai_translated.clone());

    if let Some(i) = request.id {
        params.insert("id", i.to_string());
    } else if let Some(i) = request.parent_id {
        params.insert("parent_feature_id", i.to_string());
        params.insert("order_by", "title".to_string());
    } else {
        params.insert("query", request.query.clone());
    }

    let response: SubtitlesResponse = osb_client
        .call(Method::GET, "/subtitles", |rq| rq.query(&params))
        .await?;

    Ok(response.data)
}

#[derive(Deserialize, Debug)]
pub struct SubtitlesResponse {
    data: Vec<Subtitle>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Subtitle {
    pub id: String,
    pub r#type: String,
    pub attributes: Attributes,
}

impl Subtitle {
    pub fn upload_date(&self) -> String {
        self.attributes
            .upload_date
            .split('T')
            .next()
            .unwrap_or(&self.attributes.upload_date)
            .to_string()
    }

    pub fn downloads(&self) -> i32 {
        self.attributes.download_count + self.attributes.new_download_count
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FeatureDetails {
    pub feature_id: i64,
    pub title: String,
    pub movie_name: String,
    pub year: Option<i32>,
    pub parent_feature_id: Option<i64>,
    pub parent_title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct File {
    pub file_id: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Attributes {
    pub feature_details: FeatureDetails,
    pub language: String,
    pub download_count: i32,
    pub new_download_count: i32,
    pub ai_translated: bool,
    pub votes: i32,
    pub upload_date: String,
    pub release: String,
    pub files: Vec<File>,
}

#[derive(Debug, Clone)]
pub struct SubtitlesRequest {
    pub query: String,
    pub languages: Vec<String>,
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub ai_translated: String,
}
