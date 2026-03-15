use crate::osb::subtitles::SubtitlesResponse;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{FeatureInfo, FetchSubs, SearchQueryUpdated};
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::subs_list_widget::SingleFeature::Triggered;
use SingleFeature::{Disabled, Enabled};
use crossterm::event::KeyModifiers;
use log::{debug, info};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Style, Stylize, Text, Widget};
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::widgets::{
    Block, Cell, Row, ScrollDirection, Scrollbar, ScrollbarOrientation, ScrollbarState,
    StatefulWidget, Table, TableState,
};

#[derive(Default)]
pub struct SubsListWidget {
    subs: Vec<Sub>,
    state: TableState,
    scroll_state: ScrollbarState,
    single_feature: SingleFeature,
}

#[derive(Debug, Default, PartialEq)]
enum SingleFeature {
    #[default]
    Disabled,
    Triggered,
    Enabled,
}

// todo reduce pubs
#[derive(Debug, Default)]
pub struct Sub {
    feature_id: i64,
    pub file_id: i64,
    pub title: String,
    year: String,
    pub language: String,
    upload_date: String,
    downloads: i32,
    ai_translated: String,
    votes: String,
}

pub struct SubListQueryParams {
    pub feature_id: Option<i64>,
}

impl SubsListWidget {
    pub fn selected(&self) -> Option<&Sub> {
        self.state
            .selected()
            .and_then(|selection| self.subs.get(selection))
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) -> Option<Option<SubListQueryParams>> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.state.select_previous();
                self.scroll_state.prev();
                Some(None)
            }

            (KeyCode::Down, KeyModifiers::NONE) => {
                self.state.select_next();
                self.scroll_state.next();
                Some(None)
            }

            (KeyCode::PageUp, KeyModifiers::NONE) => {
                let next = self.state.selected().map_or(0, |i| i.saturating_sub(10));
                self.state.select(Some(next));
                for i in 1..10 {
                    self.scroll_state.scroll(ScrollDirection::Backward)
                }
                Some(None)
            }

            (KeyCode::PageDown, KeyModifiers::NONE) => {
                let next = self.state.selected().map_or(0, |i| i.saturating_add(10));
                self.state.select(Some(next));
                for i in 1..10 {
                    self.scroll_state.scroll(ScrollDirection::Forward)
                }
                Some(None)
            }

            (KeyCode::Char('l'), KeyModifiers::CONTROL) => {
                if (self.single_feature != Disabled) {
                    self.single_feature = Disabled;
                    Some(Some(SubListQueryParams {feature_id: None}))
                } else {
                    self.single_feature = Triggered;
                    self.state
                        .selected()
                        .and_then(|selection| self.subs.get(selection))
                        .map(|s| Some(SubListQueryParams {feature_id: Some(s.feature_id)}))
                }
            }

            // (KeyCode::Char('i'), KeyModifiers::CONTROL) => self
            //     .state
            //     .selected()
            //     .and_then(|selection| self.subs.get(selection))
            //     .map(|s| FeatureInfo(s.feature_id)),
            _ => None,
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        // SingleFeature option is enabled only for *single* subs update
        // after another update it gets disabled again
        if (self.single_feature == Triggered) {
            self.single_feature = Enabled;
        } else {
            self.single_feature = Disabled;
        }

        self.scroll_state = self
            .scroll_state
            .content_length(subtitles_response.data.len());

        let subs = subtitles_response
            .data
            .iter()
            .map(|resp| Sub {
                feature_id: resp.attributes.feature_details.feature_id,
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
                downloads: (resp.attributes.download_count + resp.attributes.new_download_count),
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
        let wide = area.width > 90;
        let rows = self.subs.iter().map(|item| {
            Row::from_iter(vec![
                Cell::from(Text::from(item.title.as_str())),
                Cell::from(Text::from(item.language.as_str())),
                Cell::from(Text::from(item.year.as_str())),
                Cell::from(Text::from(item.upload_date.as_str())),
                if (wide) {
                    Cell::from(Text::from(item.downloads.to_string()).right_aligned())
                } else {
                    if (item.downloads >= 1000) {
                        Cell::from(
                            Text::from((item.downloads / 1000).to_string() + "k").right_aligned(),
                        )
                    } else {
                        Cell::from(Text::from(item.downloads.to_string()).right_aligned())
                    }
                },
                Cell::from(Text::from(item.ai_translated.as_str())),
                Cell::from(Text::from(item.votes.as_str()).right_aligned()),
            ])
        });

        let title = if (self.single_feature == Enabled) {
            format!("Results: {} (single feature)", self.subs.len())
        } else {
            format!("Results: {}", self.subs.len())
        };

        let block_bot = Block::bordered()
            .title_pad(&title)
            .border_set(border::PLAIN);

        let (widths, headers) = if (wide) {
            (
                [95, 10, 10, 12, 12, 10, 10],
                vec![
                    "Title",
                    "Language",
                    "Year",
                    "Uploaded",
                    "Downloads",
                    "AI",
                    "Votes",
                ],
            )
        } else {
            (
                [50, 4, 4, 10, 10, 3, 3],
                vec!["Title", "Lng", "Yr", "Upl", "Downs", "AI", "Vt"],
            )
        };

        let table = Table::new(rows, widths)
            .header(Row::from_iter(headers))
            .block(block_bot)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_stateful_widget(table, area, &mut self.state);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(Color::DarkGray))
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut self.scroll_state,
        );
    }
}
