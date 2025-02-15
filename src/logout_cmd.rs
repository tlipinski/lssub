use crate::secret::clear;
use anyhow::Result;

pub async fn handle_logout_cmd() -> Result<()> {
    let result = clear().await;

    println!("Logged out successfully");

    result
}
