use lazy_static::lazy_static;
use std::iter::Iterator;

pub const API_URL: &str = "https://api.opensubtitles.com/api/v1";
pub const USER_AGENT: &str = "subster v0.1.0";
const K: &str = env!("OSBK");

lazy_static! {
    pub static ref KEY: String = K.chars().rev().collect::<String>();
}