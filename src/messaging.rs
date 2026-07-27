//! Structs, functions, etc for working with Twilio's Messaging API endpoint
use serde::Deserialize;

/// This models the response received from Twilio when messages are successfully sent
///
/// Messages can be SMS, MMS, RCS, and WhatsApp.
///
/// You can find full details about all of the available properties
/// [in the documentation](https://www.twilio.com/docs/messaging/api/message-resource#message-properties).
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct MessageResource {
    pub account_sid: Option<String>,
    pub api_version: Option<String>,
    pub body: Option<String>,
    pub date_created: Option<String>,
    pub date_sent: Option<String>,
    pub date_updated: Option<String>,
    pub direction: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub from: Option<String>,
    pub messaging_service_sid: Option<String>,
    pub num_media: Option<String>,
    pub num_segments: Option<String>,
    pub price: Option<String>,
    pub price_unit: Option<String>,
    pub sid: Option<String>,
    pub status: Option<String>,
    pub subresource_uris: Option<SubresourceUris>,
    pub to: Option<String>,
    pub uri: Option<String>,
}

/// This models the subresource_uris property of the response received from Twilio when messages are successfully sent
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SubresourceUris {
    pub all_time: String,
    pub today: String,
    pub yesterday: String,
    pub this_month: String,
    pub last_month: String,
    pub daily: String,
    pub monthly: String,
    pub yearly: String,
}
