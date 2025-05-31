use crate::ui::app::QUIT_KEY;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};

#[derive(Debug, Default)]
pub struct SearchWidget {
    pub search_text: String,
    pub active: bool,
}

impl SearchWidget {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Backspace => {
                self.search_text.pop();
            }
            KeyCode::Char(key) => {
                self.search_text.push(key);
            }
            KeyCode::Esc => {
                self.active = false;
            }
            _ => {}
        }
    }
}

impl Widget for &SearchWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Search ".bold());
        let span = if self.active {
            " Search ".bold().red()
        } else {
            " Search ".bold()
        };
        let block = Block::bordered()
            .title(span)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.search_text.clone().bold());

        Paragraph::new(par).block(block).render(area, buf);
    }
}
