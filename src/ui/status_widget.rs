use crate::ui::actions::Action;
use crate::ui::spinner::Spinner;
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
use crate::ui::pad::BlockTitlePadExt;

pub struct StatusWidget {
    pub info: String,
    spinner: Arc<RwLock<Spinner>>,
    pub in_progress: bool,
}

impl StatusWidget {
    pub fn from(spinner: Arc<RwLock<Spinner>>) -> Self {
        Self {
            info: "".to_string(),
            spinner,
            in_progress: false,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let c = self.spinner.read().unwrap().c;
        let mut title = if (self.in_progress) {
            ("Status ".to_string() + &c.to_string())
        } else {
            ("Status".to_string())
        };

        let block = Block::bordered().title_pad(title.as_str()).border_set(border::PLAIN);

        let par = Line::from(self.info.clone());

        let view = Paragraph::new(par).block(block);

        frame.render_widget(view, area);
    }
}
