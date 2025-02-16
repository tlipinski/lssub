use anyhow::Result;
use osb::search::search;

pub async fn handle_search_cmd(title: &str, languages: Vec<&str>) -> Result<()> {
    search(title, languages).await?;
    Ok(())
}