use crate::secret::{clear, retrieve, store};
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::actions::Action;
use crate::ui::actions::Action::{ReceivedInput, UserLoggedIn, UserLoggedOut, SwitchScreen};
use crate::ui::app::CurrentScreen::Main;
use crate::ui::login_widget::LoginWidget;
use anyhow::Result;
use crossterm::event::Event;
use log::{error, info};
use osb::login::login;
use osb::user_info::{UserInfo, get_user_info};
use ratatui::Frame;
use ratatui::layout::Rect;

pub struct AccountWidget {
    login_widget: LoginWidget,
    logged_in_widget: LoggedInWidget,
    logged_in: bool,
}

impl AccountWidget {
    pub fn new() -> Self {
        Self {
            login_widget: LoginWidget::from(),
            logged_in_widget: LoggedInWidget::from(UserInfo {
                data: Default::default(),
            }),
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

    pub async fn update(&mut self, action: Action) -> Result<Vec<Action>> {
        match action {
            Action::Init => {
                let result = tokio::spawn(async move {
                    match retrieve().await {
                        Ok(Some(jwt)) => match get_user_info(&jwt).await {
                            Ok(user_info) => Ok(Some(user_info)),
                            Err(e) => {
                                error!("Error getting user info: {e}");
                                Err(e)
                            }
                        },
                        Ok(None) => Ok(None),
                        Err(e) => {
                            error!("Error retrieving jwt: {e}");
                            Ok(None)
                        }
                    }
                })
                .await?;

                match result {
                    Ok(Some(user_info)) => {
                        self.logged_in = true;
                        self.logged_in_widget.user_info = user_info;
                        Ok(vec![UserLoggedIn])
                    }
                    Ok(None) => {
                        self.logged_in = false;
                        Ok(vec![UserLoggedOut])
                    }
                    Err(_) => {
                        self.logged_in = false;
                        Ok(vec![UserLoggedOut])
                    }
                }
            }

            _ => Ok(vec![]),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if (self.logged_in) {
            self.logged_in_widget.render(frame, area);
        } else {
            self.login_widget.render(frame, area);
        }
    }

    pub async fn handle_key_event(&mut self, event: Event) -> Result<Option<Action>> {
        if (self.logged_in) {
            match self.logged_in_widget.handle_key_event(event).await? {
                Some(UserLoggedOut) => {
                    self.logged_in = false;
                    Ok(Some(UserLoggedOut))
                }
                other => Ok(other),
            }
        } else {
            match self.login_widget.handle_key_event(event).await? {
                Some(UserLoggedIn) => {
                    self.logged_in = true;
                    self.update(Action::Init).await?; // todo
                    Ok(Some(UserLoggedIn))
                }
                other => Ok(other),
            }
        }
    }
}
