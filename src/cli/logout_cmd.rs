use crate::secret::clear;
use anyhow::Result;
use log::info;

pub async fn handle_logout_cmd() -> Result<()> {
    let result = clear().await;

    info!("Logged out successfully");

    result
}
