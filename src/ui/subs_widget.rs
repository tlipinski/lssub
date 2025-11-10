use crate::ui::events::UiEvent;
use log::info;
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
}

#[derive(Debug, Default)]
pub struct Sub {
    file_id: i64,
    title: String,
    year: String,
    language: String,
    upload_date: String,
    downloads: String,
}

impl SubsWidget {
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let rows = self.subs.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.language.as_str())),
                Cell::from(Text::from(item.year.as_str())),
                Cell::from(Text::from(item.upload_date.as_str())),
                Cell::from(Text::from(item.downloads.as_str())),
            ])
        });
        let mut title = format!(" Results: {} ", self.subs.len()).bold();

        let block_bot = Block::bordered().title(title).border_set(border::THICK);

        let table = Table::new(rows, [70, 10, 10, 10, 10])
            .header(Row::from_iter(vec![
                "Title",
                "Language",
                "Year",
                "Uploaded",
                "Downloads",
            ]))
            .block(block_bot)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        f.render_stateful_widget(table, area, &mut self.state)
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<UiEvent> {
        match key_event.code {
            KeyCode::Up => {
                self.state.select_previous();
                None
            }
            KeyCode::Down => {
                self.state.select_next();
                None
            }
            KeyCode::Enter => {
                self.state
                    .selected()
                    .map(|selection| self.subs.get(selection))
                    .flatten()
                    .map(|s| UiEvent::DownloadConfirmed(s.file_id))
            }
            _ => None,
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: SubtitlesResponse) {
        let subs = subtitles_response
            .data
            .iter()
            .map(|resp| Sub {
                file_id: resp.attributes.files.get(0).unwrap().file_id,
                title: resp.attributes.release.clone(),
                year: resp.attributes.feature_details.year.to_string(),
                language: resp.attributes.language.clone(),
                upload_date: resp.attributes.upload_date.clone(),
                downloads: resp.attributes.download_count.to_string(),
            })
            .collect::<Vec<Sub>>();

        self.subs = subs;
    }
}
