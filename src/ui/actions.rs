use crate::osb::languages::Language;
use crate::osb::subtitles::Subtitle;
use crate::osb::user_info::User;
use crate::ui::app::Screen;
use crate::ui::downloader::Downloaded;
use crate::ui::search_widget::SubtitlesQuery;
use crate::ui::task_runner::Task;
use ratatui::crossterm::event::Event;

#[derive(Debug)]
pub enum Action {
    Init,
    Exit,
    Tick,
    NoOp,
    Multi(Vec<Action>),

    InputReceived(Event),
    SwitchScreen(Screen),

    SearchInitialized(SubtitlesQuery),
    SearchQueryUpdated(SubtitlesQuery),
    FetchSubtitles,
    SubtitlesFetched(Vec<Subtitle>),
    SubtitleDownloaded(Downloaded),

    LanguagesUpdated(Vec<String>),
    LanguagesInitialized(Vec<String>),
    LanguagesFetched(Vec<Language>),
    UserLanguagesFetched(Vec<String>),

    UserLoggedIn(User),
    UserLoggedOut,

    ChangeStatus(String),

    RunTask(Task),
    StartProgress,
    StopProgress,
}
