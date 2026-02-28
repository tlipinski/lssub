use crate::config::ConfigProvider;
use crate::secret::{clear, retrieve, store};
use crate::ui::app::CurrentScreen::{Auth, Language, Main};
use crate::ui::app::UiMessage::{Input, SubsFetched};
use crate::ui::downloader_task::{SubsDownload, downloader_task};
use crate::ui::input_handler::handle_input_task;
use crate::ui::language_widget::LanguageWidget;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::spinner_task;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use crate::ui::ui_messages::UiMessage;
use crate::ui::ui_messages::UiMessage::{
    DownloadSubs, DownloadSubsFailed, DownloadedSubs, Exit, FetchSubs, GoToLogin, Init,
    LanguagesUpdated, Login, LoginFailed, LoginSuccessful, QueryUpdated, SpinnerUpdate,
    StartSpinner, StopSpinner, SwitchScreen,
};
use anyhow::{Error, Result, bail};
use log::{error, info};
use osb::get_download_link::get_download_link;
use osb::login::login;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::StatefulWidget;
use ratatui::{DefaultTerminal, Frame};
use std::ops::Deref;
use std::path::Path;
use std::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

#[derive(Debug)]
pub struct App {
    config_provider: ConfigProvider,
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    subs_widget: SubsWidget,
    language_widget: LanguageWidget,
    status_widget: StatusWidget,
    login_widget: LoginWidget,
    logged_in_widget: LoggedInWidget,
    features_tx: Sender<SubtitlesQuery>,
    downloader_tx: Sender<SubsDownload>,
    exit: bool,
}

impl App {
    pub async fn run(
        terminal: &mut DefaultTerminal,
        base_path: &Path,
        file_name: Option<&str>,
    ) -> Result<()> {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<UiMessage>(100);
        let (features_tx, features_rx) = tokio::sync::mpsc::channel::<SubtitlesQuery>(100);
        let (downloader_tx, downloader_rx) = tokio::sync::mpsc::channel::<SubsDownload>(100);

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(spinner_task(ui_tx.clone()));
        tokio::spawn(downloader_task(
            downloader_rx,
            ui_tx.clone(),
            base_path.to_owned(),
            file_name.map(|s| s.to_string()),
        ));

        let provider = ConfigProvider::default();
        let languages = provider.get_config()?.languages;
        let mut app = App {
            config_provider: provider,
            current_screen: CurrentScreen::default(),
            search_widget: SearchWidget::from(file_name.unwrap_or("").into()),
            subs_widget: SubsWidget::default(),
            language_widget: LanguageWidget::from(languages),
            status_widget: StatusWidget::from("...".into()),
            login_widget: LoginWidget::from(),
            logged_in_widget: LoggedInWidget::from(),
            features_tx,
            downloader_tx,
            exit: false,
        };

        let mut message_opt = Some(Init);

        while !app.exit {
            while let Some(msg) = message_opt {
                let result = app.update(msg).await;
                match result {
                    Ok(r) => message_opt = r,
                    Err(e) => {
                        error!("Error while updating UI: {e}");
                        message_opt = None
                    }
                }
            }

            terminal.draw(|frame| app.draw(frame))?;

            message_opt = ui_rx.recv().await;
        }

        shutdown_tx.send(())?;

        Ok(())
    }

