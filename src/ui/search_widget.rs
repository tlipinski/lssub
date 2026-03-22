use crate::osb::subtitles::Subtitle;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, Init, Multi, RunTask, SearchQueryUpdated, SubtitlesFetched,
};
use crate::ui::component::Component;
use crate::ui::downloader::Downloader;
use crate::ui::handled::HandleResult::{Handled, Unhandled};
use crate::ui::query_widget::QueryWidget;
use crate::ui::subs_list_widget::{QueryParams, SubsListWidget};
use crate::ui::task_runner::Task;
use KeyCode::{Enter, Esc, F};
use crossterm::event::KeyModifiers;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Text};
use std::path::Path;
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
                self.update_subtitles(subtitles.to_vec());
                None
            }
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            if self.help {
                match (key_event.code, key_event.modifiers) {
                    (Esc | F(1), KeyModifiers::NONE) => {
                        self.help = !self.help;
                        None
                    }
                    _ => None,
                }
            } else {
                match (key_event.code, key_event.modifiers) {
                    (F(1), KeyModifiers::NONE) => {
                        self.help = !self.help;
                        None
                    }

                    (Enter, KeyModifiers::NONE) => {
                        self.subs_list_widget.selected().map(|selected_sub| {
                            let downloader = self.downloader.clone();
                            let file_id = selected_sub.attributes.files.first().unwrap().file_id;
                            let language = selected_sub.attributes.language.clone();

                            Multi(vec![
                                ChangeStatus(format!(
                                    "Downloading {}",
                                    selected_sub.attributes.release
                                )),
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

        if self.help {
            let body = Text::from(vec![
                "".into(),
                Line::from("Ctrl+P: Narrow results to TV series currently selected"),
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

    pub fn update_subtitles(&mut self, subtitles: Vec<Subtitle>) {
        self.subs_list_widget.update_subtitles(subtitles);
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubtitlesQuery {
    pub query: String,
    pub params: QueryParams,
}
