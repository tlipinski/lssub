use crate::config::{Config, ConfigProvider};
use crate::secret::{clear, retrieve, store};
use crate::ui::account_screen::AccountScreen;
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    DisabledLimitSubsToId, DownloadSubs, DownloadSubsFailed, DownloadedSubs, EnabledLimitSubsToId,
    Exit, FetchSubs, Init, LanguagesUpdated, LoggedOut, QueryUpdated, SpinnerUpdate, StartSpinner,
    StopSpinner, SwitchScreen,
};
use crate::ui::app::Action::{Input, SubsFetched};
use crate::ui::app::CurrentScreen::{Account, Language, Main};
use crate::ui::downloader::Downloader;
use crate::ui::input_handler::handle_input_task;
use crate::ui::languages_screen::LanguagesScreen;
use crate::ui::login_widget::LoginWidget;
use crate::ui::search_screen::SearchScreen;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::spinner_task;
use crate::ui::status_widget::StatusWidget;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use crate::ui::user_widget::UserWidget;
use anyhow::{Error, Result, bail};
use clap::builder::TypedValueParser;
use gio::prelude::DBusInterfaceSkeletonExt;
use log::{debug, error, info};
use osb::get_download_link::get_download_link;
use osb::login::login;
use osb::user_info;
use osb::user_info::{UserInfo, get_user_info};
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
    current_screen: CurrentScreen,
    search_screen: SearchScreen,
    languages_screen: LanguagesScreen,
    account_screen: AccountScreen,
    ui_tx: Sender<Action>,
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
        let search_screen = SearchScreen::from(base_path, file_name, features_tx)?;

        let mut app = App {
            search_screen,
            current_screen: CurrentScreen::default(),
            languages_screen: LanguagesScreen::new(provider)?,
            account_screen: AccountScreen::new(),
            ui_tx: ui_tx.clone(),
            exit: false,
        };

        let mut messages = VecDeque::from([Init]);

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
        debug!("action: {:?}", action);
        match action {
            Input(event) => {
                if let Ok(Some(m)) = self.handle_key_event(event).await {
                    Ok(vec![m])
                } else {
                    Ok(vec![])
                }
            }

            SubsFetched(subtitles) => {
                self.search_screen.update_subtitles(&subtitles);
                Ok(vec![StopSpinner])
            }

            // SpinnerUpdate(chr) => {
            // self.status_widget.spin(chr);
            // Ok(vec![])
            // }

            // LanguagesUpdated => {
            // let languages = self.languages_screen.languages();
            // let query: String = self.search_widget.input.value().into();
            // Ok(vec![SwitchScreen(Main), FetchSubs(query, languages)])
            // }

            // Action::LoggedIn => {
            //     match self.account_screen.user_info() {
            //         Some(user_info) => {
            //             self.search_screen.user_widget.requests = user_info.data.downloads_count;
            //             self.search_screen.user_widget.remaining = user_info.data.remaining_downloads;
            //             self.search_screen.user_widget.username = user_info.data.username.clone();
            //         }
            //
            //         None => {}
            //     }
            //
            //     Ok(vec![SwitchScreen(Main)])
            // }

            // LoggedOut => {
            //     self.search_screen.user_widget.requests = 0;
            //     self.search_screen.user_widget.remaining = 0;
            //     self.search_screen.user_widget.username = "".into();
            //
            //     Ok(vec![])
            // }
            QueryUpdated(query) => {
                let languages = self.languages_screen.languages();
                Ok(vec![FetchSubs(query, languages)])
            }

            // FetchSubs(query, languages) => {
            //     self.features_tx
            //         .send(SubtitlesQuery {
            //             query,
            //             languages,
            //             id: None,
            //         })
            //         .await?;
            //     Ok(vec![StartSpinner])
            // }

            // StartSpinner => {
            //     self.search_screen.status_widget.spinning = true;
            //     Ok(vec![])
            // }
            //
            // StopSpinner => {
            //     self.search_screen.status_widget.spinning = false;
            //     Ok(vec![])
            // }
            Init => {
                let mut actions = self.account_screen.update(Init).await?;

                // let query: String = self.search_widget.input.value().into();
                // if !query.is_empty() {
                //     let languages = self.languages_screen.languages();
                //     actions.push(FetchSubs(query, languages));
                // }

                Ok(actions)
            }

            // DownloadSubs(file_id, language) => {
            //     self.status_widget.info = "Downloading...".into();
            //
            //     let downloader = self.downloader.clone();
            //     let ui_tx = self.ui_tx.clone();
            //     tokio::spawn(async move {
            //         let token_result = retrieve().await;
            //         match token_result {
            //             Ok(token_opt) => {
            //                 let msg = match downloader.download(token_opt, file_id, &language).await
            //                 {
            //                     Ok(downloaded) => DownloadedSubs(downloaded),
            //                     Err(e) => DownloadSubsFailed(e.to_string()),
            //                 };
            //                 ui_tx.send(msg).await;
            //             }
            //             Err(e) => {
            //                 let msg = DownloadSubsFailed(e.to_string());
            //                 ui_tx.send(msg).await;
            //             }
            //         }
            //     });
            //
            //     Ok(vec![StartSpinner])
            // }

            // DownloadedSubs(downloaded) => {
            //     self.status_widget.info = format!("Downloaded: {:?}", downloaded.path);
            //     self.user_widget.requests = downloaded.requests;
            //     self.user_widget.remaining = downloaded.remaining;
            //     Ok(vec![StopSpinner])
            // }
            SwitchScreen(screen) => {
                self.current_screen = screen;
                Ok(vec![])
            }

            // DownloadSubsFailed(error) => {
            //     self.status_widget.info = format!("Error: {:?}", error);
            //     Ok(vec![StopSpinner])
            // }
            Exit => {
                self.exit = true;
                Ok(vec![])
            }

            // EnabledLimitSubsToId(id) => {
            //     let languages = self.languages_screen.languages();
            //     let query = self.search_widget.input.value().into();
            //     self.features_tx
            //         .send(SubtitlesQuery {
            //             query,
            //             languages,
            //             id: Some(id),
            //         })
            //         .await?;
            //     Ok(vec![StartSpinner])
            // }

            // DisabledLimitSubsToId => {
            //     let languages = self.languages_screen.languages();
            //     let query = self.search_widget.input.value().into();
            //     self.features_tx
            //         .send(SubtitlesQuery {
            //             query,
            //             languages,
            //             id: None,
            //         })
            //         .await?;
            //     Ok(vec![StartSpinner])
            // }
            _ => Ok(vec![]),
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        let main_nav = {
            Paragraph::new(Line::from(vec![
                Span::from("F2:").bold(),
                Span::from(" Search | "),
                Span::from("F3:").bold(),
                Span::from(" Account | "),
                Span::from("F4:").bold(),
                Span::from(" Languages | "),
                Span::from("F10:").bold(),
                Span::from(" Exit"),
            ]))
            .centered()
            .block(Block::default().borders(Borders::ALL))
        };

        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Length(3)])
            .split(area);

        frame.render_widget(main_nav, layout[1]);

        match &self.current_screen {
            Main => {
                self.search_screen.render(frame, layout[0]);
            }
            Language => {
                self.languages_screen.render(frame, layout[0]);
            }
            Account => {
                self.account_screen.render(frame, layout[0]);
            }
        }
    }

    async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                QUIT_KEY => Ok(Some(SwitchScreen(Main))),
                KeyCode::F(2) => Ok(Some(SwitchScreen(Main))),
                KeyCode::F(3) => Ok(Some(SwitchScreen(Account))),
                KeyCode::F(4) => Ok(Some(SwitchScreen(Language))),
                KeyCode::F(10) => Ok(Some(Exit)),

                _ => match self.current_screen {
                    Main => match key_event.code {
                        _ => self.search_screen.handle_key_event(event).await,
                    },

                    Language => match key_event.code {
                        _ => self.languages_screen.handle_key_event(event),
                    },

                    Account => match key_event.code {
                        _ => self.account_screen.handle_key_event(event).await,
                    },
                },
            }
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default)]
pub enum CurrentScreen {
    #[default]
    Main,
    Account,
    Language,
}
