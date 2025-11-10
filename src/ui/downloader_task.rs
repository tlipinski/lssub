use log::info;
use osb::download::download;
use std::sync::mpsc::Receiver;

pub async fn downloader_task(rx: Receiver<i64>) -> anyhow::Result<()> {
    loop {
        match rx.recv() {
            Ok(file_id) => {
                info!("Downloading: {file_id}");
                download(file_id).await;
            }
            Err(err) => {
                info!("Error: {err}");
                break Ok(());
            }
        }
    }
}
