use std::path::Path;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Command {
    Login,
    Logout,
    UserInfo,
    Search {
        file_path: String,
        languages: Vec<String>,
    },
    Features {
        query: String,
    },
    Gui {
        file_path: Option<String>,
    },
}
