use crossterm::event::Event;
use crate::osb::subtitles::Subtitle;
use crate::ui::handled::HandleResult;
use crate::ui::handled::HandleResult::Unhandled;
use crate::ui::pad::BlockTitlePadExt;
use HandleResult::Handled;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::StatefulWidget;
use ratatui::prelude::{Style, Text, Widget};
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::widgets::{
    Block, Cell, Row, ScrollDirection, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    TableState,
};
use crate::ui::actions::Action;
use crate::ui::actions::Action::Init;
use crate::ui::app_state::AppState;
use crate::ui::component::Component;

#[derive(Default)]
pub struct SubsListWidget {
    subs: Vec<Subtitle>,
    state: TableState,
    scroll_state: ScrollbarState,
    params: QueryParams,
    single_title: String,
}

#[derive(Default, Clone, Debug)]
pub struct QueryParams {
    pub feature_id: Option<i64>,
    pub parent_feature_id: Option<i64>,
    pub exclude_ai: bool,
}

impl Component for SubsListWidget {
    fn update(&mut self, action: &Action, state: AppState) -> Option<(Action, AppState)> {
        match action {
            Init => {
                self.params = state.params_snapshot?;
                None
            }
            _ => {
                None
            }
        }
    }

    fn handle_key_event(&mut self, event: &Event, state: AppState) -> Option<(Action, AppState)> {
        todo!()
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let wide = area.width > 90;
        let rows: Vec<Row> = self.subs.iter().map(Into::into).collect();

        let mut title = format!("Results: {}", self.subs.len());
        if self.params.feature_id.is_some() || self.params.parent_feature_id.is_some() {
            title.push_str(&format!(" (single title: '{}')", self.single_title));
        }
        if self.params.exclude_ai {
            title.push_str(" (AI excluded)");
        }

        let block_bot = Block::bordered()
            .title_pad(&title)
            .border_set(border::PLAIN);

        let (widths, headers) = if wide {
            (
                [95, 10, 10, 12, 12, 12, 10, 10],
                vec![
                    "Title",
                    "Language",
                    "Year",
                    "Uploaded",
                    "Uploader",
                    "Downloads",
                    "AI",
                    "Votes",
                ],
            )
        } else {
            (
                [50, 4, 4, 10, 10, 10, 3, 3],
                vec!["Title", "Lng", "Yr", "UplDt", "Upl", "Downs", "AI", "Vt"],
            )
        };

        let table = Table::new(rows, widths)
            .header(Row::from_iter(headers))
            .block(block_bot)
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));
        StatefulWidget::render(table, area, frame.buffer_mut(), &mut self.state);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(Color::DarkGray))
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        StatefulWidget::render(scrollbar, area, frame.buffer_mut(), &mut self.scroll_state);
    }
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
                for _i in 1..10 {
                    self.scroll_state.scroll(ScrollDirection::Backward);
                }

                Handled(None)
            }

            (KeyCode::PageDown, KeyModifiers::NONE) => {
                let next = self.state.selected().map_or(0, |i| i.saturating_add(10));
                self.state.select(Some(next));
                for _i in 1..10 {
                    self.scroll_state.scroll(ScrollDirection::Forward);
                }

                Handled(None)
            }

            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                if self.params.feature_id.is_some() {
                    self.params.feature_id = None;
                    Handled(Some(self.params.clone()))
                } else {
                    match self
                        .state
                        .selected()
                        .and_then(|selection| self.subs.get(selection))
                    {
                        Some(selected) => {
                            self.params.feature_id =
                                Some(selected.attributes.feature_details.feature_id);
                            self.single_title = selected.attributes.feature_details.title.clone();
                            Handled(Some(self.params.clone()))
                        }
                        None => Handled(None),
                    }
                }
            }

            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                if self.params.parent_feature_id.is_some() {
                    self.params.parent_feature_id = None;
                    Handled(Some(self.params.clone()))
                } else {
                    match self
                        .state
                        .selected()
                        .and_then(|selection| self.subs.get(selection))
                    {
                        Some(selected) => {
                            if let Some(pfid) =
                                selected.attributes.feature_details.parent_feature_id
                            {
                                self.params.parent_feature_id = Some(pfid);
                                self.single_title = selected
                                    .attributes
                                    .feature_details
                                    .parent_title
                                    .clone()
                                    .unwrap_or(String::new());
                                Handled(Some(self.params.clone()))
                            } else {
                                Handled(None)
                            }
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

    pub fn update_subtitles(&mut self, subs: Vec<Subtitle>) {
        self.scroll_state = self.scroll_state.content_length(subs.len());
        self.scroll_state.first();
        self.state.select_first();

        self.subs = subs;
    }
}

impl<'a> From<&'a Subtitle> for Row<'a> {
    fn from(sub: &'a Subtitle) -> Row<'a> {
        Row::from_iter(vec![
            Cell::from(Text::from(sub.attributes.release.as_str())),
            Cell::from(Text::from(sub.attributes.language.as_str())),
            Cell::from(Text::from(
                sub.attributes
                    .feature_details
                    .year
                    .map(|a| a.to_string())
                    .unwrap_or_default(),
            )),
            Cell::from(Text::from(sub.upload_date())),
            if sub.attributes.uploader.rank == "Trusted member" {
                Cell::from(
                    Text::from(sub.attributes.uploader.name.clone())
                        .style(Style::default().fg(Color::Green)),
                )
            } else {
                Cell::from(Text::from(sub.attributes.uploader.name.clone()))
            },
            Cell::from(Text::from(sub.downloads().to_string())),
            Cell::from(Text::from(if sub.attributes.ai_translated {
                "✓".to_string()
            } else {
                String::new()
            })),
            Cell::from(Text::from(match sub.attributes.votes {
                0 => String::new(),
                other => other.to_string(),
            })),
        ])
    }
}
