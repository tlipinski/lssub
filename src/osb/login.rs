use crate::osb::osb_client::OsbClient;
use crate::osb::values::API_URL;
use crate::osb::values::{AK, USER_AGENT};
use anyhow::{Error, Result};
use log::{debug, error, info};
use reqwest::{Client, Method};
use secrecy::SecretBox;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};

pub async fn login(osb_client: OsbClient, credentials: &Credentials) -> Result<JwtToken> {
    info!("Logging in");

    let url = format!("{}/login", API_URL);

    let login = LoginRequest {
        username: &credentials.username,
        password: &credentials.password,
    };

    let response: LoginResponse = osb_client
        .call(Method::POST, "/login", |req| req.json(&login))
        .await?;

    Ok(JwtToken(response.token))
}

#[derive(Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Debug for Credentials {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Credentials {{ username: {}, password: ******** }}",
            self.username
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct JwtToken(pub SecretBox<String>);

#[derive(Serialize, Debug)]
struct LoginRequest<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
struct LoginResponse {
    token: SecretBox<String>,
    user: User,
}

#[derive(Deserialize, Debug)]
struct User {
    allowed_downloads: i32,
}

#[cfg(test)]
mod tests {
    use crate::osb::login::{Credentials, login};
    use crate::osb::osb_client::OsbClient;
    use log::info;
    use reqwest::Method;
    use secrecy::ExposeSecret;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn login_and_parse_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"
                        {
                          "user": {
                            "allowed_translations": 10,
                            "allowed_downloads": 1000,
                            "level": "VIP Member",
                            "user_id": 936829,
                            "ext_installed": false,
                            "vip": true
                          },
                          "token": "123456",
                          "status": 200,
                          "base_url": "vip-api.opensubtitles.com"
                        }
                    "#),
            )
            .mount(&mock_server)
            .await;

        let client = OsbClient::new(&mock_server.uri());

        let credentials = Credentials {
            username: "test_user".into(),
            password: "test_password".into(),
        };

        let response = login(client, &credentials).await.unwrap();

        assert_eq!(response.0.expose_secret(), "123456");
    }
}
