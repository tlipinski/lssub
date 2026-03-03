use crate::config::{Config, ConfigProvider};
use crate::secret::{clear, retrieve, store};
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    DisabledLimitSubsToId, DownloadSubs, DownloadSubsFailed, DownloadedSubs, EnabledLimitSubsToId,
    Exit, FetchSubs, SwitchToAccountScreen, Init, LanguagesUpdated, Login, LoginFailed, Logout,
    QueryUpdated, SpinnerUpdate, StartSpinner, StopSpinner, SwitchScreen, UpdateDownloadCount,
    UpdateUser, UpdateUsername,
};
use crate::ui::app::Action::{Input, SubsFetched};
use crate::ui::app::CurrentScreen::{Account, Auth, Language, Main};
use crate::ui::downloader::Downloader;
use crate::ui::input_handler::handle_input_task;
use crate::ui::language_widget::LanguageWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::spinner_task;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use crate::ui::user_widget::UserWidget;
use anyhow::{Error, Result, bail};
use clap::builder::TypedValueParser;
use gio::prelude::DBusInterfaceSkeletonExt;
use log::{error, info};
use osb::get_download_link::get_download_link;
use osb::login::login;
use osb::user_info;
use osb::user_info::get_user_info;
use ratatui::crossterm::event::{Event, KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{StatefulWidget, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use std::collections::VecDeque;
use std::ops::Deref;
use std::path::Path;
use std::sync::mpsc;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

pub struct App {
    config_provider: ConfigProvider,
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    user_widget: UserWidget,
    subs_widget: SubsWidget,
    language_widget: LanguageWidget,
    status_widget: StatusWidget,
    login_widget: LoginWidget,
    account_widget: AccountWidget,
    ui_tx: Sender<Action>,
    features_tx: Sender<SubtitlesQuery>,
    downloader: Downloader,
    exit: bool,
}

impl App {
    pub async fn run(
        terminal: &mut DefaultTerminal,
        base_path: &Path,
        file_name: Option<&str>,
    ) -> Result<()> {
        let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<Action>(100);
        let (features_tx, features_rx) = tokio::sync::mpsc::channel::<SubtitlesQuery>(100);

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(spinner_task(ui_tx.clone()));

        let provider = ConfigProvider::default();
        let languages = provider.get_config()?.languages;
        let mut app = App {
            config_provider: provider,
            current_screen: CurrentScreen::default(),
            search_widget: SearchWidget::from(file_name.unwrap_or("").into()),
            user_widget: UserWidget::from(),
            subs_widget: SubsWidget::default(),
            language_widget: LanguageWidget::from(languages),
            status_widget: StatusWidget::from("".into()),
            login_widget: LoginWidget::from(),
            account_widget: AccountWidget::from(),
            downloader: Downloader::new(base_path.to_owned(), file_name.map(String::from)),
            ui_tx: ui_tx.clone(),
            features_tx,
            exit: false,
        };

        let mut messages = VecDeque::from([Init, UpdateUser]);

        while !app.exit {
            while let Some(msg) = messages.pop_front() {
                match app.update(msg).await {
                    Ok(next) => messages.extend(next),
                    Err(e) => {
                        error!("Error while updating UI: {e}");
                    }
                };
            }

            terminal.draw(|frame| app.draw(frame))?;

            messages.extend(ui_rx.recv().await);
        }

        shutdown_tx.send(())?;

        Ok(())
    }

    async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        match action {
            Input(event) => {
                if let Some(m) = self.handle_key_event(event) {
                    Ok(vec![m])
                } else {
                    Ok(vec![])
                }
            }

            SubsFetched(subtitles) => {
                self.subs_widget.update_subtitles(&subtitles);
                self.status_widget.info = format!("{} results", subtitles.data.len());
                Ok(vec![StopSpinner])
            }

            SpinnerUpdate(chr) => {
                self.status_widget.spin(chr);
                Ok(vec![])
            }

            LanguagesUpdated(languages) => {
                self.current_screen = Main;
                let query: String = self.search_widget.input.value().into();
                self.config_provider.modify(|c: &Config| {
                    let mut updated = c.clone();
                    updated.languages = languages.clone();
                    updated
                });
                Ok(vec![FetchSubs(query, languages)])
            }

            QueryUpdated(query) => {
                let languages = self.language_widget.languages();
                Ok(vec![FetchSubs(query, languages)])
            }

            FetchSubs(query, languages) => {
                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: None,
                    })
                    .await?;
                Ok(vec![StartSpinner])
            }

            StartSpinner => {
                self.status_widget.spinning = true;
                Ok(vec![])
            }

            StopSpinner => {
                self.status_widget.spinning = false;
                Ok(vec![])
            }

            Init => {
                let query: String = self.search_widget.input.value().into();
                if (!query.is_empty()) {
                    let languages = self.language_widget.languages();
                    Ok(vec![FetchSubs(query, languages)])
                } else {
                    Ok(vec![])
                }
            }

            DownloadSubs(file_id, language) => {
                self.status_widget.info = "Downloading...".into();

                let downloader = self.downloader.clone();
                let ui_tx = self.ui_tx.clone();
                tokio::spawn(async move {
                    let token_result = retrieve().await;
                    match token_result {
                        Ok(token_opt) => {
                            let msg = match downloader.download(token_opt, file_id, &language).await {
                                Ok(downloaded) => DownloadedSubs(downloaded),
                                Err(e) => DownloadSubsFailed(e.to_string()),
                            };
                            ui_tx.send(msg).await;
                        }
                        Err(e) => {
                            let msg = DownloadSubsFailed(e.to_string());
                            ui_tx.send(msg).await;
                        }
                    }
                });

                Ok(vec![StartSpinner])
            }

            DownloadedSubs(downloaded) => {
                self.status_widget.info = format!("Downloaded: {:?}", downloaded.path);
                self.user_widget.requests = downloaded.requests;
                self.user_widget.remaining = downloaded.remaining;
                Ok(vec![StopSpinner])
            }

            SwitchScreen(screen) => {
                self.current_screen = screen;
                Ok(vec![])
            }

            SwitchToAccountScreen => {
                let result = retrieve().await;
                match result {
                    Ok(Some(token)) => Ok(vec![SwitchScreen(Account)]),
                    Ok(None) => Ok(vec![SwitchScreen(Auth)]),
                    Err(e) => {
                        error!("Failed to retrieve token: {e}");
                        Ok(vec![])
                    }
                }
            }

            Login(credentials) => {
                let result = tokio::spawn(async move {
                    match login(&credentials).await {
                        Ok(api_token) => {
                            store(&api_token, &credentials.username).await;
                            UpdateUser
                        }
                        Err(e) => LoginFailed(e.to_string()),
                    }
                })
                .await;

                match result {
                    Ok(msg) => Ok(vec![msg]),
                    Err(e) => {
                        error!("Error logging in: {}", e);
                        Err(e.into())
                    }
                }
            }

            LoginFailed(reason) => {
                self.login_widget.failed = reason;
                Ok(vec![])
            }

            DownloadSubsFailed(error) => {
                self.status_widget.info = format!("Error: {:?}", error);
                Ok(vec![StopSpinner])
            }

            Exit => {
                self.exit = true;
                Ok(vec![])
            }

            UpdateUser => {
                let ui_tx = self.ui_tx.clone();
                tokio::spawn(async move {
                    match retrieve().await {
                        Ok(Some(jwt)) => match get_user_info(&jwt).await {
                            Ok(user_info) => {
                                ui_tx
                                    .send(UpdateDownloadCount(
                                        user_info.data.downloads_count,
                                        user_info.data.remaining_downloads,
                                    ))
                                    .await;

                                ui_tx.send(UpdateUsername(user_info.data.username)).await
                            }
                            Err(e) => {
                                error!("Error getting user info: {e}");
                                Ok(())
                            }
                        },
                        Ok(None) => {
                            ui_tx.send(UpdateDownloadCount(0, 0)).await;
                            ui_tx.send(UpdateUsername("".to_string())).await;
                            Ok(())
                        }
                        Err(e) => {
                            error!("Error retrieving jwt: {e}");
                            Ok(())
                        }
                    }
                });
                Ok(vec![SwitchScreen(Main)])
            }

            Logout => {
                clear().await?;
                Ok(vec![UpdateUser, SwitchScreen(Auth)])
            }

            UpdateDownloadCount(rq, rm) => {
                self.user_widget.requests = rq;
                self.user_widget.remaining = rm;
                Ok(vec![])
            }

            UpdateUsername(username) => {
                self.user_widget.username = username;
                Ok(vec![])
            }

            EnabledLimitSubsToId(id) => {
                let languages = self.language_widget.languages();
                let query = self.search_widget.input.value().into();
                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: Some(id),
                    })
                    .await?;
                Ok(vec![StartSpinner])
            }

            DisabledLimitSubsToId => {
                let languages = self.language_widget.languages();
                let query = self.search_widget.input.value().into();
                self.features_tx
                    .send(SubtitlesQuery {
                        query,
                        languages,
                        id: None,
                    })
                    .await?;
                Ok(vec![StartSpinner])
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
                        Constraint::Length(3),
                    ])
                    .split(area);

                let status = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Fill(1), Constraint::Length(50)])
                    .split(layout[2]);

                let main_nav = {
                    Paragraph::new(Line::from(vec![
                        Span::from("F2:").bold(),
                        Span::from(" Languages | "),
                        Span::from("F10:").bold(),
                        Span::from(" Exit | "),
                        Span::from("F12:").bold(),
                        Span::from(" Account"),
                    ]))
                    .block(Block::default().borders(Borders::ALL))
                };

                self.search_widget.render(frame, layout[0]);
                self.subs_widget.render(frame, layout[1]);
                self.status_widget.render(frame, status[0]);
                self.user_widget.render(frame, status[1]);
                frame.render_widget(main_nav, layout[3]);
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

            CurrentScreen::Account => {
                let area = frame.area();

                let right = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Length(3), Constraint::Length(3)])
                    .split(area);

                self.account_widget.render(frame, area);
            }
        }
    }

    fn handle_key_event(&mut self, event: Event) -> Option<Action> {
        if let Event::Key(key_event) = event {
            match self.current_screen {
                Main => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    KeyCode::F(2) => Some(SwitchScreen(Language)),
                    KeyCode::F(12) => Some(SwitchToAccountScreen),
                    KeyCode::PageUp
                    | KeyCode::PageDown
                    | KeyCode::Up
                    | KeyCode::Down
                    | KeyCode::Enter
                    | KeyCode::F(5) => self.subs_widget.handle_key_event(key_event),
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

                CurrentScreen::Account => match key_event.code {
                    KeyCode::F(10) => Some(Exit),
                    QUIT_KEY => Some(SwitchScreen(Main)),
                    _ => self.account_widget.handle_key_event(event),
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
    Account,
}
