use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Text, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Cell, Row, Table, TableState};

#[derive(Debug, Default)]
pub struct Subs{
    pub data: Vec<Sub>,
    pub state: TableState
}

#[derive(Debug, Default)]
pub struct Sub {
    pub title: String,
    pub language: String,
    pub upload_date: String,
}

impl Widget for &Subs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.data.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.language.as_str())),
                Cell::from(Text::from(item.upload_date.as_str())),
            ])
        });

        let block_bot = Block::bordered()
            .title(format!(" Results: {} ", self.data.len()).bold())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Table::new(rows, [70, 10, 10])
            .header(Row::from_iter(vec!["Title", "Language", "Uploaded"]))
            .block(block_bot)
            .render(area, buf);
    }
}
