use log::info;
use osb::get_download_link::get_download_link;
use std::sync::mpsc::Receiver;
use osb::download::download;
use std::fs;

pub async fn downloader_task(rx: Receiver<SubsDownload>) -> anyhow::Result<()> {
    loop {
        match rx.recv() {
            Ok(subs_download) => {
                info!("Downloading: {subs_download:?}");
                let download_link_response = get_download_link(subs_download.file_id).await?;
                let content = download(download_link_response.link).await?;
                fs::write(format!("{}.srt", subs_download.save_path), content)?;
            }
            Err(err) => {
                info!("Error: {err}");
                break Ok(());
            }
        }
    }
}

#[derive(Debug)]
pub struct SubsDownload {
    pub file_id: i64,
    pub save_path: String
}