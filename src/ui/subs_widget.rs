use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Stylize, Text, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Cell, Row, Table};

#[derive(Debug, Default)]
pub struct Subs(pub Vec<Sub>);

#[derive(Debug, Default)]
pub struct Sub {
    pub id: String,
    pub title: String,
    pub year: String,
}

impl Widget for &Subs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.0.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.id.as_str())),
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.year.as_str())),
            ])
        });

        let block_bot = Block::bordered()
            .title(format!(" Results: {} ", self.0.len()).bold())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Table::new(rows, [10, 50, 10])
            .header(Row::from_iter(vec!["ID", "Title", "Year"]))
            .block(block_bot)
            .render(area, buf);
    }
}
