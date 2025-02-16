use anyhow::Result;
use osb::guess::guess;

pub async fn handle_search_cmd(title: &str) -> Result<()> {
    let guess_response = guess(title).await;

    Ok(())
}