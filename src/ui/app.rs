use crate::ui::app::UiMessage::{Input, SubsFetched};
use crate::ui::downloader_task::{SubsDownload, downloader_task};
use crate::ui::input_handler::handle_input_task;
use crate::ui::language_widget::LanguageWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::handle_spinner_task;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use crate::ui::ui_messages::UiMessage;
use crate::ui::ui_messages::UiMessage::{
    DownloadSubs, Exit, FetchSubs, Init, LanguagesUpdated, QueryUpdated, SpinnerUpdate,
    StartSpinner, StopSpinner, SwitchScreen, Tuple,
};
use anyhow::Result;
use log::info;
use osb::get_download_link::get_download_link;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{DefaultTerminal, Frame};
use std::ops::Deref;
use std::path::Path;
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
    downloader_tx: Sender<SubsDownload>,
    exit: bool,
}

impl App {
    pub fn run(
        terminal: &mut DefaultTerminal,
        base_path: &Path,
        file_name: Option<&str>,
    ) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiMessage>();
        let (features_tx, features_rx) = mpsc::channel::<SubtitlesQuery>();
        let (downloader_tx, downloader_rx) = mpsc::channel::<SubsDownload>();

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(handle_spinner_task(ui_tx.clone()));
        tokio::spawn(downloader_task(downloader_rx, base_path.to_owned(), file_name.map(|s| s.to_string())));

        let mut app = App {
            current_screen: CurrentScreen::default(),
            search_widget: SearchWidget::from(file_name.unwrap_or("").into()),
            subs_widget: SubsWidget::default(),
            language_widget: LanguageWidget::from(),
            features_tx,
            downloader_tx,
            exit: false,
        };

        let mut message_opt = Some(Init);

        while !app.exit {
            while let Some(msg) = message_opt {
                message_opt = app.handle_ui_message(msg)?
            }

            terminal.draw(|frame| app.draw(frame))?;

            message_opt = Some(ui_rx.recv()?);
        }

        shutdown_tx.send(())?;

        Ok(())
    }

    fn handle_ui_message(&mut self, ui_message: UiMessage) -> Result<Option<UiMessage>> {
        match ui_message {
            Input(event) => Ok(self.handle_key_event(event)),
            SubsFetched(subtitles) => {
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
            FetchSubs(query, languages) => {
                self.features_tx.send(SubtitlesQuery { query, languages })?;
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
            DownloadSubs(file_id) => {
                self.downloader_tx.send(SubsDownload { file_id });
                Ok(None)
            }
            SwitchScreen(screen) => {
                self.current_screen = screen;
                Ok(None)
            }
            Tuple(first, second) => {
                let handled1 = self.handle_ui_message(*first)?;
                let handled2 = self.handle_ui_message(*second)?;
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

    fn handle_key_event(&mut self, event: Event) -> Option<UiMessage> {
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
