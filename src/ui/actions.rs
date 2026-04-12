use crate::osb::languages::Language;
use crate::osb::subtitles::Subtitle;
use crate::osb::user_info::User;
use crate::ui::downloader::Downloaded;
use crate::ui::main_widget::Screen;
use crate::ui::task_runner::Task;

use std::fmt::{Debug, Formatter};

pub enum Action {
    Init,
    Exit,
    Tick,
    NoOp,
    Multi(Vec<Action>),

    SwitchScreen(Screen),

    SearchQueryUpdated,
    SearchParamsUpdated,

    FetchSubtitles,
    SubtitlesFetched(Vec<Subtitle>),
    SubtitleDownloaded(Downloaded),

    LanguagesInitialized,
    LanguagesUpdated,
    LanguagesFetched(Vec<Language>),
    UserLanguagesFetched(Vec<String>),

    UserLoggedIn(User),
    UserLoggedOut,

    ChangeStatus(String),

    RunTask(Task),
    StartProgress,
    StopProgress,
}

impl Debug for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Init => write!(f, "Init"),
            Action::Exit => write!(f, "Exit"),
            Action::Tick => write!(f, "Tick"),
            Action::NoOp => write!(f, "NoOp"),
            Action::Multi(actions) => f.debug_tuple("Multi").field(actions).finish(),
            Action::SwitchScreen(screen) => f.debug_tuple("SwitchScreen").field(screen).finish(),
            Action::SearchQueryUpdated => write!(f, "SearchQueryUpdated"),
            Action::SearchParamsUpdated => write!(f, "SearchParamsUpdated"),
            Action::FetchSubtitles => write!(f, "FetchSubtitles"),
            Action::SubtitlesFetched(subtitles) => {
                write!(f, "SubtitlesFetched({} subtitles)", subtitles.len())
            }
            Action::SubtitleDownloaded(downloaded) => {
                f.debug_tuple("SubtitleDownloaded").field(downloaded).finish()
            }
            Action::LanguagesInitialized => write!(f, "LanguagesInitialized"),
            Action::LanguagesUpdated => write!(f, "LanguagesUpdated"),
            Action::LanguagesFetched(languages) => {
                write!(f, "LanguagesFetched({} languages)", languages.len())
            }
            Action::UserLanguagesFetched(languages) => {
                f.debug_tuple("UserLanguagesFetched").field(languages).finish()
            }
            Action::UserLoggedIn(user) => f.debug_tuple("UserLoggedIn").field(user).finish(),
            Action::UserLoggedOut => write!(f, "UserLoggedOut"),
            Action::ChangeStatus(status) => f.debug_tuple("ChangeStatus").field(status).finish(),
            Action::RunTask(task) => f.debug_tuple("RunTask").field(task).finish(),
            Action::StartProgress => write!(f, "StartProgress"),
            Action::StopProgress => write!(f, "StopProgress"),
        }
    }
}
