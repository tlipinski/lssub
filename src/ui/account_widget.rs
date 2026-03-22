use crate::osb::login::login;
use crate::osb::osb_client::OsbClient;
use crate::osb::user_info::{User, get_user_info};
use crate::secret::{retrieve_credentials, retrieve_token, store_token};
use crate::ui::actions::Action;
use crate::ui::actions::Action::{NoOp, RunTask, UserLoggedIn, UserLoggedOut};
use crate::ui::component::Component;
use crate::ui::logged_in_widget::LoggedInWidget;
use crate::ui::login_widget::LoginWidget;
use crate::ui::task_runner::Task;
use Action::Init;
use anyhow::Error;
use crossterm::event::Event;
use log::{error, info, warn};
use ratatui::Frame;
use ratatui::layout::Rect;

pub struct AccountWidget {
    login_widget: LoginWidget,
    logged_in_widget: LoggedInWidget,
    logged_in: bool,
}

impl Component for AccountWidget {
    fn update(&mut self, action: &Action) -> Option<Action> {
        match action {
            Init => Some(RunTask(Task::new("init account", async move {
                let c = retrieve_credentials().await;
                warn!("Credentials retrieved: {:?}", c);

                match retrieve_token().await {
                    Ok(Some(jwt)) => match get_user_info(OsbClient::default(), &jwt).await {
                        Ok(user) => Ok(UserLoggedIn(user)),
                        Err(e) => {
                            error!("Error getting user info: {e}");
                            info!("Refreshing token");
                            let credentials_opt = retrieve_credentials().await?;
                            match credentials_opt {
                                None => Ok(NoOp),
                                Some(credentials) => {
                                    let client = OsbClient::default();
                                    let token = login(client, &credentials).await?;
                                    store_token(&token).await?;
                                    info!("Token refreshed");

                                    let user = get_user_info(OsbClient::default(), &token).await?;
                                    Ok(UserLoggedIn(user))
                                }
                            }
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
        if self.logged_in {
            self.logged_in_widget.handle_key_event(event)
        } else {
            self.login_widget.handle_key_event(event)
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.logged_in {
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
            logged_in_widget: LoggedInWidget::from(User::default()),
            logged_in: false,
        }
    }
}
