use crossterm::event::Event;
use log::error;
use ratatui::Frame;
use ratatui::layout::Rect;
use osb::login::login;
use crate::secret::{clear, retrieve, store};
use crate::ui::account_widget::AccountWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{Input, LoggedIn, LoggedOut, SwitchScreen, UpdateDownloadCount, UpdateUser, UpdateUsername};
use crate::ui::login_widget::LoginWidget;
use anyhow::Result;
use osb::user_info::get_user_info;
use crate::ui::app::CurrentScreen::Main;

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

    pub async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        match action {
            Action::Init => {
                let result = tokio::spawn(async move {
                    match retrieve().await {
                        Ok(Some(jwt)) => match get_user_info(&jwt).await {
                            Ok(user_info) => {
                                Ok(Some(user_info))
                            }
                            Err(e) => {
                                error!("Error getting user info: {e}");
                                Err(e)
                            }
                        },
                        Ok(None) => {
                            Ok(None)
                        }
                        Err(e) => {
                            error!("Error retrieving jwt: {e}");
                            Ok(None)
                        }
                    }
                }).await?;

                match result {
                    Ok(Some(user_info)) => {
                        self.logged_in = true;
                        Ok(vec![LoggedIn])
                    }
                    Ok(None) => {
                        self.logged_in = false;
                        Ok(vec![LoggedOut])
                    }
                    Err(_) => {
                        self.logged_in = false;
                        Ok(vec![LoggedOut])
                    }
                }
            }

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
            self.account_widget.handle_key_event(event).await
        } else {
            self.login_widget.handle_key_event(event).await
        }
    }
}