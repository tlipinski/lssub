use crate::ui::actions::Action;
use crate::ui::actions::Action::{DownloadSubs, EnabledLimitSubsToId, DisabledLimitSubsToId};
use crossterm::event::KeyModifiers;
use log::info;
use osb::subtitles::SubtitlesResponse;
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::prelude::{Style, Stylize, Text, Widget};
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Cell, Row, StatefulWidget, Table, TableState};

#[derive(Debug, Default)]
pub struct SubsWidget {
    pub subs: Vec<Sub>,
    pub limiting_to_id: bool,
    pub state: TableState,
}

#[derive(Debug, Default)]
pub struct Sub {
    id: i64,
    file_id: i64,
    title: String,
    year: String,
    language: String,
    upload_date: String,
    downloads: String,
    ai_translated: String,
    votes: String,
}

impl SubsWidget {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<Action> {
        match key_event {
            KeyEvent {
                code: KeyCode::Up, ..
            } => {
                self.state.select_previous();
                None
            }

            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                self.state.select_next();
                None
            }

            KeyEvent {
                code: KeyCode::PageUp,
                ..
            } => {
                let next = self.state.selected().map_or(0, |i| i.saturating_sub(10));
                self.state.select(Some(next));
                None
            }

            KeyEvent {
                code: KeyCode::PageDown,
                ..
            } => {
                let next = self.state.selected().map_or(0, |i| i.saturating_add(10));
                self.state.select(Some(next));
                None
            }

            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => self
                .state
                .selected()
                .and_then(|selection| self.subs.get(selection))
                .map(|s| DownloadSubs(s.file_id)),

            KeyEvent {
                code: KeyCode::F(5),
                ..
            } => {
                self.limiting_to_id = !self.limiting_to_id;

                self.state
                    .selected()
                    .and_then(|selection| self.subs.get(selection))
                    .map(|s| {
                        if (self.limiting_to_id) {
                            EnabledLimitSubsToId(s.id)
                        } else {
                            DisabledLimitSubsToId
                        }
                    })
            }

            _ => None,
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        let subs = subtitles_response
            .data
            .iter()
            .map(|resp| Sub {
                id: resp.attributes.feature_details.feature_id,
                file_id: resp.attributes.files.first().unwrap().file_id,
                title: resp.attributes.release.clone(),
                year: resp.attributes.feature_details.year.to_string(),
                language: resp.attributes.language.clone(),
                upload_date: resp
                    .attributes
                    .upload_date
                    .split('T')
                    .next()
                    .unwrap_or(&resp.attributes.upload_date)
                    .to_string(),
                downloads: (resp.attributes.download_count + resp.attributes.new_download_count)
                    .to_string(),
                ai_translated: match resp.attributes.ai_translated {
                    true => "✓".to_string(),
                    false => "".to_string(),
                },
                votes: match resp.attributes.votes {
                    0 => "".to_string(),
                    x => x.to_string(),
                },
            })
            .collect::<Vec<Sub>>();

        self.subs = subs;
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let rows = self.subs.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.language.as_str())),
                Cell::from(Text::from(item.year.as_str())),
                Cell::from(Text::from(item.upload_date.as_str())),
                Cell::from(Text::from(item.downloads.as_str())),
                Cell::from(Text::from(item.ai_translated.as_str())),
                Cell::from(Text::from(item.votes.as_str())),
            ])
        });
        let title = format!(" Results: {} ", self.subs.len()).bold();

        let block_bot = Block::bordered().title(title).border_set(border::THICK);

        let table = Table::new(rows, [70, 10, 10, 12, 10, 10, 10])
            .header(Row::from_iter(vec![
                "Title",
                "Language",
                "Year",
                "Uploaded",
                "Downloads",
                "AI",
                "Votes",
            ]))
            .block(block_bot)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_stateful_widget(table, area, &mut self.state);
    }
}
