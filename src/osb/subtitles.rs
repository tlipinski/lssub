use crate::osb::osb_request::osb_request;
use crate::osb::values::{AK, API_URL, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn subtitles(request: SubtitlesRequest) -> Result<SubtitlesResponse> {
    let mut params: HashMap<&'static str, String> = HashMap::new();
    params.insert("query", request.query.to_string());
    let langs = request.languages.join(",");
    params.insert("languages", langs);

    params.insert("ai_translated", request.ai_translated.to_string());

    if let Some(i) = request.id {
        params.insert("id", i.to_string());
    }

    let request = reqwest::Client::new()
        .get(format!("{}/subtitles", API_URL))
        .query(&params);

    osb_request(request).await
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubtitlesResponse {
    pub data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Data {
    pub id: String,
    pub r#type: String,
    pub attributes: Attributes,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureDetails {
    pub feature_id: i64,
    pub movie_name: String,
    pub year: Option<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct File {
    pub file_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
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
    pub ai_translated: String,
}
