use osb::subtitles::SubtitlesResponse;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Stylize, Text};
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget, Table, TableState};

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
    // fn r(&mut self, f: &mut Frame, area: Rect) {
    //     let table = self.view();
    //     f.render_stateful_widget(table, area, &mut self.state)
    // }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let rows = self.subs.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.language.as_str())),
                Cell::from(Text::from(item.upload_date.as_str())),
            ])
        });
        let mut title = format!(" Results: {} ", self.subs.len()).bold();
        
        if (!self.active) {
            title = title.red().gray();
        }

        let block_bot = Block::bordered()
            .title(title)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let table = Table::new(rows, [70, 10, 10])
            .header(Row::from_iter(vec!["Title", "Language", "Uploaded"]))
            .block(block_bot)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_stateful_widget(table, area, &mut self.state)
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => self.state.select_next(),
            _ => {}
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: SubtitlesResponse) {
        let subs = subtitles_response
            .data
            .iter()
            .take(20)
            .map(|resp| Sub {
                title: resp.attributes.release.clone(),
                language: resp.attributes.language.clone(),
                upload_date: resp.attributes.upload_date.clone(),
            })
            .collect::<Vec<Sub>>();

        self.subs = subs;
    }

}
