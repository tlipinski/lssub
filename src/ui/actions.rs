use crate::osb::login::Credentials;
use crate::osb::subtitles::SubtitlesResponse;
use crate::ui::app::CurrentScreen;
use crate::ui::downloader::Downloaded;
use crate::ui::subtitles_fetcher::SubtitlesQuery;
use ratatui::crossterm::event::Event;
use std::path::PathBuf;
use crate::osb::user_info::UserInfo;

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
    SwitchScreen(CurrentScreen),
    EnabledLimitSubsToId(i64),
    DisabledLimitSubsToId,
    FeatureInfo(i64),
    ChangeStatus(String),
    Tuple(Box<Action>, Box<Action>),
    Exit,
}
