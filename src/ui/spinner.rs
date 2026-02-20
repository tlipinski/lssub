use crate::ui::ui_messages::UiMessage;
use std::sync::mpsc::Sender;
use tokio::sync::broadcast::Receiver;
use tokio::time::{Duration, sleep};

pub async fn handle_spinner_task(tx: Sender<UiMessage>) -> anyhow::Result<()> {
    let spinner = ['|', '/', '-', '\\'];
    let mut pos = 0;
    loop {
        sleep(Duration::from_millis(200)).await;
        pos += 1;
        pos %= spinner.len();
        let ch = spinner[pos];
        tx.send(UiMessage::SpinnerUpdate(ch))?
    }
}
