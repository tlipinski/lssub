use crate::config::Api;
use crate::login::ApiToken;
use anyhow::Result;
use libsecret::{Schema, SchemaAttributeType, SchemaFlags};
use log::{debug, error, info};
use secrecy::{ExposeSecret, SecretBox};
use std::collections::HashMap;

pub async fn store(api_token: &ApiToken, username: &str) -> Result<()> {
    debug!("Storing api token");
    let mut attributes = HashMap::new();
    attributes.insert("username", SchemaAttributeType::String);

    let collection = libsecret::COLLECTION_DEFAULT;
    let schema = Schema::new("com.subster", SchemaFlags::NONE, attributes);

    let mut attributes = HashMap::new();
    attributes.insert("username", username);

    // todo make async
    libsecret::password_store_sync(
        Some(&schema),
        attributes,
        Some(&collection),
        "Subster",
        api_token.0.expose_secret(),
        None::<&gio::Cancellable>,
    )?;

    debug!("Api token successfully stored");

    Ok(())
}

pub async fn retrieve() -> Result<Option<ApiToken>> {
    let mut attributes = HashMap::new();
    attributes.insert("username", SchemaAttributeType::String);

    let schema = Schema::new("com.subster", SchemaFlags::NONE, attributes);

    let mut attributes = HashMap::new();
    // attributes.insert("username", "pplcanfly");

    match libsecret::password_lookup_sync(Some(&schema), attributes, None::<&gio::Cancellable>) {
        Ok(Some(token)) => Ok(Some(ApiToken(SecretBox::from(Box::new(String::from(
            token.as_str(),
        )))))),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("{}", e);
            Err(anyhow::Error::new(e))
        }
    }
}
