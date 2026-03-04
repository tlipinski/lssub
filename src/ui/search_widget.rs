use crate::ui::actions::Action;
use crate::ui::app::QUIT_KEY;
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use anyhow::Result;
use gio::glib::random_int_range;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, TableState};
use std::sync::mpsc::Sender;
use std::thread::sleep;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use crate::ui::actions::Action::{SearchQueryUpdated, RequestedSubs};

#[derive(Debug)]
pub struct SearchWidget {
    pub input: Input,
}

impl SearchWidget {
    pub fn from(search_text: String) -> Self {
        SearchWidget {
            input: Input::from(search_text),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title("Search")
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.input.value().bold());

        let view = Paragraph::new(par).block(block);

        let x = self.input.visual_cursor();
        frame.set_cursor_position((area.x + (x + 1) as u16, area.y + 1));

        frame.render_widget(view, area);
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Some(state_changed) = self.input.handle_event(&event)
            && state_changed.value
        {
            return Ok(Some(SearchQueryUpdated));
        }
        Ok(None)
    }
}
