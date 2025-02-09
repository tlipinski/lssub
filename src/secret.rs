use crate::login::ApiToken;
use anyhow::Result;
use libsecret::{Schema, SchemaAttributeType, SchemaFlags};
use log::debug;
use secrecy::ExposeSecret;
use std::collections::HashMap;

pub async fn store(api_token: &ApiToken) -> Result<()> {
    debug!("Storing api token");
    let mut attributes = HashMap::new();
    attributes.insert("username", SchemaAttributeType::String);

    let collection = libsecret::COLLECTION_DEFAULT;
    let schema = Schema::new("com.subster", SchemaFlags::NONE, attributes);

    let mut attributes = HashMap::new();
    attributes.insert("username", api_token.username.as_str());

    // todo make async
    libsecret::password_store_sync(
        Some(&schema),
        attributes,
        Some(&collection),
        "Subster",
        api_token.token.expose_secret(),
        None::<&gio::Cancellable>,
    )?;

    debug!("Api token successfully stored");

    Ok(())
}