    async fn update(&mut self, ui_message: UiMessage) -> Result<Option<UiMessage>> {
        match ui_message {
            Input(event) => Ok(self.handle_key_event(event)),

            SubsFetched(subtitles) => {
                self.subs_widget.update_subtitles(&subtitles);
                self.status_widget.info = format!("{} results", subtitles.data.len());
                Ok(Some(StopSpinner))
            }

            SpinnerUpdate(chr) => {
                self.status_widget.spin(chr);
                Ok(None)
            }

            LanguagesUpdated(languages) => {
                self.current_screen = Main;
                let query: String = self.search_widget.input.value().into();
                let mut config = self.config_provider.get_config()?;
                config.languages = languages.clone();
                self.config_provider.save_config(&config);
                Ok(Some(FetchSubs(query, languages)))
            }

            QueryUpdated(query) => {
                let languages = self.language_widget.languages();
                Ok(Some(FetchSubs(query, languages)))
            }

            FetchSubs(query, languages) => {
                self.features_tx
                    .send(SubtitlesQuery { query, languages })
                    .await?;
                Ok(Some(StartSpinner))
            }

            StartSpinner => {
                self.status_widget.spinning = true;
                Ok(None)
            }

            StopSpinner => {
                self.status_widget.spinning = false;
                Ok(None)
            }

            Init => {
                let query: String = self.search_widget.input.value().into();
                if (!query.is_empty()) {
                    let languages = self.language_widget.languages();
                    Ok(Some(FetchSubs(query, languages)))
                } else {
                    Ok(None)
                }
            }

            DownloadSubs(file_id) => {
                self.downloader_tx.send(SubsDownload { file_id }).await?;
                self.status_widget.info = "Downloading...".into();
                Ok(Some(StartSpinner))
            }

            DownloadedSubs(path) => {
                self.status_widget.info = format!("Downloaded: {:?}", path);
                Ok(Some(StopSpinner))
            }

            SwitchScreen(screen) => {
                self.current_screen = screen;
                Ok(None)
            }

            GoToLogin => {
                let result = retrieve().await;
                match result {
                    Ok(Some(token)) => Ok(Some(SwitchScreen(CurrentScreen::Logout))),
                    Ok(None) => Ok(Some(SwitchScreen(Auth))),
                    Err(e) => Err(e),
                }
            }

            Login(credentials) => {
                let result = tokio::spawn(async move {
                    match login(&credentials).await {
                        Ok(api_token) => {
                            store(&api_token, &credentials.username).await;
                            LoginSuccessful
                        }
                        Err(e) => LoginFailed(e.to_string()),
                    }
                })
                .await;

                match result {
                    Ok(msg) => Ok(Some(msg)),
                    Err(e) => {
                        error!("Error logging in: {}", e);
                        Err(e.into())
                    }
                }
            }

            LoginFailed(reason) => {
                self.login_widget.failed = reason;
                Ok(None)
            }

            DownloadSubsFailed(error) => {
                self.status_widget.info = format!("Error: {:?}", error);
                Ok(Some(StopSpinner))
            }

            Exit => {
                self.exit = true;
                Ok(None)
            }

            LoginSuccessful => {
                self.status_widget.info = "Login successful".into();
                Ok(Some(SwitchScreen(Main)))
            }

            UiMessage::Logout => {
                clear().await?;
                Ok(Some(SwitchScreen(Auth)))
            }
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.current_screen {
            Main => {
                let area = frame.area();

                let layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Min(10),
                        Constraint::Length(3),
                    ])
                    .split(area);

                self.search_widget.render(frame, layout[0]);
                self.subs_widget.render(frame, layout[1]);
                self.status_widget.render(frame, layout[2]);
            }
            Language => {
                let area = frame.area();

                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3)])
                    .split(area);

                self.language_widget.render(frame, area);
            }
            Auth => {
                let area = frame.area();

                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)])
                    .split(area);

                self.login_widget.render(frame, area);
            }

            CurrentScreen::Logout => {
                let area = frame.area();

                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)])
                    .split(area);

                self.logged_in_widget.render(frame, area);
            }
        }
    }

    fn handle_key_event(&mut self, event: Event) -> Option<UiMessage> {
        if let Event::Key(key_event) = event {
            match self.current_screen {
                Main => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    KeyCode::F(2) => Some(SwitchScreen(Language)),
                    KeyCode::F(12) => Some(GoToLogin),
                    KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Enter => self.subs_widget.handle_key_event(key_event),
                    _ => self.search_widget.handle_key_event(event),
                },

                Language => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    QUIT_KEY => Some(SwitchScreen(Main)),
                    KeyCode::F(2) => Some(SwitchScreen(Main)),
                    _ => self.language_widget.handle_key_event(event),
                },

                Auth => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    QUIT_KEY => Some(SwitchScreen(Main)),
                    _ => self.login_widget.handle_key_event(event),
                },

                CurrentScreen::Logout => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    QUIT_KEY => Some(SwitchScreen(Main)),
                    _ => self.logged_in_widget.handle_key_event(event),
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
    Auth,
    Logout,
}
