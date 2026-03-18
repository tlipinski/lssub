use crate::osb::login::Credentials;
use crate::osb::subtitles::{Subtitle, SubtitlesRequest, SubtitlesResponse};
use crate::ui::app::Screen;
use crate::ui::downloader::Downloaded;
use crate::ui::search_widget::SubtitlesQuery;
use crate::ui::task_runner::Task;
use ratatui::crossterm::event::Event;
use std::path::PathBuf;
use crate::osb::user_info::User;

#[derive(Debug)]
pub enum Action {
    Init,
    Exit,
    Tick,
    NoOp,
    Multi(Vec<Action>),

    InputReceived(Event),
    SwitchScreen(Screen),

    SearchQueryUpdated(SubtitlesQuery),
    LanguagesUpdated(Vec<String>),
    FetchSubtitles(SubtitlesRequest),
    SubtitlesFetched(Vec<Subtitle>),
    SubtitleDownloaded(Downloaded),
    FeatureInfo(i64),

    UserLoggedIn(User),
    UserLoggedOut,

    ChangeStatus(String),

    RunTask(Task),
    StartProgress,
    StopProgress,
}
