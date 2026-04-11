use crate::osb::subtitles::Subtitle;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, Init, Multi, RunTask, SearchParamsUpdated, SubtitlesFetched,
};
use crate::ui::app_state::AppState;
use crate::ui::component::Component;
use crate::ui::downloader::Downloader;
use crate::ui::handled::HandleResult::{Handled, Unhandled};
use crate::ui::query_widget::QueryWidget;
use crate::ui::subs_list_widget::SubsListWidget;
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
    fn update(&mut self, action: &Action, state: AppState) -> Option<(Action, AppState)> {
        match action {
            Init => {
                if let Some((action1, state1)) = self.query_widget.update(action, state.clone()) {
                    if let Some((action2, state2)) =
                        self.subs_list_widget.update(action, state1.clone())
                    {
                        Some((Multi(vec![action1, action2]), state2))
                    } else {
                        Some((Multi(vec![action1]), state1))
                    }
                } else {
                    self.subs_list_widget.update(action, state)
                }
            }

            SubtitlesFetched(subtitles) => {
                self.update_subtitles(subtitles.clone());
                None
            }
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event, state: AppState) -> Option<(Action, AppState)> {
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

                            (
                                Multi(vec![
                                    ChangeStatus(format!(
                                        "Downloading {}",
                                        selected_sub.attributes.release
                                    )),
                                    RunTask(Task::new("download subs", async move {
                                        downloader.download(file_id, &language).await
                                    })),
                                ]),
                                state,
                            )
                        })
                    }

                    _ => match self.subs_list_widget.handle_key_event(key_event) {
                        Handled(result) => result.map(|params| {
                            let new_state = AppState {
                                params: params,
                                ..state
                            };
                            (SearchParamsUpdated, new_state)
                        }),
                        Unhandled => self.query_widget.handle_key_event(event, state),
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
            query_widget: QueryWidget::default(),
            subs_list_widget: SubsListWidget::default(),
            downloader: Downloader::new(base_path.to_owned(), file_name.map(String::from)),
            help: false,
        }
    }

    pub fn update_subtitles(&mut self, subtitles: Vec<Subtitle>) {
        self.subs_list_widget.update_subtitles(subtitles);
    }
}
