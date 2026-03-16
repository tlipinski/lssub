use crate::osb::subtitles::SubtitlesResponse;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{FeatureInfo, FetchSubtitles, SearchQueryUpdated};
use crate::ui::handled::HandleResult;
use crate::ui::handled::HandleResult::Unhandled;
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::subs_list_widget::SingleFeature::Triggered;
use HandleResult::Handled;
use SingleFeature::{Disabled, Enabled};
use log::{debug, info};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyModifiers;
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
    subs: Vec<Subtitle>,
    state: TableState,
    scroll_state: ScrollbarState,
    single_feature: SingleFeature,
    pub params: QueryParams,
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
pub struct Subtitle {
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

#[derive(Default, Clone, Debug)]
pub struct QueryParams {
    pub feature_id: Option<i64>,
    pub exclude_ai: bool,
}

impl SubsListWidget {
    pub fn selected(&self) -> Option<&Subtitle> {
        self.state
            .selected()
            .and_then(|selection| self.subs.get(selection))
    }

    pub fn handle_key_event(&mut self, key_event: &KeyEvent) -> HandleResult<Option<QueryParams>> {
        match (key_event.code, key_event.modifiers) {
            (KeyCode::Up, KeyModifiers::NONE) => {
                self.state.select_previous();
                self.scroll_state.prev();

                Handled(None)
            }

            (KeyCode::Down, KeyModifiers::NONE) => {
                self.state.select_next();
                self.scroll_state.next();

                Handled(None)
            }

            (KeyCode::PageUp, KeyModifiers::NONE) => {
                let next = self.state.selected().map_or(0, |i| i.saturating_sub(10));
                self.state.select(Some(next));
                for i in 1..10 {
                    self.scroll_state.scroll(ScrollDirection::Backward)
                }

                Handled(None)
            }

            (KeyCode::PageDown, KeyModifiers::NONE) => {
                let next = self.state.selected().map_or(0, |i| i.saturating_add(10));
                self.state.select(Some(next));
                for i in 1..10 {
                    self.scroll_state.scroll(ScrollDirection::Forward)
                }

                Handled(None)
            }

            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if (self.single_feature != Disabled) {
                    self.single_feature = Disabled;
                    self.params.feature_id = None;
                    Handled(Some(self.params.clone()))
                } else {
                    self.single_feature = Triggered;
                    match self
                        .state
                        .selected()
                        .and_then(|selection| self.subs.get(selection))
                    {
                        Some(selected) => {
                            self.params.feature_id = Some(selected.feature_id);
                            Handled(Some(self.params.clone()))
                        }
                        None => Handled(None),
                    }
                }
            }

            (KeyCode::Char('t'), KeyModifiers::CONTROL) => {
                self.params.exclude_ai = !self.params.exclude_ai;

                Handled(Some(self.params.clone()))
            }

            // (KeyCode::Char('i'), KeyModifiers::CONTROL) => self
            //     .state
            //     .selected()
            //     .and_then(|selection| self.subs.get(selection))
            //     .map(|s| FeatureInfo(s.feature_id)),
            _ => Unhandled,
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        // SingleFeature option is enabled only for *single* subs update
        // after another update it gets disabled again
        if (self.single_feature == Triggered) {
            self.single_feature = Enabled;
        } else {
            self.single_feature = Disabled;
            self.params.feature_id = None;
        }

        self.scroll_state = self
            .scroll_state
            .content_length(subtitles_response.data.len());

        let subs = subtitles_response
            .data
            .iter()
            .map(|resp| Subtitle {
                feature_id: resp.attributes.feature_details.feature_id,
                file_id: resp.attributes.files.first().unwrap().file_id,
                title: resp.attributes.release.clone(),
                year: resp
                    .attributes
                    .feature_details
                    .year
                    .map(|year| year.to_string())
                    .unwrap_or_default(),
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
            .collect::<Vec<Subtitle>>();

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

        let mut title = format!("Results: {}", self.subs.len());
        if (self.single_feature == Enabled) {
            title.push_str(" (single feature)");
        }
        if (self.params.exclude_ai) {
            title.push_str(" (AI excluded)");
        }

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
