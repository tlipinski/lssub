use crate::ui::ui_messages::UiMessage;
use crate::ui::ui_messages::UiMessage::SubsFetched;
use anyhow::{Context, Result, bail};
use log::{debug, error, info};
use osb::features::{FeaturesResponse, features};
use osb::subtitles::{SubtitlesResponse, subtitles};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::join;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

pub struct SubtitlesQuery {
    pub query: String,
    pub languages: Vec<String>,
}

pub async fn subtitles_fetch_task(
    mut rx: Receiver<SubtitlesQuery>,
    tx: Sender<UiMessage>,
) -> Result<()> {
    'outer: loop {
        sleep(Duration::from_millis(1000)).await;

        let mut last: Option<SubtitlesQuery> = None;

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
            if text.query.len() < 3 {
                tx.send(SubsFetched(SubtitlesResponse { data: vec![] })).await?
            } else {
                // let result = subtitles(&text, vec![String::from("pl")]).await;
                let result = subtitles(&text.query, text.languages).await;
                match result {
                    Ok(subtitles) => tx.send(SubsFetched(subtitles)).await?,
                    Err(_) => break 'outer Ok(()),
                }
            }
        }
    }
}
