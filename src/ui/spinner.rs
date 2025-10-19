use std::sync::mpsc::Sender;
use tokio::sync::broadcast::Receiver;
use tokio::time::{sleep, Duration};
use crate::ui::events::UiEvent;

pub async fn handle_spinner(tx: Sender<UiEvent>) -> anyhow::Result<()> {
    let spinner = ['|', '/', '-', '\\'];
    let mut pos = 0;
    loop {
        sleep(Duration::from_millis(200)).await;
        pos += 1;
        pos = pos % spinner.len();
        let ch = spinner[pos];
        tx.send(UiEvent::SpinnerUpdate(ch))?
    }
}