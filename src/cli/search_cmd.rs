use anyhow::Result;
use osb::subtitles::subtitles;
use std::path::Path;

pub async fn handle_search_cmd(file_path: &str, languages: Vec<String>) -> Result<()> {
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

    subtitles(file_name, languages).await?;

    Ok(())
}
