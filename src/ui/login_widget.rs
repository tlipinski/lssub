use crate::ui::ui_messages::UiMessage;
use ratatui::Frame;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Line, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug)]
pub struct LoginWidget {
    pub username: Input,
    pub password: Input,
}

impl LoginWidget {
    pub fn from() -> Self {
        LoginWidget {
            username: Input::new("user".into()),
            password: Input::new("pass".into()),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(" Login ".to_string().bold())
            .border_set(border::THICK);

        let user_block = Block::bordered()
            .title(" Username ")
            .border_set(border::ROUNDED);

        let pass_block = Block::bordered()
            .title(" Password ")
            .border_set(border::ROUNDED);

        let buttons_block = Block::default().title(
            Line::from("OK [Enter] Cancel [Esc]").right_aligned()
        );

        let user_par = Paragraph::new(Line::from(self.username.value())).block(user_block);
        let pass_par = Paragraph::new(Line::from(self.password.value())).block(pass_block);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Fill(1),
                Constraint::Fill(1),
                Constraint::Fill(1),
            ])
            .split(area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(block.inner(outer_layout[1]));

        frame.render_widget(block, area);

        frame.render_widget(user_par, layout[0]);
        frame.render_widget(pass_par, layout[1]);
        frame.render_widget(buttons_block, layout[2]);

        frame.set_cursor_position((
            layout[0].x + (self.username.visual_cursor() + 1) as u16,
            area.y + 2,
        ));
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<UiMessage> {
        if let Event::Key(key_event) = event {
            match key_event.code {
                KeyCode::Enter => Some(UiMessage::LoggedIn),
                _ => {
                    self.username.handle_event(&event);
                    None
                }
            }
        } else {
            None
        }
    }
}
