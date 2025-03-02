use anyhow::{Context, Result, bail};
use log::{debug, error};
use osb::features::features;
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

impl App {
    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let (tx, rx) = mpsc::channel::<String>();
        tokio::spawn(async move {
            loop {
                if let Ok(text) = rx.recv() {
                    features(&text).await;
                } else {
                    error!("Error while receiving")
                }
            }
        });
        let events_tx = tx.clone();
        while !self.exit {
            // println!("loop");
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&events_tx);
            // let z = rx.recv();
            // debug!("{:?}", z)
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_events(&mut self, tx: &Sender<String>) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event, &tx)
            }
            // .with_context(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }?;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent, tx: &Sender<String>) -> Result<()> {
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
                    tx.send(self.search_text.clone());
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
