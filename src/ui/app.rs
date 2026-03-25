use crate::config::ConfigProvider;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{Exit, FetchSubtitles, Init};
use crate::ui::main_widget::{MainWidget, Screen};
use crate::ui::component::Component;
use crate::ui::debouncer::debouncer_task;
use crate::ui::input_handler::handle_input_task;
use crate::ui::spinner::{spinner_task, Spinner};
use crate::ui::task_runner::TaskRunner;
use log::info;
use ratatui::DefaultTerminal;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};

pub struct App {
    ui_tx: Sender<Action>,
    ui_rx: Receiver<Action>,
    debouncer_rx: Receiver<()>,
    spinner: Arc<RwLock<Spinner>>,
    main_widget: MainWidget,
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
            main_widget,
        }
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        tokio::spawn(handle_input_task(self.ui_tx.clone()));
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

        let mut message_opt = Some(Init);

        'main_loop: loop {
            while let Some(msg) = message_opt {
                match msg {
                    Exit => {
                        info!("Exiting application");
                        break 'main_loop;
                    }
                    _ => message_opt = self.main_widget.update(&msg),
                }
            }

            terminal.draw(|frame| self.main_widget.render(frame, frame.area()))?;

            message_opt = self.ui_rx.recv().await;
        }

        Ok(())
    }
}
