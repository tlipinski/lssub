use crate::config::ConfigProvider;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{Exit, FetchSubtitles, Init, Tick};
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use crate::ui::debouncer::debouncer_task;
use crate::ui::main_widget::MainWidget;
use crate::ui::spinner::{Spinner, spinner_task};
use crate::ui::subs_list_widget::QueryParams;
use crate::ui::task_runner::TaskRunner;
use crossterm::event::EventStream;
use futures_util::FutureExt;
use futures_util::StreamExt;
use log::{error, info};
use ratatui::DefaultTerminal;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::interval;

pub struct App {
    ui_tx: Sender<Action>,
    ui_rx: Receiver<Action>,
    debouncer_rx: Receiver<()>,
    spinner: Arc<RwLock<Spinner>>,
    main_widget: Box<dyn Component>,
    file_name: Option<String>,
}

impl App {
    pub fn new(base_path: &Path, file_name: Option<&str>) -> Self {
        let (ui_tx, ui_rx) = tokio::sync::mpsc::channel::<Action>(100);
        let (debouncer_tx, debouncer_rx) = tokio::sync::mpsc::channel::<()>(100);

        let spinner = Arc::new(RwLock::new(Spinner { c: ' ' }));

        let config_provider = ConfigProvider::default();

        let main_widget = MainWidget::new(
            base_path,
            file_name,
            debouncer_tx,
            TaskRunner::new(ui_tx.clone()),
            config_provider,
            spinner.clone(),
        );

        App {
            ui_tx,
            ui_rx,
            debouncer_rx,
            spinner: spinner.clone(),
            main_widget: Box::new(main_widget),
            file_name: file_name.map(String::from),
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        // tokio::spawn(handle_input_task(self.ui_tx.clone()));
        tokio::spawn(spinner_task(self.spinner.clone()));

        let ui_tx = self.ui_tx.clone();
        tokio::spawn(debouncer_task(
            self.debouncer_rx,
            Duration::from_millis(1000),
            async move || {
                ui_tx
                    .send(FetchSubtitles)
                    .await
                    .expect("Sending to channel failed");
            },
        ));

        let mut event_stream = EventStream::new();
        let mut tick_interval = interval(Duration::from_secs_f64(1.0 / 4.0));

        let mut app_state = AppState {
            query: self.file_name.unwrap_or_default(),
            params: QueryParams::default(),
            languages: vec![],
        };
        let mut message_opt = Some(Init);

        'main_loop: loop {
            while let Some(msg) = message_opt {
                match msg {
                    Exit => {
                        info!("Exiting application");
                        break 'main_loop;
                    }
                    _ => {
                        if let Some((new_msg, next_state)) =
                            self.main_widget.update(&msg, app_state.clone())
                        {
                            message_opt = Some(new_msg);
                            app_state = next_state;
                        } else {
                            message_opt = None;
                        }
                    }
                }
            }

            terminal.draw(|frame| self.main_widget.render(frame, frame.area()))?;

            let (msg, next_state) = tokio::select! {
                maybe_event = event_stream.next().fuse() => match maybe_event {
                    Some(Ok(event)) => {
                        if let Some((action, new_state)) = self.main_widget.handle_key_event(&event, app_state.clone()) {
                            (Some(action), new_state)
                        } else {
                            (None, app_state)
                        }
                    }
                    Some(Err(err)) => {
                        error!("{err}");
                        return Err(err.into())
                    },
                    None => (None, app_state),
                },
                ui = self.ui_rx.recv().fuse() => (ui, app_state),
                _ = tick_interval.tick() => (Some(Tick), app_state),
            };
            message_opt = msg;
            app_state = next_state;
        }

        Ok(())
    }
}
