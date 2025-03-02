use crate::app::UiEvent::{Input, ResultsUpdate};
use anyhow::{Context, Result, bail};
use log::{debug, error, info};
use osb::features::{FeaturesResponse, features};
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
use std::sync::mpsc::{Sender, TryRecvError};
use std::time::Duration;
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
    ResultsUpdate(FeaturesResponse),
}

impl App {
    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let (ui_tx, ui_rx) = mpsc::channel::<UiEvent>();
        let (features_tx, features_rx) = mpsc::channel::<String>();

        let result_update_tx = ui_tx.clone();
        tokio::spawn(async move {
            'outer: loop {
                sleep(Duration::from_millis(500)).await;
                let mut last = String::from("");
                'debouncing: loop {
                    match features_rx.try_recv() {
                        Ok(ev) => {
                            debug!("Debouncing: {}", ev);
                            last = ev
                        }
                        Err(TryRecvError::Empty) => break 'debouncing,
                        Err(TryRecvError::Disconnected) => {
                            error!("Disconnected");
                            break 'outer;
                        }
                    }
                }
                if !last.is_empty() {
                    let result = features(&last).await;
                    match result {
                        Ok(features) => result_update_tx.send(ResultsUpdate(features)).unwrap(),
                        Err(_) => break,
                    }
                }
            }
        });

        let key_event_tx = ui_tx.clone();
        tokio::spawn(async move {
            loop {
                match event::read().unwrap() {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        key_event_tx.send(Input(key_event))
                    }
                    _ => break,
                };
            }
        });

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match ui_rx.recv()? {
                Input(key_event) => {
                    info!("Input: {:?}", key_event);
                    self.handle_key_event(key_event);
                    if (!self.search_text.is_empty()) {
                        features_tx.send(self.search_text.clone()).unwrap();
                    } else {
                        // ui_tx
                        //     .send(ResultsUpdate(FeaturesResponse { data: vec![] }))
                        //     .unwrap();
                        // todo blinks with single char search_text
                        self.handle_features_event(FeaturesResponse { data: vec![] })?
                    }
                }
                ResultsUpdate(features) => {
                    // info!("ResultsUpdate: {:?}", features);
                    self.handle_features_event(features)?
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_features_event(&mut self, features_response: FeaturesResponse) -> Result<()> {
        let subs = features_response
            .data
            .iter()
            .take(20)
            .map(|resp| Sub {
                id: resp.id.clone(),
                title: resp.attributes.title.clone(),
                year: resp.attributes.year.clone(),
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
    #[default]
    Main,
    Searching,
}

#[derive(Debug, Default)]
struct Subs(Vec<Sub>);

#[derive(Debug, Default)]
struct Sub {
    id: String,
    title: String,
    year: String,
}

impl Widget for &Subs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.0.iter().map(|item| {
            let cell = Cell::from(Text::from(item.id.as_str()));
            let cell2 = Cell::from(Text::from(item.title.as_str()));
            let cell3 = Cell::from(Text::from(item.year.as_str()));
            let vec1: Vec<Cell> = vec![cell, cell2, cell3];
            Row::from_iter(vec1)
        });

        let block_bot = Block::bordered()
            .title(format!(" Results: {} ", self.0.len()).bold())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Table::new(rows, [10, 50, 10])
            .block(block_bot)
            .render(area, buf);
    }
}
