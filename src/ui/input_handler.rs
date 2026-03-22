use crate::ui::actions::Action;
use crate::ui::actions::Action::{InputReceived, Tick};
use anyhow::Result;
use futures_util::{FutureExt, StreamExt};
use ratatui::crossterm::event::EventStream;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::interval;

pub async fn handle_input_task(tx: Sender<Action>) -> Result<()> {
    let mut tick_interval = interval(Duration::from_secs_f64(1.0 / 4.0));
    let mut event_stream = EventStream::new();

    loop {
        let action = tokio::select! {
            maybe_event = event_stream.next().fuse() => match maybe_event {
                Some(Ok(event)) => InputReceived(event),
                Some(Err(err)) => return Err(err.into()),
                None => break Ok(()),
            },
            _ = tick_interval.tick() => Tick,
        };

        tx.send(action).await?;
    }
}
