use crate::osb::subtitles::{SubtitlesResponse, subtitles, SubtitlesRequest};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, SubsFetched};
use crate::ui::task_runner::TaskRunner;
use anyhow::{Context, Error, Result, bail};
use gio::Task;
use log::{debug, error, info};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use tokio::join;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

pub async fn subtitles_fetch_task(
    mut rx: Receiver<SubtitlesRequest>,
    task_runner: TaskRunner,
) -> Result<()> {
    'outer: loop {
        sleep(Duration::from_millis(1000)).await;

        let mut last: Option<SubtitlesRequest> = None;

        // Receive as much as possible within outer loop cycle to reduce OSB calls.
        'debouncing: loop {
            match rx.try_recv() {
                Ok(ev) => last = Some(ev),

                Err(TryRecvError::Empty) => break 'debouncing,

                Err(TryRecvError::Disconnected) => {
                    error!("Disconnected");
                    break 'outer Ok(());
                }
            }
        }

        if let Some(request) = last {
            task_runner.run(async move {
                if request.query.len() < 3 {
                    Ok(SubsFetched(SubtitlesResponse { data: vec![] }))
                } else {
                    let result = subtitles(request).await;
                    match result {
                        Ok(subtitles) => Ok(SubsFetched(subtitles)),
                        Err(e) => {
                            error!("Error fetching subtitles {e}");
                            Err(Error::msg("Error fetching subtitles list, check logs"))
                        }
                    }
                }
            })
        }
    }
}
