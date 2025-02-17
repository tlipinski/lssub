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
}
