use crate::ui::app::UiEvent::{Input, ResultsUpdate};
use crate::ui::subs_widget::{Sub, Subs};
use anyhow::{Context, Result, bail};
use log::{debug, error, info};
use osb::features::{FeaturesResponse, features};
use osb::subtitles::{SubtitlesResponse, subtitles};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::widgets::{Cell, Row, Table};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::time::Duration;
use clap::Subcommand;
use tokio::time::sleep;

#[derive(Debug, Default)]
pub struct App {
    current_screen: CurrentScreen,
    search_text: String,
    subs: Subs,
    exit: bool,
}

#[derive(Debug)]
enum UiEvent {
    Input(KeyEvent),
    ResultsUpdate(SubtitlesResponse),
}

impl App {
    fn exit(&mut self) {
        self.exit = true;
    }

    async fn features_fetch(rx: Receiver<String>, tx: Sender<UiEvent>) {
        'outer: loop {
            sleep(Duration::from_millis(1000)).await;

            let mut last: Option<String> = None;

            // Receive as much as possible within outer loop cycle to reduce OSB calls.
            'debouncing: loop {
                match rx.try_recv() {
                    Ok(ev) => {
                        // debug!("Debouncing: {}", ev);
                        last = Some(ev)
                    }

                    Err(TryRecvError::Empty) => break 'debouncing,

                    Err(TryRecvError::Disconnected) => {
                        error!("Disconnected");
                        break 'outer;
                    }
                }
            }

            if let Some(text) = last {
                if text.len() < 3 {
                    tx.send(ResultsUpdate(SubtitlesResponse { data: vec![] }))
                        .unwrap()
                } else {
                    let result = subtitles(&text, vec![String::from("pl")]).await;
                    match result {
                        Ok(subtitles) => tx.send(ResultsUpdate(subtitles)).unwrap(),
                        Err(_) => break,
                    }
                }
            }
        }
    }

    async fn input_handler(tx: Sender<UiEvent>) {
        loop {
            match event::read().unwrap() {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    tx.send(Input(key_event))
                }
                _ => break,
            };
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (features_tx, features_rx) = mpsc::channel::<String>();

        tokio::spawn(Self::features_fetch(features_rx, ui_tx.clone()));
        tokio::spawn(Self::input_handler(ui_tx.clone()));

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match ui_rx.recv()? {
                Input(key_event) => {
                    // info!("Input: {:?}", key_event);
                    self.handle_key_event(key_event);
                    features_tx.send(self.search_text.clone()).unwrap();
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
        frame.render_widget(self, frame.area())
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

        self.subs = Subs(subs);

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.current_screen {
            CurrentScreen::Main => match key_event.code {
                KeyCode::Esc => self.exit(),
                KeyCode::Char('s') => self.current_screen = CurrentScreen::Searching,
                _ => {}
            },
            CurrentScreen::Searching => match key_event.code {
                KeyCode::Backspace => {
                    self.search_text.pop();
                }
                KeyCode::Char(key) => {
                    self.search_text.push(key);
                }
                KeyCode::Esc => {
                    self.exit();
                }
                _ => {}
            },
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(10)])
            .split(area);

        let title = Line::from(" Search ".bold());
        let span = match self.current_screen {
            CurrentScreen::Main => " Search ".bold(),
            CurrentScreen::Searching => " Search ".bold().red(),
        };
        let block = Block::bordered()
            .title(span)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.search_text.clone().bold());

        Paragraph::new(par).block(block).render(layout[0], buf);

        self.subs.render(layout[1], buf);
    }
}

#[derive(Debug, Default)]
enum CurrentScreen {
    Main,
    #[default]
    Searching,
}
