mod config;

use crate::config::get_config;
use log::{error, info};

#[tokio::main]
async fn main() {
    env_logger::init();

    match get_config() {
        Ok(conf) => {
            info!("{:?}", conf.api.key);
        }
        Err(e) => {
            error!("{:?}", e);
        }
    };
}
