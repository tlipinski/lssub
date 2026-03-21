use crate::APP_NAME;
use crate::osb::login::{Credentials, JwtToken};
use anyhow::{Error, Result};
use libsecret::{Schema, SchemaAttributeType, SchemaFlags};
use log::{debug, error, info};
use secrecy::{ExposeSecret, SecretBox};
use std::collections::HashMap;
use tokio::task;

pub async fn store_token(api_token: &JwtToken, username: &str) -> Result<()> {
    debug!("Storing api token");
    let token = api_token.0.expose_secret().clone();
    let un = username.to_string();
    task::spawn_blocking(move || {
        let schema = create_schema_pass();

        let mut attributes = HashMap::new();
        attributes.insert("username", un.as_str());

        if let Err(e) = libsecret::password_store_sync(
            Some(&schema),
            attributes,
            Some(libsecret::COLLECTION_DEFAULT),
            &format!("{}{}", APP_NAME, "_token"),
            token.as_str(),
            None::<&gio::Cancellable>,
        ) {
            error!("Storing API token failed: {e}")
        };
    })
    .await?;

    debug!("Api token successfully stored");

    Ok(())
}

pub async fn retrieve() -> Result<Option<JwtToken>> {
    task::spawn_blocking(move || {
        let schema = create_schema_pass();
        match libsecret::password_lookup_sync(
            Some(&schema),
            HashMap::new(),
            None::<&gio::Cancellable>,
        ) {
            Ok(Some(token)) => Ok(Some(JwtToken(SecretBox::from(Box::new(String::from(
                token.as_str(),
            )))))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Error retrieving token: {}", e);
                Err(Error::msg("Error retrieving token"))
            }
        }
    })
    .await?
}

pub async fn clear() -> Result<()> {
    task::spawn_blocking(move || {
        let schema = create_schema_pass();
        match libsecret::password_clear_sync(
            Some(&schema),
            HashMap::new(),
            None::<&gio::Cancellable>,
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e);
                Err(Error::new(e))
            }
        }
    })
    .await?
}

pub async fn store_credentials(credentials: Credentials) -> Result<()> {
    debug!("Storing credentials");
    task::spawn_blocking(move || {
        let schema = create_schema_credentials();

        let mut attributes = HashMap::new();
        attributes.insert("username", credentials.username.as_str());

        if let Err(e) = libsecret::password_store_sync(
            Some(&schema),
            attributes,
            Some(libsecret::COLLECTION_DEFAULT),
            &format!("{}{}", APP_NAME, "_pass"),
            credentials.password.as_str(),
            None::<&gio::Cancellable>,
        ) {
            error!("Storing credentials failed: {e}")
        };
    })
    .await?;

    debug!("Credentials successfully stored");

    Ok(())
}

pub async fn retrieve_credentials() -> Result<Option<Credentials>> {
    task::spawn_blocking(move || {
        let schema = create_schema_credentials();
        match libsecret::password_lookup_sync(
            Some(&schema),
            HashMap::new(),
            None::<&gio::Cancellable>,
        ) {
            Ok(Some(password)) => Ok(Some(Credentials {
                username: "username".to_string(),
                password: password.to_string(),
            })),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Error retrieving token: {}", e);
                Err(Error::msg("Error retrieving token"))
            }
        }
    })
    .await?
}

pub async fn clear_credentials() -> Result<()> {
    task::spawn_blocking(move || {
        let schema = create_schema_credentials();
        match libsecret::password_clear_sync(
            Some(&schema),
            HashMap::new(),
            None::<&gio::Cancellable>,
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e);
                Err(Error::new(e))
            }
        }
    })
    .await?
}

fn create_schema_pass() -> Schema {
    let mut attributes = HashMap::new();
    attributes.insert("username", SchemaAttributeType::String);

    Schema::new(
        format!("com.{APP_NAME}.token").as_str(),
        SchemaFlags::NONE,
        attributes,
    )
}

fn create_schema_credentials() -> Schema {
    let mut attributes = HashMap::new();
    attributes.insert("username", SchemaAttributeType::String);

    Schema::new(
        format!("com.{APP_NAME}.pass").as_str(),
        SchemaFlags::NONE,
        attributes,
    )
}
