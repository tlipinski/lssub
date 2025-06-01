use crate::ui::app::UiEvent::{Input, ResultsUpdate};
use crate::ui::events::UiEvent;
use crate::ui::features_fetcher::fetch_features_task;
use crate::ui::input_handler::handle_input_task;
use crate::ui::search_widget::SearchWidget;
use crate::ui::subs_widget::{Sub, SubsWidget};
use anyhow::Result;
use log::info;
use osb::subtitles::SubtitlesResponse;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::TableState;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::sync::mpsc;

pub const QUIT_KEY: KeyCode = KeyCode::Esc;

#[derive(Debug, Default)]
pub struct App {
    current_screen: CurrentScreen,
    search_widget: SearchWidget,
    subs: SubsWidget,
    exit: bool,
}

impl App {
    pub fn init(file_name: String) -> App {
        App {
            search_widget: SearchWidget {
                search_text: file_name,
                active: true,
            },
            ..App::default()
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (features_tx, features_rx) = mpsc::channel::<String>();

        tokio::spawn(fetch_features_task(features_rx, ui_tx.clone()));
        tokio::spawn(handle_input_task(ui_tx.clone()));

        if !self.search_widget.search_text.is_empty() {
            features_tx.send(self.search_widget.search_text.clone())?;
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match ui_rx.recv()? {
                Input(key_event) => {
                    // info!("Input: {:?}", key_event);
                    self.handle_key_event(key_event);
                    features_tx.send(self.search_widget.search_text.clone())?;
                }
                ResultsUpdate(subtitles) => {
                    // info!("ResultsUpdate: {:?}", subtitles);
                    self.handle_features_event(subtitles)?
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        frame.render_widget(&self.search_widget, layout[0]);
        frame.render_widget(&self.subs, layout[1]);
    }

    fn handle_features_event(&mut self, subtitles_response: SubtitlesResponse) -> Result<()> {
        let subs = subtitles_response
            .data
            .iter()
            .take(20)
            .map(|resp| Sub {
                title: resp.attributes.release.clone(),
                language: resp.attributes.language.clone(),
                upload_date: resp.attributes.upload_date.clone(),
            })
            .collect::<Vec<Sub>>();
        self.subs = SubsWidget {
            subs: subs,
            state: TableState::default().with_selected(0),
            active: false
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.current_screen {
            CurrentScreen::Main => match key_event.code {
                QUIT_KEY => self.exit(),
                KeyCode::Char('s') => {
                    self.current_screen = CurrentScreen::Searching;
                    self.search_widget.active = true
                }
                _ => {}
            },
            CurrentScreen::Searching => {
                self.search_widget.handle_key_event(key_event);
                if (!self.search_widget.active) {
                    self.current_screen = CurrentScreen::Main
                }
            }
            CurrentScreen::Table => {
                self.subs.handle_key_event(key_event);
                if (!self.subs.active) {
                    self.current_screen = CurrentScreen::Main
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
enum CurrentScreen {
    Main,
    #[default]
    Searching,
    Table,
}
