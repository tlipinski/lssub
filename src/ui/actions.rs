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
    Init,
    Exit,
    Tick,
    Multi(Vec<Action>),

    InputReceived(Event),
    SwitchScreen(Screen),

    SearchQueryUpdated(SubtitlesQuery),
    LanguagesUpdated(Vec<String>),
    FetchSubtitles(SubtitlesRequest),
    SubtitlesFetched(SubtitlesResponse),
    SubtitleDownloaded(Downloaded),
    FeatureInfo(i64),

    UserLoggedIn(UserInfo),
    UserLoggedOut,

    ChangeStatus(String),

    RunTask(Task),
    StartProgress,
    StopProgress,
}
