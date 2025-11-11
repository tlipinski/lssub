use log::info;
use osb::download::download;
use osb::get_download_link::get_download_link;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Receiver;

pub async fn downloader_task(
    rx: Receiver<SubsDownload>,
    base_path: PathBuf,
    file_name: Option<String>,
) -> anyhow::Result<()> {
    loop {
        match rx.recv() {
            Ok(subs_download) => {
                info!("Downloading: {subs_download:?}");

                let download_link_response = get_download_link(subs_download.file_id).await?;

                let content = download(download_link_response.link).await?;

                let file_base = file_name.as_deref().unwrap_or(download_link_response.file_name.as_str());
                let file = Path::new(file_base).with_extension("srt");

                fs::write(base_path.join(file), content)?;
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
}
