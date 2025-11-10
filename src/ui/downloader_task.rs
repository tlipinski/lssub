use log::info;
use osb::download::get_download_link;
use std::sync::mpsc::Receiver;
use osb::download_link::download;
use std::fs;

pub async fn downloader_task(rx: Receiver<i64>) -> anyhow::Result<()> {
    loop {
        match rx.recv() {
            Ok(file_id) => {
                info!("Downloading: {file_id}");
                let download_link_response = get_download_link(file_id).await?;
                let content = download(download_link_response.link).await?;
                fs::write(format!("{}.srt", file_id), content)?;
            }
            Err(err) => {
                info!("Error: {err}");
                break Ok(());
            }
        }
    }
}
