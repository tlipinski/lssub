use lazy_static::lazy_static;
use std::iter::Iterator;

pub const API_URL: &str = "https://api.opensubtitles.com/api/v1";
pub const VIP_API_URL: &str = "https://vip-api.opensubtitles.com/api/v1";
pub const USER_AGENT: &str = "lssub v0.0.1";
const K: &str = env!("OSB_API_KEY");

lazy_static! {
    pub static ref KEY: String = K.chars().collect::<String>();
}