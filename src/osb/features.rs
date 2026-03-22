use crate::osb::osb_client::OsbClient;
use anyhow::Result;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn _features(osb_client: OsbClient, feature_id: i32) -> Result<FeaturesResponse> {
    let mut params = HashMap::new();
    params.insert("id", feature_id);

    let response: FeaturesResponse = osb_client
        .call(Method::GET, "/features", |request| request.query(&params))
        .await?;

    Ok(response)
}

#[derive(Deserialize, Serialize, Debug)]
#[allow(unused)]
pub struct FeaturesResponse {
    pub data: Vec<Data>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Data {
    pub id: String,
    pub r#type: String,
    pub attributes: FeatureAttributes,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FeatureAttributes {
    pub title: String,
    pub year: String,
    pub subtitles_count: i32,
}
