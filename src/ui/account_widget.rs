use crate::osb::login::login;
use crate::osb::user_info::{User, get_user_info};
use crate::secret::{clear, retrieve, retrieve_credentials, store_token};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, InputReceived, NoOp, RunTask, SwitchScreen, UserLoggedIn, UserLoggedOut,
};
use crate::ui::component::Component;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::task_runner::{Task, TaskRunner};
use Action::Init;
use anyhow::{Error, Result};
use crossterm::event::Event;
use log::{error, info, warn};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::ToSpan;

pub struct AccountWidget {
    login_widget: LoginWidget,
    logged_in_widget: LoggedInWidget,
    logged_in: bool,
}

impl Component for AccountWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Init => Some(RunTask(Task::new("init account", async move {
                retrieve_credentials().await;

                match retrieve().await {
                    Ok(Some(jwt)) => match get_user_info(&jwt).await {
                        Ok(user) => Ok(UserLoggedIn(user)),
                        Err(e) => {
                            error!("Error getting user info: {e}");
                            // todo refresh token
                            // warn!("Logging out because token might have expired");
                            // clear().await;
                            Ok(ChangeStatus(e.to_string()))
                        }
                    },
                    Ok(None) => {
                        info!("User token not found");
                        Ok(NoOp)
                    }
                    Err(e) => {
                        error!("Error retrieving user token: {e}");
                        Err(Error::msg(e.to_string()))
                    }
                }
            }))),
            UserLoggedIn(user) => {
                self.logged_in = true;
                self.logged_in_widget.user = user.clone();
                None
            }
            UserLoggedOut => {
                self.logged_in = false;
                self.logged_in_widget.user = User::default();
                None
            }
            _ => None,
        }
    }

    fn handle_key_event(&mut self, event: &Event) -> Option<Action> {
        if (self.logged_in) {
            self.logged_in_widget.handle_key_event(event)
        } else {
            self.login_widget.handle_key_event(event)
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if (self.logged_in) {
            self.logged_in_widget.render(frame, area);
        } else {
            self.login_widget.render(frame, area);
        }
    }
}

impl AccountWidget {
    pub fn new() -> Self {
        Self {
            login_widget: LoginWidget::from(),
            logged_in_widget: LoggedInWidget::from(Default::default()),
            logged_in: false,
        }
    }

    pub fn user_info(&self) -> Option<User> {
        if self.logged_in {
            Some(self.logged_in_widget.user.clone())
        } else {
            None
        }
    }
}
