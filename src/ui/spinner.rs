use std::sync::mpsc::Sender;
use tokio::sync::broadcast::Receiver;
use tokio::time::{sleep, Duration};
use crate::ui::events::UiEvent;

pub async fn handle_spinner(tx: Sender<UiEvent>) -> anyhow::Result<()> {
    let spinner = "|/-\\";
    let mut pos = 0;
    loop {
        sleep(Duration::from_millis(200)).await;
        pos += 1;
        if pos == 4 {
            pos = 0;
        }
        let ch = spinner.chars().nth(pos).unwrap_or('|');
        tx.send(UiEvent::SpinnerUpdate(ch))?
    }
}