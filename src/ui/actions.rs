use crate::osb::languages::Language;
use crate::osb::subtitles::Subtitle;
use crate::osb::user_info::User;
use crate::ui::downloader::Downloaded;
use crate::ui::search_widget::SubtitlesQuery;
use crate::ui::task_runner::Task;
use ratatui::crossterm::event::Event;
use crate::ui::main_widget::Screen;
use crate::ui::subs_list_widget::QueryParams;

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
    SearchQueryUpdated(String),
    SearchParamsUpdated(QueryParams),
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
