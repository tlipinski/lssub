mod config;
mod user_info;

use crate::config::get_config;
use log::{error, info};
use anyhow::Result;
use crate::user_info::get_user_info;

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

    let r = get_user_info(&config).await?;

    Ok(())
}
