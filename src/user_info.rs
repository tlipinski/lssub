use crate::config::Config;
use anyhow::Result;
use log::{debug, info, log};
use secrecy::ExposeSecret;

pub async fn get_user_info(config: &Config) -> Result<()> {
    let url = format!("{}/infos/user", config.api.url);
    let resp = reqwest::Client::new()
        .get(url)
        .header(
            "Authorization",
            format!("Bearer {}", config.api.token.expose_secret()),
        )
        .header("Api-Key", config.api.key.expose_secret().as_str())
        .header("User-Agent", "subster v0.1.0") // Replace with actual header and value
        .send()
        .await?;
    debug!("{:?}", resp);
    Ok(())
}
