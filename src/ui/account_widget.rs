use crate::osb::login::login;
use crate::osb::user_info::{UserInfo, get_user_info};
use crate::secret::{clear, retrieve, store};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, ReceivedInput, SwitchScreen, UserLoggedIn, UserLoggedOut,
};
use crate::ui::app::CurrentScreen::Search;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::task_runner::TaskRunner;
use anyhow::Result;
use crossterm::event::Event;
use log::{error, info};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::ToSpan;

pub struct AccountWidget {
    login_widget: LoginWidget,
    pub logged_in_widget: LoggedInWidget,
    task_runner: TaskRunner,
    pub logged_in: bool,
}

impl AccountWidget {
    pub fn new(task_runner: TaskRunner) -> Self {
        Self {
            login_widget: LoginWidget::from(task_runner.clone()),
            logged_in_widget: LoggedInWidget::from(
                UserInfo {
                    data: Default::default(),
                },
                task_runner.clone(),
            ),
            task_runner,
            logged_in: false,
        }
    }

    pub fn user_info(&self) -> Option<UserInfo> {
        if self.logged_in {
            Some(self.logged_in_widget.user_info.clone())
        } else {
            None
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if (self.logged_in) {
            self.logged_in_widget.render(frame, area);
        } else {
            self.login_widget.render(frame, area);
        }
    }

    pub fn handle_key_event(&mut self, event: Event) -> Option<Action> {
        if (self.logged_in) {
            match self.logged_in_widget.handle_key_event(event) {
                Some(UserLoggedOut) => {
                    self.logged_in = false;
                    Some(UserLoggedOut)
                }
                other => other,
            }
        } else {
            self.login_widget.handle_key_event(event)
        }
    }

    pub fn refresh(&mut self) {
        self.task_runner.run(async move {
            match retrieve().await {
                Ok(Some(jwt)) => match get_user_info(&jwt).await {
                    Ok(user_info) => Ok(UserLoggedIn(user_info)),
                    Err(e) => {
                        error!("Error getting user info: {e}");
                        Ok(ChangeStatus(e.to_string()))
                    }
                },
                Ok(None) => Ok(ChangeStatus("".into())), // todo
                Err(e) => {
                    error!("Error retrieving jwt: {e}");
                    Ok(ChangeStatus(e.to_string()))
                }
            }
        });
    }
}
