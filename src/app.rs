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
use std::sync::mpsc::Sender;

#[derive(Debug, Default)]
pub struct App {
    current_screen: CurrentScreen,
    search_text: String,
    exit: bool,
}

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
        let (tx, rx) = mpsc::channel::<String>();
        
        let result_update_tx = ui_tx.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(text) = rx.recv() {
                    let result = features(&text).await;
                    match result {
                        Ok(features) => { result_update_tx.send(ResultsUpdate(features)).unwrap()},
                        Err(_) => {}
                    }
                } else {
                    error!("Error while receiving")
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
                    _ => {Ok(())}
                };
            }
        });
        
        let events_tx = ui_tx.clone();
        
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match ui_rx.recv()? {
                Input(key_event) => {
                    info!("Input: {:?}", key_event);
                    self.handle_key_event(key_event);
                    tx.send(self.search_text.clone()).unwrap();
                }
                ResultsUpdate(features) => {
                    info!("ResultsUpdate: {:?}", features)
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.current_screen {
            CurrentScreen::Main => match key_event.code {
                KeyCode::Char('q') => self.exit(),
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

        let sub1 = Sub {
            id: "1".to_string(),
            title: "title".into(),
        };
        let sub2 = Sub {
            id: "2".to_string(),
            title: "title".to_string(),
        };

        let subs = Subs(vec![sub1, sub2]);

        subs.render(layout[1], buf);
    }
}

#[derive(Debug, Default)]
enum CurrentScreen {
    #[default]
    Main,
    Searching,
}

struct Subs(Vec<Sub>);

struct Sub {
    id: String,
    title: String,
}

impl Widget for &Subs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.0.iter().map(|item| {
            let cell = Cell::from(Text::from(item.id.as_str()));
            let cell2 = Cell::from(Text::from(item.title.as_str()));
            let vec1: Vec<Cell> = vec![cell, cell2];
            Row::from_iter(vec1)
        });

        let block_bot = Block::bordered()
            .title(format!(" Results: {} ", self.0.len()).bold())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        Table::new(rows, [10, 10])
            .block(block_bot)
            .render(area, buf);
    }
}
