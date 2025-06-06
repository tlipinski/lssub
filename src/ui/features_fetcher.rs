use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::ResultsUpdate;
use anyhow::{bail, Context, Result};
use log::{debug, error, info};
use osb::features::{features, FeaturesResponse};
use osb::subtitles::{subtitles, SubtitlesResponse};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::join;
use tokio::time::sleep;

pub async fn fetch_features_task(rx: Receiver<String>, tx: Sender<UiEvent>) {
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
                    break 'outer;
                }
            }
        }

        if let Some(text) = last {
            if text.len() < 3 {
                tx.send(ResultsUpdate(SubtitlesResponse { data: vec![] }))
                    .unwrap()
            } else {
                // let result = subtitles(&text, vec![String::from("pl")]).await;
                let result = subtitles(&text, vec![]).await;
                match result {
                    Ok(subtitles) => tx.send(ResultsUpdate(subtitles)).unwrap(),
                    Err(_) => break,
                }
            }
        }
    }
}