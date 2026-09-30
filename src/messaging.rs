//! Structs, functions, etc for working with Twilio's Messaging API endpoint
use crate::{ApiError, ApiRequest, TwilioRestClient};

use http::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

const MESSAGE_BASE_URI: &str = "https://api.twilio.com/2010-04-01/Accounts";

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

/// Models the subresource_uris property of the response received from Twilio when messages are successfully sent
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SubresourceUris {
    pub media: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ContentRetention {
    Disregard,
    #[default]
    Retain,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum AddressRetention {
    Obfuscate,
    #[default]
    Retain,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum TrafficType {
    #[default]
    Free,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum ScheduleType {
    #[default]
    Fixed,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub enum RiskCheck {
    #[default]
    Enable,
    Disable,
}

/// Models the request body parameters that can be sent to the Messaging endpoint when making a request
#[derive(Debug, Deserialize, Serialize)]
pub struct MessageRequestBodyParams {
    pub to: String,
    pub status_callback: Option<String>,
    pub application_sid: Option<String>,
    pub max_price: Option<usize>,
    pub provide_feedback: Option<bool>,
    pub attempt: Option<usize>,
    pub validity_period: Option<usize>,
    pub force_delivery: Option<bool>,
    pub content_retention: Option<ContentRetention>,
    pub address_retention: Option<AddressRetention>,
    pub smart_encoded: Option<bool>,
    pub persistent_action: Option<Vec<String>>,
    pub traffic_type: Option<TrafficType>,
    pub shorten_urls: Option<bool>,
    pub schedule_type: Option<ScheduleType>,
    pub send_at: Option<String>,
    pub send_as_mms: Option<bool>,
    pub content_variables: Option<String>,
    pub risk_check: Option<RiskCheck>,
    pub from: Option<String>,
    pub fallback_from: Option<String>,
    pub messaging_service_sid: Option<String>,
    pub body: Option<String>,
    pub media_url: Option<Vec<String>>,
    pub content_sid: Option<String>,
}

#[derive(Debug)]
pub struct Message {
    pub client: TwilioRestClient,
    pub base_uri: String,
}

/// Provides a default implementation of the Message struct
impl Default for Message {
    fn default() -> Self {
        Self {
            base_uri: MESSAGE_BASE_URI.to_string(),
            client: TwilioRestClient {
                account_sid: String::from(""),
                auth_token: String::from(""),
            },
        }
    }
}

impl Message {
    /// Returns the required URI for interacting with the messaging API
    fn get_base_uri(&self) -> Url {
        Url::parse(&format!(
            "{base_uri}/{account_sid}/Messages.json",
            account_sid = self.client.account_sid,
            base_uri = self.base_uri,
        ))
        .expect("Unable to parse the provided messaging URL")
    }

    /// Sends a message based on the parameters provided
    ///
    /// For simplicities sake, the request's body parameters can be instantiated from a HashMap, so
    /// that you don't need to remember a function with far too many parameters, instead, only
    /// providing what you need, as and when you need it.
    ///
    /// # Examples
    ///
    /// Send an SMS:
    ///
    /// ```rust,no_run
    /// use std::collections::HashMap;
    ///
    /// use rustlio::{
    ///     TwilioRestClient,
    ///     messaging::{Message, MessageRequestBodyParams},
    /// };
    ///
    /// # tokio_test::block_on(async {
    /// let message_service = Message {
    ///     client: TwilioRestClient {
    ///         account_sid: String::from("ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"),
    ///         auth_token: String::from("1fdddddddddddddddddddddddddddddd"),
    ///     },
    ///     ..Default::default()
    /// };
    /// let message_params = [
    ///     ("to".to_string(), "+61123456789".to_string()),
    ///     ("from".to_string(), "+1987654321".to_string()),
    ///     ("body".to_string(), "Hi there. How are you?".to_string()),
    /// ];
    /// let response = message_service
    ///     .send_message(&message_params)
    ///     .await
    ///     .expect("Should have returned a result");
    /// let status = match response.status {
    ///     Some(status) => status,
    ///     None => "".to_string(),
    /// };
    /// println!("Message status was {status}");
    /// # })
    /// ```
    pub async fn send_message<T: Serialize + ?Sized>(
        &self,
        request_params: &T,
    ) -> Result<MessageResource, ApiError> {
        let request_url = self.get_base_uri();

        let response = self
            .client
            .make_post_request(request_url.as_str(), request_params)
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<MessageResource>().await?;
                Ok(token_response)
            }
            StatusCode::NOT_FOUND => Err(ApiError::NotFound),
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::TOO_MANY_REQUESTS => Err(ApiError::RateLimited),
            status if status.is_server_error() => {
                let body = response.text().await.unwrap_or_default();
                Err(ApiError::ServerError(body))
            }
            status => Err(ApiError::UnexpectedStatus(status)),
        }
    }
}

#[cfg(test)]
mod tests {
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    use super::*;
    use parameterized::parameterized;

    #[parameterized(account_sid = {"AC88888888888888888888888888888888", "AC88888888888888888444444444444444"})]
    fn can_build_base_uri_correctly(account_sid: &str) {
        let message_service = Message {
            client: TwilioRestClient {
                account_sid: String::from(account_sid),
                auth_token: String::from("1fcccccccccccccccccccccccccccccc"),
            },
            ..Default::default()
        };
        assert_eq!(
            message_service.get_base_uri().as_str(),
            format!(
                "{base_uri}/{account_sid}/Messages.json",
                base_uri = MESSAGE_BASE_URI
            )
        );
    }

    #[tokio::test]
    async fn can_send_an_sms() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/2010-04-01/Accounts/ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX/Messages.json"))
            .respond_with(ResponseTemplate::new(201).set_body_raw(
                r##"{"account_sid":"ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX","api_version":"2010-04-01","body":"Hi there","date_created":"Thu, 24 Aug 2023 05:01:45 +0000","date_sent":"Thu, 24 Aug 2023 05:01:45 +0000","date_updated":"Thu, 24 Aug 2023 05:01:45 +0000","direction":"outbound-api","error_code":null,"error_message":null,"from":"+1987654321","num_media":"0","num_segments":"1","price":null,"price_unit":null,"messaging_service_sid":"MGaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","sid":"SMaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":"queued","subresource_uris":{"media":"/2010-04-01/Accounts/ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Messages/SMaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Media.json"},"to":"+1123456789","uri":"/2010-04-01/Accounts/ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Messages/SMaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json"}"##,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        let message_service = Message {
            base_uri: mock_server.uri() + "/2010-04-01/Accounts",
            client: TwilioRestClient {
                account_sid: String::from("ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"),
                auth_token: String::from("1fcccccccccccccccccccccccccccccc"),
            },
        };
        let message_params = [
            ("to".to_string(), "+1123456789".to_string()),
            ("from".to_string(), "+1987654321".to_string()),
            ("body".to_string(), "Hi there".to_string()),
        ];
        let response = message_service
            .send_message(&message_params)
            .await
            .expect("Should have returned a result");
        let status = match response.status {
            Some(status) => status,
            None => "".to_string(),
        };

        assert_eq!(status, "queued".to_string());
    }

    #[tokio::test]
    async fn can_send_an_mms() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/2010-04-01/Accounts/ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX/Messages.json"))
            .respond_with(ResponseTemplate::new(201).set_body_raw(
                r##"{"account_sid":"ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX","api_version":"2010-04-01","body":"Hi there","date_created":"Thu, 24 Aug 2023 05:01:45 +0000","date_sent":"Thu, 24 Aug 2023 05:01:45 +0000","date_updated":"Thu, 24 Aug 2023 05:01:45 +0000","direction":"outbound-api","error_code":null,"error_message":null,"from":"+1987654321","num_media":"0","num_segments":"1","price":null,"price_unit":null,"messaging_service_sid":"MGaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","sid":"SMaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","status":"queued","subresource_uris":{"media":"/2010-04-01/Accounts/ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Messages/SMaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Media.json"},"to":"+1123456789","uri":"/2010-04-01/Accounts/ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Messages/SMaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json"}"##,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        let message_service = Message {
            base_uri: mock_server.uri() + "/2010-04-01/Accounts",
            client: TwilioRestClient {
                account_sid: String::from("ACXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"),
                auth_token: String::from("1fcccccccccccccccccccccccccccccc"),
            },
        };
        let message_params = [
            ("to".to_string(), "+1123456789".to_string()),
            ("from".to_string(), "+1987654321".to_string()),
            ("body".to_string(), "Hi there".to_string()),
            (
                "media_url".to_string(),
                "https://c1.staticflickr.com/3/2899/14341091933_1e92e62d12_b.jpg".to_string(),
            ),
        ];
        let response = message_service
            .send_message(&message_params)
            .await
            .expect("Should have returned a result");
        let status = match response.status {
            Some(status) => status,
            None => "".to_string(),
        };

        assert_eq!(status, "queued".to_string());
    }
}
