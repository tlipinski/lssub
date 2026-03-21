use crate::osb::subtitles::{SubtitlesRequest, SubtitlesResponse, subtitles};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, FetchSubtitles, RunTask, SubtitlesFetched};
use crate::ui::search_widget::SubtitlesQuery;
use crate::ui::task_runner::{Task, TaskRunner};
use anyhow::{Context, Error, Result, bail};
use log::{debug, error, info};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::join;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

pub async fn debouncer_task(mut rx: Receiver<SubtitlesQuery>, ui_tx: Sender<Action>) -> Result<()> {
    'outer: loop {
        sleep(Duration::from_millis(1000)).await;

        let mut last: Option<SubtitlesQuery> = None;

        // Receive as much as possible within outer loop cycle to reduce OSB calls.
        'debouncing: loop {
            match rx.try_recv() {
                Ok(request) => last = Some(request),

                Err(TryRecvError::Empty) => break 'debouncing,

                Err(TryRecvError::Disconnected) => {
                    error!("Disconnected");
                    break 'outer Ok(());
                }
            }
        }

        if let Some(debounced) = last {
            info!("Debounced {:?}", debounced);
            ui_tx.send(FetchSubtitles).await;
        }
    }
}
