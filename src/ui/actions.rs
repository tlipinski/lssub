use crate::osb::login::Credentials;
use crate::osb::subtitles::{SubtitlesRequest, SubtitlesResponse};
use crate::osb::user_info::UserInfo;
use crate::ui::app::Screen;
use crate::ui::downloader::Downloaded;
use crate::ui::search_widget::SubtitlesQuery;
use crate::ui::task_runner::Task;
use ratatui::crossterm::event::Event;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Action {
    ReceivedInput(Event),
    SubsFetched(SubtitlesResponse),
    LanguagesUpdated(Vec<String>),
    UserLoggedIn(UserInfo),
    UserLoggedOut,
    SearchQueryUpdated(SubtitlesQuery),
    FetchSubs(SubtitlesRequest),
    Init,
    DownloadedSubs(Downloaded),
    SwitchScreen(Screen),
    FeatureInfo(i64),
    ChangeStatus(String),
    Multi(Vec<Action>),
    StartProgress,
    StopProgress,
    RunTask(Task),
    Tick,
    Exit,
}
