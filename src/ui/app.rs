use crate::ui::app::UiEvent::{Input, ResultsUpdate};
use crate::ui::events::UiEvent;
use crate::ui::features_fetcher::fetch_features_task;
use crate::ui::input_handler::handle_input_task;
use crate::ui::search_widget::SearchWidget;
use crate::ui::subs_widget::SubsWidget;
use anyhow::Result;
use log::info;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::TableState;
use ratatui::{DefaultTerminal, Frame};
use std::sync::mpsc;
use std::thread::current;
use tokio::io::split;
use tokio::sync::broadcast;
use crate::ui::events::UiEvent::FileSelected;
use crate::ui::explorer_widget::Explorer;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

#[derive(Debug)]
pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    subs_widget: SubsWidget,
    explorer_widget: Explorer,
    exit: bool,
}

impl App {
    pub fn run(terminal: &mut DefaultTerminal, file_name: String) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (features_tx, features_rx) = mpsc::channel::<String>();

        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(16);

        tokio::spawn(handle_input_task(ui_tx.clone(), shutdown_tx.subscribe()));
        tokio::spawn(fetch_features_task(features_rx, ui_tx.clone()));

        let mut app = App {
            current_screen: CurrentScreen::default(),
            search_widget: SearchWidget::from(features_tx, file_name),
            subs_widget: SubsWidget::default(),
            explorer_widget: Explorer::new(ui_tx)?,
            exit: false,
        };

        app.activate(CurrentScreen::default());

        app.search_widget.init();

        while !app.exit {
            terminal.draw(|frame| app.draw(frame))?;
            let ui_event = ui_rx.recv()?;
            app.handle_ui_events(ui_event)?;
        }
        shutdown_tx.send(())?;
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_ui_events(&mut self, ui_event: UiEvent) -> Result<()> {
        match ui_event {
            Input(event) => {
                self.handle_key_event(event);
            }
            ResultsUpdate(subtitles) => {
                // info!("ResultsUpdate: {:?}", subtitles);
                self.subs_widget.update_subtitles(subtitles)
            }
            FileSelected(name) => {
                self.search_widget.set_input(name.as_str())
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(layout[1]);

        self.explorer_widget.render(frame, layout[0]);
        self.search_widget.render(frame, right[0]);
        self.subs_widget.render(frame, right[1]);
    }

    fn activate(&mut self, widget: CurrentScreen) {
        match widget {
            CurrentScreen::Main => {
                self.subs_widget.active = false;
                self.explorer_widget.active = false;
                self.search_widget.active = false;
            }
            CurrentScreen::Searching => {
                self.subs_widget.active = false;
                self.explorer_widget.active = false;
                self.search_widget.active = true;
            }
            CurrentScreen::Explorer => {
                self.subs_widget.active = false;
                self.explorer_widget.active = true;
                self.search_widget.active = false;
            }
            CurrentScreen::Table => {
                self.subs_widget.active = true;
                self.explorer_widget.active = false;
                self.search_widget.active = false;
            }
        }
    }

    fn activate_main(&mut self) {
        self.activate(CurrentScreen::Main)
    }

    fn handle_key_event(&mut self, event: Event) -> Result<()> {
        // info!("key {key_event:?}");
        // let scr = &self.current_screen;
        // info!("scr before {scr:?}");
        if let Event::Key(key_event) = event {
            match self.current_screen {
                CurrentScreen::Main => match key_event.code {
                    QUIT_KEY => self.exit(),
                    KeyCode::Char('s') => {
                        self.current_screen = CurrentScreen::Searching;
                        self.search_widget.active = true
                    }
                    _ => {}
                },
                CurrentScreen::Searching => match key_event.code {
                    QUIT_KEY => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::Tab => {
                        self.current_screen = CurrentScreen::Table;
                        self.activate(CurrentScreen::Table);
                    }
                    _ => {
                        self.search_widget.handle_key_event(event);
                    }
                },
                CurrentScreen::Explorer => match key_event.code {
                    QUIT_KEY => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::Tab => {
                        self.current_screen = CurrentScreen::Searching;
                        self.activate(CurrentScreen::Searching);
                    }
                    _ => self.explorer_widget.handle_key_event(event)?
                },
                CurrentScreen::Table => match key_event.code {
                    QUIT_KEY => {
                        self.current_screen = CurrentScreen::Main;
                        self.activate_main();
                    }
                    KeyCode::Tab => {
                        self.current_screen = CurrentScreen::Explorer;
                        self.activate(CurrentScreen::Explorer);
                    }
                    _ => self.subs_widget.handle_key_event(key_event),
                },
            }
            // let scr = &self.current_screen;
            // info!("scr after {scr:?}");
            Ok(())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Default)]
enum CurrentScreen {
    Main,
    Searching,
    #[default]
    Explorer,
    Table,
}
