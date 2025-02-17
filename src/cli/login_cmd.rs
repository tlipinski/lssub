use osb::login::{login, Credentials};
use crate::secret::store;
use anyhow::Result;
use std::io::{stdin, stdout, Write};

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
        password: password,
    };

    let api_token = login(&credentials).await?;

    let _ = store(&api_token, &username).await?;

    println!("Logged in successfully");

    Ok(())
}
