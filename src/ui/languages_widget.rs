use crate::config::ConfigProvider;
use crate::osb::languages::{Language, get_languages};
use crate::osb::osb_client::OsbClient;
use crate::ui::actions::Action;
use crate::ui::app_state::AppState;
use crate::ui::actions::Action::{
    LanguagesFetched, LanguagesUpdated, Multi, RunTask, UserLanguagesFetched,
};
use crate::ui::component::Component;
use crate::ui::pad::BlockTitlePadExt;
use crate::ui::task_runner::Task;
use Action::{Init, LanguagesInitialized};
use KeyCode::{Char, Down, Enter, Left, Right, Up};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::prelude::Line;
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_checkbox::Checkbox;

const GRID_CELL_WIDTH: u16 = 36;

pub struct LanguagesWidget {
    languages: Vec<(Language, bool)>,
    focused_idx: usize,
    grid_columns: usize,
    osb_languages_opt: Option<Vec<Language>>,
    user_languages_opt: Option<Vec<String>>,
}

impl Component for LanguagesWidget {
    fn update(&mut self, action: &Action, state: AppState) -> Option<(Action, AppState)> {
        match action {
            Init => Some((
                Multi(vec![
                    RunTask(Task::new("fetch languages", async {
                        let osb_languages = get_languages(OsbClient::default()).await?;

                        Ok(LanguagesFetched(osb_languages))
                    })),
                    RunTask(Task::new("fetch user languages", async {
                        let user_languages = ConfigProvider::default()
                            .get_config()
                            .unwrap()
                            .languages
                            .unwrap_or(Vec::new());

                        Ok(UserLanguagesFetched(user_languages))
                    })),
                ]),
                state,
            )),
            LanguagesFetched(languages) => {
                self.osb_languages_opt = Some(languages.clone());
                self.try_init().map(|action| (action, state))
            }
            UserLanguagesFetched(user_languages) => {
                self.user_languages_opt = Some(user_languages.clone());
                self.try_init().map(|action| (action, state))
            }
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event, state: AppState) -> Option<(Action, AppState)> {
        if let Event::Key(key_event) = event {
            match (key_event.code, key_event.modifiers) {
                (Enter, KeyModifiers::NONE) => {
                    Some((LanguagesUpdated(self.languages()), state))
                }
                (Char(' '), KeyModifiers::NONE) => {
                    self.toggle_focused();
                    None
                }
                (Left | Char('h'), KeyModifiers::NONE) => {
                    self.move_left();
                    None
                }
                (Right | Char('l'), KeyModifiers::NONE) => {
                    self.move_right();
                    None
                }
                (Up | Char('k'), KeyModifiers::NONE) => {
                    self.move_up();
                    None
                }
                (Down | Char('j'), KeyModifiers::NONE) => {
                    self.move_down();
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title_pad("Languages (Space toggle, Enter apply)")
            .border_set(border::PLAIN);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let hint = Paragraph::new(Line::from(
            "Use arrow keys (or h/j/k/l) to move, Space to select/unselect.",
        ));
        let hint_height = 1u16.min(inner.height);
        frame.render_widget(hint, Rect::new(inner.x, inner.y, inner.width, hint_height));

        if inner.height <= hint_height {
            return;
        }

        let grid_area = Rect::new(
            inner.x,
            inner.y + hint_height,
            inner.width,
            inner.height - hint_height,
        );

        self.grid_columns = Self::columns_for_width(grid_area.width);
        let columns = self.grid_columns as u16;
        let cell_width = (grid_area.width / columns).max(1);

        for idx in 0..self.languages.len() {
            let row = idx / self.grid_columns;
            let col = idx % self.grid_columns;
            let y = grid_area.y + row as u16;
            if y >= grid_area.y + grid_area.height {
                break;
            }

            let x = grid_area.x + (col as u16 * cell_width);
            let width = if col + 1 == self.grid_columns {
                grid_area.width - (cell_width * (columns - 1))
            } else {
                cell_width
            };

            let language = &self.languages[idx];
            let mut checkbox = Checkbox::new(
                format!(
                    "{} ({})",
                    language.0.language_name, language.0.language_code
                ),
                language.1,
            )
            .style(Style::default())
            .checked_symbol("[X]")
            .unchecked_symbol("[ ]");

            if language.1 {
                checkbox = checkbox.style(Style::default().bg(Color::Gray));
            }

            if idx == self.focused_idx {
                checkbox = checkbox.style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }

            frame.render_widget(checkbox, Rect::new(x, y, width, 1));
        }
    }
}

impl LanguagesWidget {
    pub fn new() -> LanguagesWidget {
        Self {
            languages: vec![],
            focused_idx: 0,
            grid_columns: 4,
            osb_languages_opt: None,
            user_languages_opt: None,
        }
    }

    fn try_init(&mut self) -> Option<Action> {
        match (
            self.osb_languages_opt.clone(),
            self.user_languages_opt.clone(),
        ) {
            (Some(osb_languages), Some(user_languages)) => {
                self.languages = osb_languages
                    .iter()
                    .map(|lang| (lang.clone(), user_languages.contains(&lang.language_code)))
                    .collect::<Vec<(Language, bool)>>();

                Some(LanguagesInitialized(user_languages))
            }

            _ => None,
        }
    }

    pub fn languages(&self) -> Vec<String> {
        self.languages
            .iter()
            .enumerate()
            .filter(
                |&(
                    _idx,
                    (
                        Language {
                            language_name: _,
                            language_code: _,
                        },
                        selected,
                    ),
                )| *selected,
            )
            .map(
                |(
                    _idx,
                    (
                        Language {
                            language_name: _,
                            language_code,
                        },
                        _selected,
                    ),
                )| (*language_code).clone(),
            )
            .collect()
    }

    fn columns_for_width(width: u16) -> usize {
        usize::max(1, (width / GRID_CELL_WIDTH) as usize)
    }

    fn toggle_focused(&mut self) {
        if let Some(item) = self.languages.get_mut(self.focused_idx) {
            item.1 = !item.1;
        }
    }

    fn move_left(&mut self) {
        if self.focused_idx > 0 {
            self.focused_idx -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.focused_idx + 1 < self.languages.len() {
            self.focused_idx += 1;
        }
    }

    fn move_up(&mut self) {
        if self.focused_idx >= self.grid_columns {
            self.focused_idx -= self.grid_columns;
        }
    }

    fn move_down(&mut self) {
        let next = self.focused_idx + self.grid_columns;
        if next < self.languages.len() {
            self.focused_idx = next;
        }
    }
}
