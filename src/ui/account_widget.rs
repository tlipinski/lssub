use crate::osb::login::login;
use crate::osb::user_info::{UserInfo, get_user_info};
use crate::secret::{clear, retrieve, store};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{
    ChangeStatus, ReceivedInput, SwitchScreen, UserLoggedIn, UserLoggedOut,
};
use crate::ui::component::Component;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::task_runner::{Task, TaskRunner};
use anyhow::{Error, Result};
use crossterm::event::Event;
use log::{error, info};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::ToSpan;

pub struct AccountWidget {
    login_widget: LoginWidget,
    logged_in_widget: LoggedInWidget,
    task_runner: TaskRunner,
    logged_in: bool,
}

impl Component for AccountWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Action::Init => {
                self.refresh(); // todo side effect
            }
            UserLoggedIn(user_info) => {
                self.logged_in = true;
                self.logged_in_widget.user_info = user_info.clone();
            }
            UserLoggedOut => {
                self.logged_in = false;
                self.logged_in_widget.user_info = UserInfo::default();
            }
            _ => {}
        }

        None
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

    pub fn refresh(&mut self) {
        self.task_runner.run(Task::new(async move {
            match retrieve().await {
                Ok(Some(jwt)) => match get_user_info(&jwt).await {
                    Ok(user_info) => Ok(UserLoggedIn(user_info)),
                    Err(e) => {
                        error!("Error getting user info: {e}");
                        Err(Error::msg(e.to_string()))
                    }
                },
                Ok(None) => Ok(ChangeStatus("".into())), // todo
                Err(e) => {
                    error!("Error retrieving jwt: {e}");
                    Err(Error::msg(e.to_string()))
                }
            }
        }));
    }
}
