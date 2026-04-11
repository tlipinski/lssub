use anyhow::Error;
use log::{error, info};
use reqwest::{Method, RequestBuilder};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::fmt::{Display, Formatter};

pub struct OsbClient {
    base_url: String,
    api_key: String,
    user_agent: String,
}

impl OsbClient {
    pub fn new(base_url: &str, api_key: &str, user_agent: &str) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            user_agent: user_agent.into(),
        }
    }

    pub async fn call<A: DeserializeOwned, F>(
        &self,
        method: Method,
        url: &str,
        build: F,
    ) -> anyhow::Result<A>
    where
        F: Fn(RequestBuilder) -> RequestBuilder,
    {
        let string = format!("{}{}", self.base_url, url);
        info!("Request: {}", string);

        let request = build(reqwest::Client::new().request(method, string));
        self.osb_request::<A>(request).await
    }

    async fn osb_request<A: DeserializeOwned>(&self, request: RequestBuilder) -> anyhow::Result<A> {
        let request = request
            .timeout(std::time::Duration::from_secs(5))
            .header("Api-Key", self.api_key.clone())
            .header("User-Agent", self.user_agent.clone());

        // debug!("Request {:?}", request);

        let http_response = request.send().await?;
        let status = http_response.status();
        let text_body = http_response.text().await?;

        // debug!("Response: {}", text_body);

        match status {
            s if s.is_success() || s.is_redirection() => {
                let body = text_body.clone();
                let json: Result<A, _> = serde_json::from_str(&body);
                match json {
                    Ok(subtitles_response) => {
                        // debug!("{}", serde_json::to_string_pretty(&subtitles_response)?);
                        Ok(subtitles_response)
                    }
                    Err(e) => {
                        error!("Failed decoding body {:?} {}", e, text_body);
                        Err(Error::from(e))
                    }
                }
            }
            s if s.is_client_error() => {
                let error_response: ErrorResponse = serde_json::from_str(&text_body)?;
                info!("Client error [{}]: {:?}", s.as_u16(), error_response);
                Err(Error::msg(error_response))
            }
            s => {
                error!("Server error [{}]: {}", s.as_u16(), text_body);
                Err(Error::msg("Server error, check logs"))
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct ErrorResponse {
    pub message: Option<String>,
    pub errors: Option<Vec<String>>,
    pub error: Option<String>,
    pub status: Option<u32>,
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.message {
            write!(f, "{}", msg)?;
        } else if let Some(errors) = &self.errors {
            write!(f, "{}", errors.join(", "))?;
        } else if let Some(error) = &self.error {
            write!(f, "{}", error)?;
        }

        if let Some(status) = self.status {
            write!(f, " (status: {})", status)?;
        }

        Ok(())
    }
}
