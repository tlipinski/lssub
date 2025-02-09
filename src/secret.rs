use crate::login::ApiToken;
use anyhow::{Error, Result};
use libsecret::{Schema, SchemaAttributeType, SchemaFlags};
use log::{debug, error};
use secrecy::{ExposeSecret, SecretBox};
use std::collections::HashMap;
use tokio::task;

pub async fn store(api_token: &ApiToken, username: &str) -> Result<()> {
    debug!("Storing api token");
    let token = api_token.0.expose_secret().clone();
    let un = username.to_string();
    task::spawn_blocking(move || {
        let mut attributes = HashMap::new();
        attributes.insert("username", SchemaAttributeType::String);

        let collection = libsecret::COLLECTION_DEFAULT;
        let schema = Schema::new("com.subster", SchemaFlags::NONE, attributes);

        let mut attributes = HashMap::new();
        attributes.insert("username", un.as_str());

        let _ = libsecret::password_store_sync(
            Some(&schema),
            attributes,
            Some(&collection),
            "Subster",
            token.as_str(),
            None::<&gio::Cancellable>,
        );
    })
    .await?;

    debug!("Api token successfully stored");

    Ok(())
}

pub async fn retrieve() -> Result<Option<ApiToken>> {
    task::spawn_blocking(move || {
        let mut attributes = HashMap::new();
        attributes.insert("username", SchemaAttributeType::String);

        let schema = Schema::new("com.subster", SchemaFlags::NONE, attributes);

        match libsecret::password_lookup_sync(
            Some(&schema),
            HashMap::new(),
            None::<&gio::Cancellable>,
        ) {
            Ok(Some(token)) => Ok(Some(ApiToken(SecretBox::from(Box::new(String::from(
                token.as_str(),
            )))))),
            Ok(None) => Ok(None),
            Err(e) => {
                error!("{}", e);
                Err(anyhow::Error::new(e))
            }
        }
    })
    .await?
}
