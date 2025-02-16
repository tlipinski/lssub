use osb::login::ApiToken;
use anyhow::Result;
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
        let schema = create_schema();

        let mut attributes = HashMap::new();
        attributes.insert("username", un.as_str());

        if let Err(e) = libsecret::password_store_sync(
            Some(&schema),
            attributes,
            Some(&libsecret::COLLECTION_DEFAULT),
            "Subster",
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

pub async fn retrieve() -> Result<Option<ApiToken>> {
    task::spawn_blocking(move || {
        let schema = create_schema();
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

pub async fn clear() -> Result<()> {
    task::spawn_blocking(move || {
        let schema = create_schema();
        match libsecret::password_clear_sync(
            Some(&schema),
            HashMap::new(),
            None::<&gio::Cancellable>,
        ) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("{}", e);
                Err(anyhow::Error::new(e))
            }
        }
    })
    .await?
}

fn create_schema() -> Schema {
    let mut attributes = HashMap::new();
    attributes.insert("username", SchemaAttributeType::String);

    Schema::new("com.subster", SchemaFlags::NONE, attributes)
}
