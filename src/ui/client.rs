use crate::osb::osb_client::OsbClient;
use crate::values::{AK, API_URL, USER_AGENT};

impl Default for OsbClient {
    fn default() -> Self {
        OsbClient::new(API_URL, AK, USER_AGENT)
    }
}
