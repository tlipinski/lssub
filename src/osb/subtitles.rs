use crate::osb::osb_client::OsbClient;
use anyhow::{Error, Result};
use log::{debug, error, info, trace};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub async fn subtitles(osb_client: OsbClient, request: SubtitlesRequest) -> Result<Vec<Subtitle>> {
    info!("Subtitles request: {:?}", request);

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

#[derive(Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FeatureDetails {
    pub feature_id: i64,
    pub title: String,
    pub movie_name: String,
    pub year: Option<i32>,
    pub parent_feature_id: Option<i64>,
    pub parent_title: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct File {
    pub file_id: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::osb::osb_client::OsbClient;
    use crate::osb::subtitles::{
        Attributes, FeatureDetails, File, Subtitle, SubtitlesRequest, subtitles,
    };
    use env_logger::{Builder, Target};
    use log::{LevelFilter, info};
    use reqwest::Method;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_subs() {
        Builder::new()
            .target(Target::Stdout)
            .filter_level(LevelFilter::Debug)
            .init();

        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/subtitles"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                r#"
                    {
                      "total_pages": 1,
                      "total_count": 34,
                      "per_page": 50,
                      "page": 1,
                      "data": [
                        {
                          "id": "8612377",
                          "type": "subtitle",
                          "attributes": {
                            "subtitle_id": "8612377",
                            "language": "en",
                            "download_count": 250,
                            "new_download_count": 491,
                            "hearing_impaired": false,
                            "hd": true,
                            "fps": 0.0,
                            "votes": 0,
                            "ratings": 0.0,
                            "from_trusted": false,
                            "foreign_parts_only": false,
                            "upload_date": "2024-07-11T06:25:49Z",
                            "ai_translated": false,
                            "nb_cd": 1,
                            "slug": "slug",
                            "machine_translated": false,
                            "release": "release",
                            "comments": "comments",
                            "legacy_subtitle_id": 11331718,
                            "legacy_uploader_id": 10030650,
                            "uploader": {
                              "uploader_id": null,
                              "name": "Anonymous",
                              "rank": "anonymous"
                            },
                            "feature_details": {
                              "feature_id": 1,
                              "feature_type": "Movie",
                              "year": 1995,
                              "title": "title",
                              "movie_name": "1995 - Title",
                              "imdb_id": 112682,
                              "tmdb_id": 902
                            },
                            "url": "https://www.opensubtitles.com/en/subtitles/title",
                            "related_links": [
                              {
                                "label": "related-links-label",
                                "url": "https://www.opensubtitles.com/en/movies/title",
                                "img_url": "https://s7.opensubtitles.com/features/1/0/0/0.jpg"
                              }
                            ],
                            "files": [
                              {
                                "file_id": 9535264,
                                "cd_number": 1,
                                "file_name": "Movie.en"
                              }
                            ]
                          }
                        }
                    ]
                }"#,
            ))
            .mount(&mock_server)
            .await;

        let client = OsbClient::new(&mock_server.uri(), "", "");

        let request = SubtitlesRequest {
            query: "".to_string(),
            languages: vec![],
            id: None,
            parent_id: None,
            ai_translated: "".to_string(),
        };

        let response = subtitles(client, request).await.unwrap();

        assert_eq!(response.len(), 1);

        assert_eq!(
            response.first().unwrap(),
            &Subtitle {
                id: "8612377".to_string(),
                r#type: "subtitle".to_string(),
                attributes: Attributes {
                    feature_details: FeatureDetails {
                        feature_id: 1,
                        title: "title".to_string(),
                        movie_name: "1995 - Title".to_string(),
                        year: Some(1995),
                        parent_feature_id: None,
                        parent_title: None,
                    },
                    language: "en".to_string(),
                    download_count: 250,
                    new_download_count: 491,
                    ai_translated: false,
                    votes: 0,
                    upload_date: "2024-07-11T06:25:49Z".to_string(),
                    release: "release".to_string(),
                    files: vec![File { file_id: 9535264 }],
                }
            }
        );

        assert_eq!(response.first().unwrap().downloads(), 741);
        assert_eq!(response.first().unwrap().upload_date(), "2024-07-11");
    }

}
