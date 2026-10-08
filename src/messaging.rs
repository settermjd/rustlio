//! Structs, functions, etc for working with Twilio's Messaging API endpoint
use std::collections::HashMap;

use crate::{ApiError, ApiRequest, RequestValue, TwilioRestClient};

use http::StatusCode;
use serde::Deserialize;
use url::Url;

const MAX_MEDIA_URLS: usize = 10;
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

#[derive(Clone, Debug, Default, Deserialize)]
pub enum ContentRetention {
    Disregard,
    #[default]
    Retain,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub enum AddressRetention {
    Obfuscate,
    #[default]
    Retain,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub enum TrafficType {
    #[default]
    Free,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub enum ScheduleType {
    #[default]
    Fixed,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub enum RiskCheck {
    #[default]
    Enable,
    Disable,
}

/// Models the request body parameters that can be sent to the Messaging endpoint when making a request
#[derive(Debug, Deserialize)]
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

impl TryFrom<&MessageRequestBodyParams> for HashMap<String, RequestValue> {
    type Error = &'static str;

    /// Attempts to transform a MessageRequestBodyParams into a HashMap
    ///
    /// # Example
    ///
    /// ```
    /// use rustlio::RequestValue;
    /// use rustlio::messaging::ContentRetention::Disregard;
    /// use rustlio::messaging::MessageRequestBodyParams;
    /// use std::collections::HashMap;
    ///
    /// let params = MessageRequestBodyParams {
    ///     address_retention: None,
    ///     application_sid: None,
    ///     attempt: None,
    ///     body: Some("Hi there".to_string()),
    ///     content_retention: Some(Disregard),
    ///     content_sid: None,
    ///     content_variables: None,
    ///     fallback_from: None,
    ///     force_delivery: None,
    ///     from: Some("+12132635137".to_string()),
    ///     max_price: None,
    ///     media_url: None,
    ///     messaging_service_sid: None,
    ///     persistent_action: None,
    ///     provide_feedback: None,
    ///     risk_check: None,
    ///     schedule_type: None,
    ///     send_as_mms: None,
    ///     send_at: None,
    ///     shorten_urls: None,
    ///     smart_encoded: None,
    ///     status_callback: None,
    ///     to: "+61123456789".to_string(),
    ///     traffic_type: None,
    ///     validity_period: None,
    /// };
    ///
    /// let params_map = HashMap::<String, RequestValue>::try_from(&params);
    /// ```
    fn try_from(val: &MessageRequestBodyParams) -> Result<Self, Self::Error> {
        let mut map: HashMap<String, RequestValue> =
            HashMap::from([("To".to_string(), RequestValue::Str(val.to.clone()))]);

        if val.status_callback.is_some() {
            map.insert(
                "StatusCallback".to_string(),
                RequestValue::Str(val.status_callback.clone().unwrap_or("".to_string())),
            );
        }

        if val.application_sid.is_some() {
            map.insert(
                "ApplicationSid".to_string(),
                RequestValue::Str(val.application_sid.clone().unwrap_or("".to_string())),
            );
        }

        if val.send_at.is_some() {
            map.insert(
                "SendAt".to_string(),
                RequestValue::Str(val.send_at.clone().unwrap_or("".to_string())),
            );
        }

        if val.content_variables.is_some() {
            map.insert(
                "ContentVariables".to_string(),
                RequestValue::Str(val.content_variables.clone().unwrap_or("".to_string())),
            );
        }

        if val.fallback_from.is_some() {
            map.insert(
                "FallbackFrom".to_string(),
                RequestValue::Str(val.fallback_from.clone().unwrap_or("".to_string())),
            );
        }

        if val.risk_check.is_some() {
            let risk_check = match val.risk_check.clone().unwrap_or_default() {
                RiskCheck::Enable => "enable",
                RiskCheck::Disable => "disable",
            };
            map.insert(
                "RiskCheck".to_string(),
                RequestValue::Str(risk_check.to_string()),
            );
        }

        if val.schedule_type.is_some() {
            let schedule_type = match val.schedule_type.clone().unwrap_or_default() {
                ScheduleType::Fixed => "fixed",
            };
            map.insert(
                "ScheduleType".to_string(),
                RequestValue::Str(schedule_type.to_string()),
            );
        }

        if val.traffic_type.is_some() {
            let traffic_type = match val.traffic_type.clone().unwrap_or_default() {
                TrafficType::Free => "free",
            };
            map.insert(
                "TrafficType".to_string(),
                RequestValue::Str(traffic_type.to_string()),
            );
        }

        if val.address_retention.is_some() {
            let address_retention = match val.address_retention.clone().unwrap_or_default() {
                AddressRetention::Obfuscate => "obfuscate",
                AddressRetention::Retain => "retain",
            };
            map.insert(
                "AddressRetention".to_string(),
                RequestValue::Str(address_retention.to_string()),
            );
        }

        if val.content_retention.is_some() {
            let content_retention = match val.content_retention.clone().unwrap_or_default() {
                ContentRetention::Disregard => "disregard",
                ContentRetention::Retain => "retain",
            };
            map.insert(
                "ContentRetention".to_string(),
                RequestValue::Str(content_retention.to_string()),
            );
        }

        if val.from.is_none() && val.messaging_service_sid.is_none() {
            return Err("Either the from or messaging service SID is required");
        } else {
            if val.from.is_some() {
                map.insert(
                    "From".to_string(),
                    RequestValue::Str(val.from.clone().unwrap_or_default()),
                );
            }

            if val.messaging_service_sid.is_some() {
                map.insert(
                    "MessagingServiceSid".to_string(),
                    RequestValue::Str(val.messaging_service_sid.clone().unwrap_or_default()),
                );
            }
        }

        if val.send_as_mms.is_some() {
            map.insert(
                "SendAsMms".to_string(),
                RequestValue::Str(val.send_as_mms.unwrap_or(false).to_string()),
            );
        }

        if val.shorten_urls.is_some() {
            map.insert(
                "ShortenUrls".to_string(),
                RequestValue::Str(val.shorten_urls.unwrap_or(false).to_string()),
            );
        }

        if val.smart_encoded.is_some() {
            map.insert(
                "SmartEncoded".to_string(),
                RequestValue::Str(val.smart_encoded.unwrap_or(false).to_string()),
            );
        }

        if val.force_delivery.is_some() {
            map.insert(
                "ForceDelivery".to_string(),
                RequestValue::Str(val.force_delivery.unwrap_or(false).to_string()),
            );
        }

        if val.provide_feedback.is_some() {
            map.insert(
                "ProvideFeedback".to_string(),
                RequestValue::Str(val.provide_feedback.unwrap_or(false).to_string()),
            );
        }

        if val.max_price.is_some() {
            map.insert(
                "MaxPrice".to_string(),
                RequestValue::Str(val.max_price.unwrap_or_default().to_string()),
            );
        }

        if val.attempt.is_some() {
            map.insert(
                "Attempt".to_string(),
                RequestValue::Str(val.attempt.unwrap_or_default().to_string()),
            );
        }

        if val.validity_period.is_some() {
            map.insert(
                "ValidityPeriod".to_string(),
                RequestValue::Str(val.validity_period.unwrap_or_default().to_string()),
            );
        }

        if val.body.is_none() && val.media_url.is_none() && val.content_sid.is_none() {
            return Err("A body, media URL, or content SID is required.");
        } else {
            if val.body.is_some() {
                map.insert(
                    "Body".to_string(),
                    RequestValue::Str(val.body.clone().unwrap_or_default()),
                );
            }

            let mut range_max = MAX_MEDIA_URLS;
            let mut media_url = val.media_url.clone().unwrap_or_default();
            if media_url.len() < MAX_MEDIA_URLS {
                range_max = media_url.len();
            }
            for url in &mut media_url[0..range_max] {
                map.insert("MediaUrl".to_string(), RequestValue::Str(url.clone()));
            }

            if val.content_sid.is_some() {
                map.insert(
                    "ContentSid".to_string(),
                    RequestValue::Str(val.content_sid.clone().unwrap_or_default()),
                );
            }
        }

        Ok(map)
    }
}

impl TryFrom<HashMap<String, String>> for MessageRequestBodyParams {
    type Error = &'static str;

    /// Attempts to transform a HashMap into a MessageRequestBodyParams
    ///
    /// # Example
    ///
    /// ```
    /// use rustlio::messaging::MessageRequestBodyParams;
    /// use std::collections::HashMap;
    ///
    /// let message_params = MessageRequestBodyParams::try_from(HashMap::from([
    ///     ("to".to_string(), "+61123456789".to_string()),
    ///     ("from".to_string(), "+1987654321".to_string()),
    ///     ("body".to_string(), "Hi there".to_string()),
    /// ]));
    /// ```
    fn try_from(value: HashMap<String, String>) -> Result<Self, Self::Error> {
        let params = MessageRequestBodyParams {
            to: value.get("to").map_or("", |v| v).to_string(),
            status_callback: value.get("status_callback").map(|v| v.to_string()),
            application_sid: value.get("application_sid").map(|v| v.to_string()),
            max_price: value.get("max_price").map(|v| v.parse::<usize>().unwrap()),
            provide_feedback: value
                .get("provide_feedback")
                .map(|v| v.parse::<bool>().unwrap()),
            attempt: value.get("attempt").map(|v| v.parse::<usize>().unwrap()),
            validity_period: value
                .get("validity_period")
                .map(|v| v.parse::<usize>().unwrap()),
            force_delivery: value
                .get("force_delivery")
                .map(|v| v.parse::<bool>().unwrap()),
            content_retention: value
                .get("content_retention")
                .and_then(|v| match v.as_str() {
                    "disregard" => Some(ContentRetention::Disregard),
                    "retain" => Some(ContentRetention::Retain),
                    _ => None,
                }),
            address_retention: value.get("address_retention").and_then(|v| {
                if v.eq_ignore_ascii_case("obfuscate") {
                    return Some(AddressRetention::Obfuscate);
                }
                if v.eq_ignore_ascii_case("retain") {
                    return Some(AddressRetention::Obfuscate);
                }
                None
            }),
            smart_encoded: value
                .get("smart_encoded")
                .map(|v| v.parse::<bool>().unwrap()),
            persistent_action: value.get("persistent_action").map(|v| vec![v.to_string()]),
            traffic_type: value.get("traffic_type").and_then(|v| {
                if v.eq_ignore_ascii_case("free") {
                    return Some(TrafficType::Free);
                }
                None
            }),
            shorten_urls: value
                .get("shorten_urls")
                .map(|v| v.parse::<bool>().unwrap()),
            schedule_type: value.get("schedule_type").and_then(|v| {
                if v.eq_ignore_ascii_case("fixed") {
                    return Some(ScheduleType::Fixed);
                }
                None
            }),
            send_at: value.get("send_at").map(|v| v.to_string()),
            send_as_mms: value.get("send_as_mms").map(|v| v.parse::<bool>().unwrap()),
            content_variables: value.get("content_variables").map(|v| v.to_string()),
            risk_check: value.get("risk_check").and_then(|v| {
                if v.eq_ignore_ascii_case("enable") {
                    return Some(RiskCheck::Enable);
                }
                if v.eq_ignore_ascii_case("disable") {
                    return Some(RiskCheck::Disable);
                }
                None
            }),
            from: value.get("from").map(|v| v.to_string()),
            fallback_from: value.get("fallback_from").map(|v| v.to_string()),
            messaging_service_sid: value.get("messaging_service_sid").map(|v| v.to_string()),
            body: value.get("body").map(|v| v.to_string()),
            media_url: value.get("media_url").map(|v| vec![v.to_string()]),
            content_sid: value.get("content_sid").map(|v| v.to_string()),
        };

        Ok(params)
    }
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
    /// let message_params = MessageRequestBodyParams::try_from(HashMap::from([
    ///     ("to".to_string(), "+61123456789".to_string()),
    ///     ("from".to_string(), "+1987654321".to_string()),
    ///     ("body".to_string(), "Hi there. How are you?".to_string()),
    /// ]));
    /// let response = message_service
    ///     .send_message(&message_params.unwrap())
    ///     .await
    ///     .expect("Should have returned a result");
    /// let status = match response.status {
    ///     Some(status) => status,
    ///     None => "".to_string(),
    /// };
    /// println!("Message status was {status}");
    /// # })
    /// ```
    pub async fn send_message(
        &self,
        request_params: &MessageRequestBodyParams,
    ) -> Result<MessageResource, ApiError> {
        let request_url = self.get_base_uri();
        let Ok(params) = HashMap::try_from(request_params) else {
            return Err(ApiError::ServerError(String::from(
                "could not create hashmap of request params",
            )));
        };

        let response = self
            .client
            .make_post_request(request_url.as_str(), &params)
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

    fn can_create_hashmap_from_message_request_body_params() {
        let params = MessageRequestBodyParams {
            address_retention: None,
            application_sid: None,
            attempt: None,
            body: Some("Hi there".to_string()),
            content_retention: Some(ContentRetention::Disregard),
            content_sid: None,
            content_variables: None,
            fallback_from: None,
            force_delivery: None,
            from: Some("+12132635137".to_string()),
            max_price: None,
            media_url: None,
            messaging_service_sid: None,
            persistent_action: None,
            provide_feedback: None,
            risk_check: None,
            schedule_type: None,
            send_as_mms: None,
            send_at: None,
            shorten_urls: None,
            smart_encoded: None,
            status_callback: None,
            to: "+61123456789".to_string(),
            traffic_type: None,
            validity_period: None,
        };
        let params_map = HashMap::<String, RequestValue>::try_from(&params).unwrap();
        assert_eq!(
            params_map.get("Body"),
            Some(&RequestValue::Str("Hi".to_string()))
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
        let message_params = MessageRequestBodyParams::try_from(HashMap::from([
            ("to".to_string(), "+1123456789".to_string()),
            ("from".to_string(), "+1987654321".to_string()),
            ("body".to_string(), "Hi there".to_string()),
        ]));
        let response = message_service
            .send_message(&message_params.unwrap())
            .await
            .expect("Should have returned a result");
        let status = match response.status {
            Some(status) => status,
            None => "".to_string(),
        };

        assert_eq!(status, "queued".to_string());
    }
}
