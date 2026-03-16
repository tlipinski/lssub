use crate::osb::subtitles::SubtitlesResponse;
use crate::secret::retrieve;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, Exit, FetchSubtitles, Init, LanguagesUpdated, Multi, RunTask, SearchQueryUpdated,
    StartProgress, SubtitleDownloaded, SubtitlesFetched, SwitchScreen, UserLoggedOut,
};
use crate::ui::app::Screen::Search;
use crate::ui::component::Component;
use crate::ui::downloader::{Downloaded, Downloader};
use crate::ui::handled::HandleResult::{Handled, Unhandled};
use crate::ui::languages_widget::LanguagesWidget;
use crate::ui::query_widget::QueryWidget;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_list_widget::{QueryParams, SubsListWidget, Subtitle};
use crate::ui::task_runner::{Task, TaskRunner};
use crate::ui::user_widget::UserWidget;
use crossterm::event::{KeyEvent, KeyModifiers};
use log::error;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use std::path::Path;
use tokio::sync::mpsc::Sender;
use tui_popup::Popup;

pub struct SearchWidget {
    query_widget: QueryWidget,
    subs_list_widget: SubsListWidget,
    downloader: Downloader,
    help: bool,
}

impl Component for SearchWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Init => Some(SearchQueryUpdated(SubtitlesQuery {
                query: self.query_widget.query(),
                params: QueryParams::default(),
            })),
            SubtitlesFetched(subtitles) => {
                self.update_subtitles(&subtitles);
                None
            }
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            if (self.help) {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::Esc, KeyModifiers::NONE) => {
                        self.help = !self.help;
                        None
                    }
                    (KeyCode::F(1), KeyModifiers::NONE) => {
                        self.help = !self.help;
                        None
                    }
                    _ => None,
                }
            } else {
                match (key_event.code, key_event.modifiers) {
                    (KeyCode::F(1), KeyModifiers::NONE) => {
                        self.help = !self.help;
                        None
                    }

                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        self.subs_list_widget.selected().map(|selected_sub| {
                            let downloader = self.downloader.clone();
                            let file_id = selected_sub.file_id;
                            let language = selected_sub.language.clone();

                            Multi(vec![
                                ChangeStatus(format!("Downloading {}", selected_sub.title)),
                                RunTask(Task::new("download subs", async move {
                                    downloader.download(file_id, &language).await
                                })),
                            ])
                        })
                    }

                    _ => match self.subs_list_widget.handle_key_event(key_event) {
                        Handled(result) => result.map(|params| {
                            SearchQueryUpdated(SubtitlesQuery {
                                query: self.query_widget.query(),
                                params,
                            })
                        }),
                        Unhandled => {
                            let params = self.subs_list_widget.params.clone();
                            self.query_widget
                                .handle_key_event(event)
                                .map(|query| SearchQueryUpdated(SubtitlesQuery { query, params }))
                        }
                    },
                }
            }
        } else {
            None
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(1)])
            .split(area);

        self.query_widget.render(frame, layout[0]);
        self.subs_list_widget.render(frame, layout[1]);

        if (self.help) {
            let body = Text::from(vec![
                "".into(),
                Line::from("Ctrl+S: Narrow results to the single title currently selected"),
                Line::from("Ctrl+T: Exclude AI translated subtitles"),
                "".into(),
            ]);
            let popup = Popup::new(body)
                .title(" Help ")
                .style(Style::new().black().on_gray());
            frame.render_widget(popup, area);
        }
    }
}

impl SearchWidget {
    pub fn from(base_path: &Path, file_name: Option<&str>) -> SearchWidget {
        Self {
            query_widget: QueryWidget::from(file_name.unwrap_or("").into()),
            subs_list_widget: SubsListWidget::default(),
            downloader: Downloader::new(base_path.to_owned(), file_name.map(String::from)),
            help: false,
        }
    }

    pub fn update_subtitles(&mut self, subtitles_response: &SubtitlesResponse) {
        self.subs_list_widget.update_subtitles(subtitles_response);
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubtitlesQuery {
    pub query: String,
    pub params: QueryParams,
}
