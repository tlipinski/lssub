use anyhow::Result;
use osb::features::features;

pub async fn handle_features_cmd(query: &str) -> Result<()> {
    features(query).await?;
    Ok(())
}