use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, DownloadedSubs, StopProgress};
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::spinner::Spinner;
use Action::StartProgress;
use anyhow::Result;
use gio::glib::random_int_range;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, TableState};
use std::sync::{Arc, RwLock};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

pub struct StatusWidget {
    info: String,
    spinner: Arc<RwLock<Spinner>>,
    in_progress: bool,
}

impl Component for StatusWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            ChangeStatus(status) => {
                self.info = status.clone();
                None
            }
            StartProgress => {
                self.in_progress = true;
                None
            }
            StopProgress => {
                self.in_progress = false;
                None
            }
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let c = self.spinner.read().unwrap().c;
        let mut title = if (self.in_progress) {
            ("Status ".to_string() + &c.to_string())
        } else {
            ("Status".to_string())
        };

        let block = Block::bordered()
            .title_pad(title.as_str())
            .border_set(border::PLAIN);

        let par = Line::from(self.info.clone());

        let view = Paragraph::new(par).block(block);

        frame.render_widget(view, area);
    }
}

impl StatusWidget {
    pub fn from(spinner: Arc<RwLock<Spinner>>) -> Self {
        Self {
            info: "".to_string(),
            spinner,
            in_progress: false,
        }
    }
}
