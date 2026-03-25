use std::collections::HashMap;
use std::path::Path;
use crate::ui::actions::Action;
use crate::ui::actions::Action::FetchSubtitles;
use crate::ui::debouncer::debouncer_task;
use crate::ui::input_handler::handle_input_task;
use crate::ui::spinner::{spinner_task, Spinner};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::config::ConfigProvider;
use crate::ui::about_widget::AboutWidget;
use crate::ui::account_widget::AccountWidget;
use crate::ui::app_widget::{AppWidget, Screen};
use crate::ui::component::Component;
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::nav_widget::NavWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::status_widget::StatusWidget;
use crate::ui::task_runner::TaskRunner;
use crate::ui::user_widget::UserWidget;

pub struct AppBackground {
    ui_tx: Sender<Action>,
    debouncer_rx: Receiver<()>,
    spinner: Arc<RwLock<Spinner>>,
}

impl AppBackground {
    pub fn from(
        ui_tx: Sender<Action>,
        debouncer_rx: Receiver<()>,
        spinner: Arc<RwLock<Spinner>>,
    ) -> AppBackground {
        AppBackground {
            ui_tx,
            debouncer_rx,
            spinner,
        }
    }

    pub fn run(self) {
        tokio::spawn(handle_input_task(self.ui_tx.clone()));
        tokio::spawn(spinner_task(self.spinner));

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
    }
}
