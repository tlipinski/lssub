use crate::ui::actions::Action;
use crate::ui::actions::Action::{ChangeStatus, StopProgress};
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::spinner::Spinner;
use Action::StartProgress;
use ratatui::Frame;
use ratatui::crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use std::sync::{Arc, RwLock};

pub struct StatusWidget {
    info: String,
    spinner: Arc<RwLock<Spinner>>,
    in_progress: bool,
}

impl Component for StatusWidget {
    fn update(&mut self, action: &Action, _state: AppState) -> Option<(Action, AppState)> {
        match action {
            ChangeStatus(status) => {
                self.info = status.clone();
            }
            StartProgress => {
                self.in_progress = true;
            }
            StopProgress => {
                self.in_progress = false;
            }
            _ => {}
        }
        None
    }

    fn handle_key_event(&mut self, _event: &Event, _state: AppState) -> Option<(Action, AppState)> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let c = self.spinner.read().unwrap().c;
        let title = if self.in_progress {
            "Status ".to_string() + &c.to_string()
        } else {
            "Status".to_string()
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
            info: String::new(),
            spinner,
            in_progress: false,
        }
    }
}
