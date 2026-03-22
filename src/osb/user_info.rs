use crate::osb::login::JwtToken;
use crate::osb::osb_client::OsbClient;
use anyhow::Result;
use log::info;
use reqwest::Method;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;

pub async fn get_user_info(osb_client: OsbClient, token: &JwtToken) -> Result<User> {
    info!("Getting user info");

    let response: UserInfo = osb_client
        .call(Method::GET, "/infos/user", |rq| {
            rq.bearer_auth(token.0.expose_secret())
        })
        .await?;

    Ok(response.data)
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct UserInfo {
    pub data: User,
}

#[derive(Deserialize, Debug, Default, Clone, PartialEq)]
pub struct User {
    pub username: String,
    pub downloads_count: i32,
    pub remaining_downloads: i32,
    pub level: String,
    pub allowed_translations: i32,
    pub allowed_downloads: i32,
}

#[tokio::test]
async fn call_user_info_endpoint_and_parse_response() {
    use env_logger::{Builder, Target};
    use log::LevelFilter;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    Builder::new()
        .target(Target::Stdout)
        .filter_level(LevelFilter::Debug)
        .try_init();

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/infos/user"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
                        {
                          "data": {
                            "allowed_translations": 10,
                            "allowed_downloads": 1000,
                            "level": "VIP Member",
                            "user_id": 936829,
                            "ext_installed": false,
                            "vip": true,
                            "reset_time_utc": "2026-03-22T23:59:58.000Z",
                            "reset_time": "13 hours and 13 minutes",
                            "downloads_count": 1,
                            "remaining_downloads": 999,
                            "username": "test_user"
                          }
                        }
                    "#,
        ))
        .mount(&mock_server)
        .await;

    let token = JwtToken(SecretBox::new(Box::new("test_token_123".to_string())));

    let client = OsbClient::new(&mock_server.uri(), "", "");

    let response = get_user_info(client, &token).await.unwrap();

    assert_eq!(
        response,
        User {
            username: "test_user".to_string(),
            level: "VIP Member".to_string(),
            downloads_count: 1,
            remaining_downloads: 999,
            allowed_translations: 10,
            allowed_downloads: 1000,
        }
    );
}
