use crate::osb::login::Credentials;
use crate::osb::subtitles::SubtitlesResponse;
use crate::osb::user_info::UserInfo;
use crate::ui::app::Screen;
use crate::ui::downloader::Downloaded;
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use ratatui::crossterm::event::Event;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Action {
    ReceivedInput(Event),
    SubsFetched(SubtitlesResponse),
    LanguagesUpdated(Vec<String>),
    UserLoggedIn(UserInfo),
    UserLoggedOut,
    SearchQueryUpdated(String),
    FetchSubs(String, Vec<String>),
    Init,
    DownloadedSubs(Downloaded),
    SwitchScreen(Screen),
    EnabledLimitSubsToId(i64),
    DisabledLimitSubsToId,
    FeatureInfo(i64),
    ChangeStatus(String),
    Multi(Vec<Action>),
    StartProgress,
    StopProgress,
    Exit,
}
