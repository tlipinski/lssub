use crate::config::Config;
use anyhow::Result;
use log::debug;
use secrecy::ExposeSecret;

pub async fn login(config: &Config, username: &str, password: &str) -> Result<String> {
    let url = format!("{}/login", config.api.url);

    let req = reqwest::Client::new()
        .post(url)
        .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", "subster v0.1.0") // Replace with actual header and value
        .json(&serde_json::json!({
                "username": username,
                "password": password
            }));
    debug!("Request {:?}", req);

    let response = req.send().await?;
    let body = crate::user_info::pretty(&response.text().await?)?;
    debug!("Body {}", body);

    Ok("token".to_owned())
}
