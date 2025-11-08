use crate::ui::app::UiEvent::{Input, ResultsUpdate};
use crate::ui::commands::UICommand;
use crate::ui::events::UiEvent;
use crate::ui::events::UiEvent::{FileSelected, SpinnerUpdate};
use crate::ui::explorer_widget::Explorer;
use crate::ui::input_handler::handle_input_task;
use crate::ui::language_widget::LanguageWidget;
use crate::ui::search_widget::SearchWidget;
use crate::ui::spinner::handle_spinner;
use crate::ui::subs_widget::SubsWidget;
use crate::ui::subtitles_fetcher::{SubtitlesQuery, subtitles_fetch_task};
use anyhow::Result;
use gio::glib::random_int_range;
use log::info;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::TableState;
use ratatui::{DefaultTerminal, Frame};
use serde::de::Unexpected::Str;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread::current;
use tokio::io::split;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Receiver;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

#[derive(Debug)]
pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    subs_widget: SubsWidget,
    language_widget: LanguageWidget,
    features_tx: Sender<SubtitlesQuery>,
    exit: bool,
}

impl App {
    pub fn run(terminal: &mut DefaultTerminal, file_name: String) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (features_tx, features_rx) = mpsc::channel::<SubtitlesQuery>();

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(subtitles_fetch_task(features_rx, ui_tx.clone()));
        tokio::spawn(handle_spinner(ui_tx.clone()));

        let mut app = App {
            current_screen: CurrentScreen::default(),
            search_widget: SearchWidget::from(file_name),
            subs_widget: SubsWidget::default(),
            language_widget: LanguageWidget::from(),
            features_tx,
            exit: false,
        };

        app.activate(CurrentScreen::default());

        while !app.exit {
            terminal.draw(|frame| app.draw(frame))?;
            let ui_event = ui_rx.recv()?;
            let cmd = app.handle_ui_events(ui_event);
            app.handle_command(cmd)?
        }
        shutdown_tx.send(())?;
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_ui_events(&mut self, ui_event: UiEvent) -> Option<UICommand> {
        match ui_event {
            Input(event) => {
                return self.handle_key_event(event);
            }
            ResultsUpdate(subtitles) => {
                // info!("ResultsUpdate: {:?}", subtitles);
                self.search_widget.spinning = false;
                self.subs_widget.update_subtitles(subtitles)
            }
            // FileSelected(name) => self.search_widget.set_input(name.as_str()),
            FileSelected(name) => {}
            SpinnerUpdate(chr) => self.search_widget.spin(chr),
        }
        None
    }

    fn draw(&mut self, frame: &mut Frame) {
        if (self.language_widget.active) {
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
        } else {
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
    }

    fn activate(&mut self, widget: CurrentScreen) {
        match widget {
            CurrentScreen::Main => {
                self.subs_widget.active = false;
                self.language_widget.active = false;

                self.search_widget.active = false;
            }
            CurrentScreen::Searching => {
                self.subs_widget.active = false;
                self.language_widget.active = false;

                self.search_widget.active = true;
            }
            CurrentScreen::Table => {
                self.search_widget.active = false;
                self.language_widget.active = false;

                self.subs_widget.active = true;
            }
            CurrentScreen::Language => {
                self.search_widget.active = false;
                self.subs_widget.active = false;

                self.language_widget.active = true;
            }
        }
    }

    fn activate_main(&mut self) {
        self.activate(CurrentScreen::Main)
    }

    fn handle_command(&mut self, command: Option<UICommand>) -> Result<()> {
        match command {
            Some(UICommand::QuerySubtitles(q)) => {
                let langs: String = self.language_widget.input.value().into();
                let v = langs.split(",").collect::<Vec<&str>>();
                let p = v.iter().map(|&x| String::from(x)).collect::<Vec<String>>();
                self.features_tx.send(SubtitlesQuery {
                    query: q,
                    languages: p,
                })?;
            }
            None => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, event: Event) -> Option<UICommand> {
        // info!("key {key_event:?}");
        // let scr = &self.current_screen;
        // info!("scr before {scr:?}");
        if let Event::Key(key_event) = event {
            match self.current_screen {
                CurrentScreen::Main => match key_event.code {
                    KeyCode::F(10) => self.exit(),
                    KeyCode::F(2) => {
                        self.current_screen = CurrentScreen::Language;
                        self.activate(CurrentScreen::Language);
                    }
                    KeyCode::Char('s') | KeyCode::Tab => {
                        self.current_screen = CurrentScreen::Searching;
                        self.activate(CurrentScreen::Searching);
                    }
                    _ => {}
                },
                CurrentScreen::Searching => match key_event.code {
                    QUIT_KEY => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::F(2) => {
                        self.current_screen = CurrentScreen::Language;
                        self.activate(CurrentScreen::Language);
                    }
                    KeyCode::Tab => {
                        self.current_screen = CurrentScreen::Table;
                        self.activate(CurrentScreen::Table);
                    }
                    _ => {
                        return self.search_widget.handle_key_event(event);
                    }
                },
                CurrentScreen::Table => match key_event.code {
                    QUIT_KEY => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::F(2) => {
                        self.current_screen = CurrentScreen::Language;
                        self.activate(CurrentScreen::Language);
                    }
                    KeyCode::Tab => {
                        self.current_screen = CurrentScreen::Searching;
                        self.activate(CurrentScreen::Searching);
                    }
                    _ => self.subs_widget.handle_key_event(key_event),
                },
                CurrentScreen::Language => match key_event.code {
                    QUIT_KEY => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::F(2) => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::Enter => {
                        self.current_screen = CurrentScreen::Searching;
                    }
                    _ => (self.language_widget).handle_key_event(event),
                },
            }
            // let scr = &self.current_screen;
            // info!("scr after {scr:?}");
            None // todo remove it and return explicitly from patterns
        } else {
            None
        }
    }
}

#[derive(Debug, Default)]
enum CurrentScreen {
    Main,
    #[default]
    Searching,
    Table,
    Language,
}
