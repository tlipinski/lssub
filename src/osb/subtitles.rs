use crate::osb::osb_request::osb_request;
use crate::osb::values::{AK, API_URL, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn subtitles(request: SubtitlesRequest) -> Result<Vec<Subtitle>> {
    let mut params: HashMap<&'static str, String> = HashMap::new();
    let langs = request.languages.join(",");
    params.insert("languages", langs);

    params.insert("ai_translated", request.ai_translated.to_string());

    if let Some(i) = request.id {
        params.insert("id", i.to_string());
    } else if let Some(i) = request.parent_id {
        params.insert("parent_feature_id", i.to_string());
    } else {
        params.insert("query", request.query.to_string());
    }

    let request = reqwest::Client::new()
        .get(format!("{}/subtitles", API_URL))
        .query(&params);

    let subtitle_responses = osb_request::<SubtitlesResponse>(request).await?;

    Ok(subtitle_responses
        .data
        .iter()
        .map(|resp| Subtitle {
            feature_id: resp.attributes.feature_details.feature_id,
            feature_title: resp.attributes.feature_details.movie_name.clone(),
            parent_feature_id: resp.attributes.feature_details.parent_feature_id,
            parent_feature_title: resp.attributes.feature_details.parent_title.clone(),
            file_id: resp.attributes.files.first().unwrap().file_id,
            title: resp.attributes.release.clone(),
            year: resp
                .attributes
                .feature_details
                .year
                .map(|year| year.to_string())
                .unwrap_or_default(),
            language: resp.attributes.language.clone(),
            upload_date: resp
                .attributes
                .upload_date
                .split('T')
                .next()
                .unwrap_or(&resp.attributes.upload_date)
                .to_string(),
            downloads: (resp.attributes.download_count + resp.attributes.new_download_count),
            ai_translated: match resp.attributes.ai_translated {
                true => "✓".to_string(),
                false => "".to_string(),
            },
            votes: match resp.attributes.votes {
                0 => "".to_string(),
                x => x.to_string(),
            },
        })
        .collect::<Vec<Subtitle>>())
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SubtitlesResponse {
    data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Data {
    id: String,
    r#type: String,
    attributes: Attributes,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureDetails {
    feature_id: i64,
    title: String,
    movie_name: String,
    year: Option<i32>,
    parent_feature_id: Option<i64>,
    parent_title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct File {
    file_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Attributes {
    feature_details: FeatureDetails,
    language: String,
    download_count: i32,
    new_download_count: i32,
    ai_translated: bool,
    votes: i32,
    upload_date: String,
    release: String,
    files: Vec<File>,
}

#[derive(Debug, Clone)]
pub struct SubtitlesRequest {
    pub query: String,
    pub languages: Vec<String>,
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub ai_translated: String,
}

#[derive(Debug, Default, Clone)]
pub struct Subtitle {
    pub feature_id: i64,
    pub feature_title: String,
    pub parent_feature_id: Option<i64>,
    pub parent_feature_title: Option<String>,
    pub file_id: i64,
    pub title: String,
    pub year: String,
    pub language: String,
    pub upload_date: String,
    pub downloads: i32,
    pub ai_translated: String,
    pub votes: String,
}
