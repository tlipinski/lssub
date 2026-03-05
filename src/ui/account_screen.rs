use crossterm::event::Event;
use log::error;
use ratatui::Frame;
use ratatui::layout::Rect;
use osb::login::login;
use crate::secret::{clear, store};
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{Input, Logout, SwitchScreen, UpdateUser};
use crate::ui::login_widget::LoginWidget;
use anyhow::Result;

pub struct AccountScreen {
    login_widget: LoginWidget,
    account_widget: AccountWidget,
    logged_in: bool,
}

impl AccountScreen {

    pub fn new() -> Self {
        Self {
            login_widget: LoginWidget::from(),
            account_widget: AccountWidget::from(),
            logged_in: false,
        }
    }

    async fn update(&mut self, action: Action) -> anyhow::Result<Vec<Action>> {
        match action {
            // todo: handle Input action or let main app call handle_key_event?

            _ => Ok(vec![]),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if (self.logged_in) {
            self.account_widget.render(frame, area);
        } else {
            self.login_widget.render(frame, area);
        }
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if (self.logged_in) {
            self.account_widget.handle_key_event(event)
        } else {
            self.login_widget.handle_key_event(event).await
        }
    }
}