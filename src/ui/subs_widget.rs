use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Text, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};

#[derive(Debug, Default)]
pub struct SubsWidget {
    pub subs: Vec<Sub>,
    pub state: TableState,
    pub active: bool
}

#[derive(Debug, Default)]
pub struct Sub {
    pub title: String,
    pub language: String,
    pub upload_date: String,
}

impl SubsWidget {

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                self.state.select_previous()
            }
            KeyCode::Down => {
                self.state.select_next()
            }
            KeyCode::Esc => {
                self.active = false;
            }
            _ => {}
        }
    }
    
}

impl Widget for &SubsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.subs.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.language.as_str())),
                Cell::from(Text::from(item.upload_date.as_str())),
            ])
        });
        let mut title = format!(" Results: {} ", self.subs.len()).bold();
        if (self.active) {
            title = title.red();
        }
        
        let block_bot = Block::bordered()
            .title(title)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Table::new(rows, [70, 10, 10])
            .header(Row::from_iter(vec!["Title", "Language", "Uploaded"]))
            .block(block_bot)
            .render(area, buf);
    }
}
