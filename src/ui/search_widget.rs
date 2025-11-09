use crate::ui::app::QUIT_KEY;
use anyhow::Result;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, TableState};
use std::sync::mpsc::Sender;
use std::thread::sleep;
use gio::glib::random_int_range;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use crate::ui::commands::UICommand;
use crate::ui::events::UiEvent;

#[derive(Debug)]
pub struct SearchWidget {
    pub input: Input,
    pub spinner: char,
    pub spinning: bool,
}

impl SearchWidget {
    pub fn from(search_text: String) -> Self {
        SearchWidget {
            input: Input::from(search_text),
            spinner: ' ',
            spinning: false,
        }
    }

    pub fn spin(&mut self, chr: char) {
        self.spinner = chr;
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let mut title =
            if (self.spinning) {
                (" Search ".to_string() + &self.spinner.to_string() + " ").bold()
            } else {
                (" Search ".to_string()).bold()
            };

        let block = Block::bordered()
            .title(title)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);

        let x = self.input.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, area);
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<UiEvent> {
        if let Some(state_changed) = self.input.handle_event(&event) {
            if state_changed.value {
                return Some(UiEvent::QueryUpdated(self.input.value().into()))
            }
        }
        None
    }
}
