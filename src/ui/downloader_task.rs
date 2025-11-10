use std::sync::mpsc::Receiver;
use log::info;
use osb::download::download;

pub async fn downloader_task(rx: Receiver<i64>) -> anyhow::Result<()>{
    loop {
        match rx.try_recv() {
            Ok(file_id) => {
                info!("Downloading: {file_id}");
                download(file_id).await;
            }
            Err(_) => {
                break Ok(())
            }
        }
    }
}