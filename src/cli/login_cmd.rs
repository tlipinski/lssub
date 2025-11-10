use crate::secret::store;
use anyhow::Result;
use log::info;
use osb::login::{Credentials, login};
use std::io::{Write, stdin, stdout};

pub async fn handle_login_cmd() -> Result<()> {
    let mut username = String::new();

    print!("Username: ");
    stdout().flush()?;
    stdin().read_line(&mut username)?;

    print!("Password: ");
    stdout().flush()?;
    let password = rpassword::read_password()?;

    username = username.trim().to_string();

    let credentials = Credentials {
        username: username.clone(),
        password,
    };

    let api_token = login(&credentials).await?;

    store(&api_token, &username).await?;

    info!("Logged in successfully");

    Ok(())
}
