//! rustlio simplifies interacting with Twilio's APIs in Rust applications.

pub mod lookup;
pub mod messaging;
pub mod security;

use serde::{Deserialize, Serialize};

/// This models the response received from Twilio when a request is unsuccessful
///
/// You can find complete documentation
/// [in the documentation](https://www.twilio.com/docs/usage/twilios-response#exceptions).
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
    pub code: Option<u16>,
    pub more_info: Option<String>,
}
