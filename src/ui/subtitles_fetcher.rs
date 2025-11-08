use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::ResultsUpdate;
use anyhow::{Context, Result, bail};
use log::{debug, error, info};
use osb::features::{FeaturesResponse, features};
use osb::subtitles::{SubtitlesResponse, subtitles};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::join;
use tokio::time::sleep;

pub async fn subtitles_fetch_task(rx: Receiver<String>, tx: Sender<UiEvent>) -> Result<()> {
    'outer: loop {
        sleep(Duration::from_millis(1000)).await;

        let mut last: Option<String> = None;

        // Receive as much as possible within outer loop cycle to reduce OSB calls.
        'debouncing: loop {
            match rx.try_recv() {
                Ok(ev) => {
                    // debug!("Debouncing: {}", ev);
                    last = Some(ev)
                }

                Err(TryRecvError::Empty) => break 'debouncing,

                Err(TryRecvError::Disconnected) => {
                    error!("Disconnected");
                    break 'outer Ok(());
                }
            }
        }

        if let Some(text) = last {
            if text.len() < 3 {
                tx.send(ResultsUpdate(SubtitlesResponse { data: vec![] }))?
            } else {
                // let result = subtitles(&text, vec![String::from("pl")]).await;
                let result = subtitles(&text, vec![]).await;
                match result {
                    Ok(subtitles) => tx.send(ResultsUpdate(subtitles))?,
                    Err(_) => break 'outer Ok(()),
                }
            }
        }
    }
}
