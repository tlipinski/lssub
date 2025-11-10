use crate::ui::app::UiEvent::{Input, ResultsUpdate};
use crate::ui::downloader_task::downloader_task;
use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::{
    DownloadConfirmed, Exit, FetchSubs, Init, LanguagesUpdated, QueryUpdated, SpinnerUpdate,
    StartSpinner, StopSpinner, SwitchScreen, Tuple,
};
use crate::ui::input_handler::handle_input_task;
use crate::ui::language_widget::LanguageWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::handle_spinner;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use anyhow::Result;
use log::info;
use osb::download::download;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{DefaultTerminal, Frame};
use std::ops::Deref;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use tokio::sync::broadcast;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

#[derive(Debug)]
pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    subs_widget: SubsWidget,
    language_widget: LanguageWidget,
    features_tx: Sender<SubtitlesQuery>,
    downloader_tx: Sender<i64>,
    exit: bool,
}

impl App {
    pub fn run(terminal: &mut DefaultTerminal, file_name: String) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (features_tx, features_rx) = mpsc::channel::<SubtitlesQuery>();

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        let (downloader_tx, downloader_rx) = mpsc::channel::<i64>();

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(handle_spinner(ui_tx.clone()));
        tokio::spawn(downloader_task(downloader_rx));

        let mut app = App {
            current_screen: CurrentScreen::default(),
            search_widget: SearchWidget::from(file_name.clone()),
            subs_widget: SubsWidget::default(),
            language_widget: LanguageWidget::from(),
            features_tx,
            downloader_tx,
            exit: false,
        };

        let mut event_opt = Some(Init);

        while !app.exit {
            while let Some(event) = event_opt {
                event_opt = app.handle_ui_events(event)?
            }

            terminal.draw(|frame| app.draw(frame))?;

            event_opt = Some(ui_rx.recv()?);
        }

        shutdown_tx.send(())?;

        Ok(())
    }

    fn handle_ui_events(&mut self, ui_event: UiEvent) -> Result<Option<UiEvent>> {
        match ui_event {
            Input(event) => Ok(self.handle_key_event(event)),
            ResultsUpdate(subtitles) => {
                // info!("ResultsUpdate: {:?}", subtitles);
                self.subs_widget.update_subtitles(subtitles);
                Ok(Some(StopSpinner))
            }
            SpinnerUpdate(chr) => {
                self.search_widget.spin(chr);
                Ok(None)
            }
            LanguagesUpdated(langs) => {
                let query: String = self.search_widget.input.value().into();
                Ok(Some(FetchSubs(query, langs)))
            }
            QueryUpdated(query) => {
                let langs = self.language_widget.languages();
                Ok(Some(FetchSubs(query, langs)))
            }
            FetchSubs(q, l) => {
                self.features_tx.send(SubtitlesQuery {
                    query: q,
                    languages: l,
                })?;
                Ok(Some(StartSpinner))
            }
            StartSpinner => {
                self.search_widget.spinning = true;
                Ok(None)
            }
            StopSpinner => {
                self.search_widget.spinning = false;
                Ok(None)
            }
            Init => {
                let query: String = self.search_widget.input.value().into();
                if (!query.is_empty()) {
                    let langs = self.language_widget.languages();
                    Ok(Some(FetchSubs(query, langs)))
                } else {
                    Ok(None)
                }
            }
            DownloadConfirmed(file_id) => {
                self.downloader_tx.send(file_id);
                Ok(None)
            }
            SwitchScreen(screen) => {
                self.current_screen = screen;
                Ok(None)
            }
            Tuple(first, second) => {
                let handled1 = self.handle_ui_events(*first)?;
                let handled2 = self.handle_ui_events(*second)?;
                match (handled1, handled2) {
                    (Some(e1), Some(e2)) => Ok(Some(Tuple(Box::new(e1), Box::new(e2)))),
                    (None, Some(e)) => Ok(Some(e)),
                    (Some(e), None) => Ok(Some(e)),
                    _ => Ok(None),
                }
            }
            Exit => {
                self.exit = true;
                Ok(None)
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_screen {
            CurrentScreen::Main => {
                let area = frame.area();

                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(0), Constraint::Percentage(100)])
                    .split(area);

                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(10)])
                    .split(layout[1]);

                self.search_widget.render(frame, right[0]);
                self.subs_widget.render(frame, right[1]);
            }
            CurrentScreen::Language => {
                let area = frame.area();

                let layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(0), Constraint::Percentage(100)])
                    .split(area);

                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Min(10)])
                    .split(layout[1]);

                self.language_widget.render(frame, area);
            }
        }
    }

    fn handle_key_event(&mut self, event: Event) -> Option<UiEvent> {
        if let Event::Key(key_event) = event {
            match self.current_screen {
                CurrentScreen::Main => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    KeyCode::F(2) => Some(SwitchScreen(CurrentScreen::Language)),
                    KeyCode::Up | KeyCode::Down | KeyCode::Enter => {
                        self.subs_widget.handle_key_event(key_event)
                    }
                    _ => self.search_widget.handle_key_event(event),
                },
                CurrentScreen::Language => match key_event.code {
                    QUIT_KEY => Some(SwitchScreen(CurrentScreen::Main)),
                    KeyCode::F(2) => Some(SwitchScreen(CurrentScreen::Main)),
                    _ => {
                        let event = self.language_widget.handle_key_event(event);
                        if let Some(evt) = event {
                            Some(Tuple(
                                Box::new(evt),
                                Box::new(SwitchScreen(CurrentScreen::Main)),
                            ))
                        } else {
                            event
                        }
                    }
                },
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    Language,
}
