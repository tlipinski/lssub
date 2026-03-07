use crate::ui::app::QUIT_KEY;
use crate::ui::actions::Action;
use anyhow::Result;
use gio::glib::random_int_range;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, TableState};
use ratatui::Frame;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

pub struct StatusWidget {
    pub info: String,
    pub spinner: char,
    pub spinning: bool,
}

impl StatusWidget {
    pub fn from(info: String) -> Self {
        Self {
            info,
            spinner: ' ',
            spinning: false,
        }
    }

    pub fn spin(&mut self, chr: char) {
        self.spinner = chr;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut title = if (self.spinning) {
            ("Status ".to_string() + &self.spinner.to_string()).bold()
        } else {
            ("Status".to_string()).bold()
        };

        let block = Block::bordered()
            .title(title)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.info.clone());

        let view = Paragraph::new(par).block(block);

        frame.render_widget(view, area);
    }

}
