use anyhow::{Error, Result};
use log::{error, info};

pub async fn download(url: String) -> Result<String> {
    let req = reqwest::Client::new().get(url);

    // debug!("{:?}", req);

    let response = req.send().await?;

    // debug!("{:?}", response);

    let status = response.status();

    let text_body = response.text().await?;

    match status {
        s if s.is_success() || s.is_redirection() => Ok(text_body),
        s if s.is_client_error() => {
            info!("Client error {:?}", text_body);
            Err(Error::msg("Error calling OSB"))
        }
        s => {
            error!("Server error [{}]: {}", s.as_u16(), text_body);
            Err(Error::msg("Server error"))
        }
    }
}
