use crate::APP_NAME;
use crate::osb::login::{Credentials, JwtToken};
use anyhow::{Error, Result};
use gio::prelude::ToSendValue;
use libsecret::prelude::{RetrievableExt, RetrievableExtManual};
use libsecret::{Schema, SchemaAttributeType, SchemaFlags, SearchFlags};
use log::{debug, error, info};
use secrecy::{ExposeSecret, SecretBox};
use std::collections::HashMap;
use tokio::task;

pub async fn store_token(api_token: &JwtToken) -> Result<()> {
    info!("Storing api token");
    let token = api_token.0.expose_secret().clone();
    task::spawn_blocking(move || {
        let schema = create_schema_token();

        if let Err(e) = libsecret::password_store_sync(
            Some(&schema),
            HashMap::new(),
            Some(libsecret::COLLECTION_DEFAULT),
            &format!("{}{}", APP_NAME, "_token"),
            token.as_str(),
            None::<&gio::Cancellable>,
        ) {
            error!("Storing API token failed: {e}")
        };
    })
    .await?;

    info!("Api token successfully stored");

    Ok(())
}

pub async fn retrieve_token() -> Result<Option<JwtToken>> {
    task::spawn_blocking(move || {
        let schema = create_schema_token();
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

pub async fn clear_token() -> Result<()> {
    task::spawn_blocking(move || {
        let schema = create_schema_token();
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
    info!("Storing credentials");
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

    info!("Credentials successfully stored");

    Ok(())
}

pub async fn retrieve_credentials() -> Result<Option<Credentials>> {
    task::spawn_blocking(move || {
        let schema = create_schema_credentials();
        match libsecret::password_search_sync(
            Some(&schema),
            HashMap::new(),
            SearchFlags::ALL,
            None::<&gio::Cancellable>,
        ) {
            Ok(vec) => {
                let result = vec.first().map(|head| {
                    let secret = head
                        .retrieve_secret_sync(None::<&gio::Cancellable>)
                        .unwrap()
                        .unwrap()
                        .text()
                        .unwrap()
                        .to_string();
                    let username_opt = head.attributes().get("username").cloned().unwrap();
                    Credentials {
                        username: username_opt,
                        password: secret,
                    }
                });
                Ok(result)
            }
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

fn create_schema_token() -> Schema {
    Schema::new(
        format!("com.{APP_NAME}.token").as_str(),
        SchemaFlags::NONE,
        HashMap::new(),
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
