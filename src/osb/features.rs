use crate::osb::osb_request::osb_request;
use crate::osb::values::{AK, API_URL, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn features(feature_id: i32) -> Result<FeaturesResponse> {
    let url = format!("{}/features", API_URL);

    let mut params = HashMap::new();
    params.insert("id", feature_id);

    let request = reqwest::Client::new().get(url).query(&params);

    osb_request(request).await
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeaturesResponse {
    pub data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Data {
    pub id: String,
    pub r#type: String,
    pub attributes: Attributes,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Attributes {
    pub title: String,
    pub year: String,
    pub subtitles_count: i32,
}
