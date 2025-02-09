use crate::config::Config;
use anyhow::Result;
use log::debug;
use secrecy::ExposeSecret;
use crate::USER_AGENT;

pub async fn get_user_info(config: &Config) -> Result<()> {
    let url = format!("{}/infos/user", config.api.url);
    let resp = reqwest::Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", config.api.token.expose_secret()),
        )
        .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", USER_AGENT) // Replace with actual header and value
        .send()
        .await?;

    let body = pretty(&resp.text().await?)?;
    debug!("Body {}", body);

    Ok(())
}

pub fn pretty(s: &str) -> Result<String> {
    let json = serde_json::from_str::<serde_json::Value>(s)?;
    Ok(serde_json::to_string_pretty(&json)?)
}
