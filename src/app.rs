use anyhow::{Context, Result, bail};
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

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
        while !self.exit {
            // println!("loop");
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area())
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            // .with_context(|| format!("handling key event failed:\n{key_event:#?}")),
            _ => Ok(()),
        }?;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.current_screen {
            CurrentScreen::Main => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('s') => self.current_screen = CurrentScreen::Searching,
                    _ => {}
                }
            }
            CurrentScreen::Searching => {
                match key_event.code {
                    KeyCode::Backspace => {
                        self.search_text.pop();
                    },
                    KeyCode::Char(key) => {
                        self.search_text.push(key);
                    }
                    _ => {}
                }
            }
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

        let title = Line::from("Search".bold());
        let span = match self.current_screen {
            CurrentScreen::Main => "Search".bold(),
            CurrentScreen::Searching => "Search".bold().red(),
        };
        let block = Block::bordered()
            .title(span)
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let block_bot = Block::bordered()
            .title("Results".bold())
            // .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let par = Line::from(self.search_text.clone().bold());

        Paragraph::new(par).block(block).render(layout[0], buf);

        Paragraph::new("ar")
            .centered()
            .block(block_bot)
            .render(layout[1], buf);
    }
}

#[derive(Debug, Default)]
enum CurrentScreen {
    #[default]
    Main,
    Searching,
}
