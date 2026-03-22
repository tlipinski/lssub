use crate::osb::login::JwtToken;
use crate::osb::osb_client::OsbClient;
use crate::osb::values::API_URL;
use anyhow::Result;
use log::{debug, error, info};
use reqwest::Method;
use secrecy::{ExposeSecret, SecretBox};
use serde::Deserialize;

pub async fn get_user_info(token: &JwtToken) -> Result<User> {
    info!("Getting user info");

    let client = OsbClient::new(API_URL);

    let response: UserInfo = client
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

#[derive(Deserialize, Debug, Default, Clone)]
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
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/infos/user"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"
                        {
                          "data": {
                            "username": "test_user",
                            "downloads_count": 50,
                            "remaining_downloads": 950,
                            "level": "VIP Member",
                            "allowed_translations": 10,
                            "allowed_downloads": 1000
                          }
                        }
                    "#,
        ))
        .mount(&mock_server)
        .await;

    let token = JwtToken(SecretBox::new(Box::new("test_token_123".to_string())));

    let response = get_user_info(&token).await.unwrap();

    assert_eq!(response.username, "test_user");
    assert_eq!(response.downloads_count, 50);
    assert_eq!(response.remaining_downloads, 950);
    assert_eq!(response.level, "VIP Member");
    assert_eq!(response.allowed_translations, 10);
    assert_eq!(response.allowed_downloads, 1000);
}
