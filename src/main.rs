mod config;
mod login;
mod user_info;

use crate::config::get_config;
use crate::login::{login, ApiToken, Credentials};
use crate::user_info::get_user_info;
use anyhow::Result;
use log::{error, info};
use secrecy::{ExposeSecret, SecretBox};

const USER_AGENT: &str = "subster v0.1.0";

#[tokio::main]
async fn main() {
    env_logger::init();

    match run().await {
        Ok(_) => {}
        Err(e) => {
            error!("Error: {e}")
        }
    };
}

async fn run() -> Result<()> {
    let config = get_config()?;

    let username = std::env::var("USER")?;
    let password = std::env::var("PASS")?;

    let credentials = Credentials { username, password };

    let api_token = login(&config, &credentials).await?;

    let _ = get_user_info(&config, &api_token).await?;

    Ok(())
}
