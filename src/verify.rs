//! Structs, functions, etc for working with Twilio's Verify API endpoint

use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use url::Url;

use crate::{ApiError, ApiRequest, TwilioRestClient};

const VERIFY_BASE_URI: &str = "https://verify.twilio.com/v2/Services";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SendCodeAttempt {
    pub attempt_sid: Option<String>,
    pub channel: Option<String>,
    pub time: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Sna {
    pub sna: Option<String>,
}

/// This models the phone number information that is returned from requests to the API
///
/// <div class="warning">
/// Currently, the struct only models some of the properties.
/// The remaining properties will be added in future versions of the crate.
/// Specifically, it only models the core properties plus properties of the Sim Swap and
/// Line Type Intelligence add on packages.
/// </div>
///
/// You can find complete documentation about all of the available properties
/// [in the documentation](https://www.twilio.com/docs/lookup/v2-api#response-properties).
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SendTokenResponse {
    pub account_sid: Option<String>,
    pub amount: Option<String>,
    pub channel: Option<String>,
    pub date_created: Option<String>,
    pub date_updated: Option<String>,
    pub payee: Option<String>,
    pub send_code_attempts: Option<Vec<SendCodeAttempt>>,
    pub service_sid: Option<String>,
    pub sid: Option<String>,
    pub sna: Option<Sna>,
    pub status: Option<String>,
    pub to: Option<String>,
    pub url: Option<String>,
    pub valid: bool,
}

/// Models the response from a Verification Check API request.
///
/// See [the
/// documentation](https://www.twilio.com/docs/verify/api/verification-check#verificationcheck-response-properties)
/// for more information.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct VerificationCheckResponse {
    pub account_sid: Option<String>,
    pub amount: Option<String>,
    pub channel: Option<String>,
    pub date_created: Option<String>,
    pub date_updated: Option<String>,
    pub payee: Option<String>,
    pub service_sid: Option<String>,
    pub sid: Option<String>,
    pub sna_attempts_error_codes: Option<Vec<String>>,
    pub status: Option<String>,
    pub to: Option<String>,
    pub valid: bool,
}

/// Models the request body parameters to the start new verification endpoint
///
/// See [the documentation](https://www.twilio.com/docs/verify/api/verification#request-body-parameters) for full details.
pub struct StartVerificationRequestParams<'a> {
    pub to: &'a str,
    pub channel: &'a str,
    pub custom_friendly_name: &'a str,
    pub send_digits: &'a str,
    pub locale: &'a str,
    pub custom_code: &'a str,
    pub amount: &'a str,
    pub payee: &'a str,
    pub app_hash: &'a str,
    pub template_sid: &'a str,
    pub device_ip: &'a str,
    pub enabled_sna_client_token: bool,
    pub risk_check: &'a str,
    pub tags: &'a str,
}

/// Models the request body parameters to the check verification endpoint
///
/// See [the documentation](https://www.twilio.com/docs/verify/api/verification-check#request-body-parameters) for full details.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct VerificationCheckRequestParams {
    pub amount: String,
    pub code: String,
    pub payee: String,
    pub sna_client_token: String,
    pub to: String,
    pub verification_sid: String,
}

/// Models the response from a Create New Factor API request.
///
/// See [the documentation](https://www.twilio.com/docs/verify/api/factor#factor-properties) for
/// more information.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct FactorResource {
    pub account_sid: Option<String>,
    pub identity: Option<String>,
    pub entity_sid: Option<String>,
    pub service_sid: Option<String>,
    pub sid: Option<String>,
    pub date_created: Option<String>,
    pub date_updated: Option<String>,
    pub friendly_name: Option<String>,
    pub status: Option<String>,
    pub factor_type: Option<String>,
    pub config: Option<String>,
    pub metadata: Option<String>,
    pub url: bool,
}

pub enum FactorType {
    Passkeys,
    Push,
    Totp,
}

/// Models the request body parameters to the Create Factor endpoint
///
/// See [the
/// documentation](https://www.twilio.com/docs/verify/api/factor#create-a-new-factor-resource) for
/// full details.
pub struct CreateNewFactorRequestParams {
    pub friendly_name: String,
    pub factor_type: FactorType,
    pub metadata: Option<String>,
}

impl From<CreateNewFactorRequestParams> for HashMap<String, String> {
    fn from(val: CreateNewFactorRequestParams) -> Self {
        let factor_type = match val.factor_type {
            FactorType::Passkeys => String::from("passkeys"),
            FactorType::Push => String::from("push"),
            FactorType::Totp => String::from("totp"),
        };
        HashMap::from([
            ("FriendlyName".to_string(), val.friendly_name),
            ("FactorType".to_string(), factor_type),
            (
                "Metadata".to_string(),
                val.metadata.unwrap_or("".to_string()),
            ),
        ])
    }
}

/// Models the request body parameters to the Update Factor endpoint
///
/// See [the
/// documentation](https://www.twilio.com/docs/verify/api/factor#request-body-parameters-1) for
/// full details.
pub struct UpdateFactorRequestParams {
    pub auth_payload: Option<String>,
    pub friendly_name: Option<String>,
}

impl From<UpdateFactorRequestParams> for HashMap<String, String> {
    fn from(val: UpdateFactorRequestParams) -> Self {
        HashMap::from([
            (
                "AuthPayload".to_string(),
                val.auth_payload.unwrap_or("".to_string()),
            ),
            (
                "FriendlyName".to_string(),
                val.friendly_name.unwrap_or("".to_string()),
            ),
        ])
    }
}

/// Models the request body parameters to the Create Challenge endpoint
///
/// See [the
/// documentation](https://www.twilio.com/docs/verify/api/challenge#request-body-parameters) for
/// full details.
pub struct CreateChallengeRequestParams {
    pub auth_payload: Option<String>,
    pub expiration_date: Option<String>,
    pub factor_sid: String,
    pub hidden_details: Option<String>,
}

impl From<CreateChallengeRequestParams> for HashMap<String, String> {
    fn from(val: CreateChallengeRequestParams) -> Self {
        HashMap::from([
            ("FactorSid".to_string(), val.factor_sid),
            (
                "AuthPayload".to_string(),
                val.auth_payload.unwrap_or("".to_string()),
            ),
            (
                "ExpirationDate".to_string(),
                val.expiration_date.unwrap_or("".to_string()),
            ),
            (
                "HiddenDetails".to_string(),
                val.hidden_details.unwrap_or("".to_string()),
            ),
        ])
    }
}

/// Models the response from a Verification Check API request.
///
/// See [the
/// documentation](https://www.twilio.com/docs/verify/api/verification-check#verificationcheck-response-properties)
/// for more information.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ChallengeResource {
    pub account_sid: Option<String>,
    pub amount: Option<String>,
    pub channel: Option<String>,
    pub date_created: Option<String>,
    pub date_updated: Option<String>,
    pub payee: Option<String>,
    pub service_sid: Option<String>,
    pub sid: Option<String>,
    pub sna_attempts_error_codes: Option<Vec<String>>,
    pub status: Option<String>,
    pub to: Option<String>,
    pub valid: bool,
}

impl VerificationCheckRequestParams {
    /// Returns the value of a VerificationCheckRequestParams field.
    ///
    /// It was created to allow for programmatic retrieval of values from
    /// VerificationCheckRequestParams objects while converting them to a
    /// HashMap. Might be useful for other conversions and operations as well.
    fn get(&self, field: &str) -> Result<String, String> {
        match field {
            "amount" => Ok(self.amount.clone()),
            "code" => Ok(self.code.clone()),
            "payee" => Ok(self.payee.clone()),
            "sna_client_token" => Ok(self.sna_client_token.clone()),
            "to" => Ok(self.to.clone()),
            "verification_sid" => Ok(self.verification_sid.clone()),
            _ => Err(format!("invalid field name to get '{}'", field)),
        }
    }
}

/// A simplistic type conversion from a VerificationCheckRequestParams object
/// into a HashMap.
impl From<VerificationCheckRequestParams> for HashMap<String, String> {
    fn from(val: VerificationCheckRequestParams) -> Self {
        let mut map = HashMap::new();
        let fields = vec![
            "amount",
            "code",
            "payee",
            "sna_client_token",
            "to",
            "verification_sid",
        ];
        for field_name in fields.into_iter() {
            if let Ok(field_value) = val.get(field_name)
                && !field_value.is_empty()
            {
                map.insert(stringcase::pascal_case(field_name), field_value);
            }
        }

        map
    }
}

// A custom error type for handling errors constructing Verify URLs.
#[derive(Debug, Clone)]
pub struct VerifyUriConstructionError {
    uri_type: String,
}

impl Error for VerifyUriConstructionError {}

impl fmt::Display for VerifyUriConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid Verify URI type {} requested.", self.uri_type)
    }
}

/// Verify simplifies interacting with Twilio's Verify API
///
/// It requires a TwilioRestClient for making calls to the Verify API and a
/// base_uri for knowing the base URI of the Verify API, on which each different
/// type of request can be based.
#[derive(Debug)]
pub struct Verify {
    pub client: TwilioRestClient,
    pub base_uri: String,
    pub verify_service_sid: String,
}

/// Provides a default implementation of a Verify struct
impl Default for Verify {
    fn default() -> Self {
        Self {
            base_uri: VERIFY_BASE_URI.to_string(),
            client: TwilioRestClient {
                account_sid: String::from(""),
                auth_token: String::from(""),
            },
            verify_service_sid: "".to_string(),
        }
    }
}

impl Verify {
    /// Sends a verification (OTP) token to the required device using the specified channel
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// use rustlio::RequestValue;
    /// use rustlio::TwilioRestClient;
    /// use rustlio::verify::Verify;
    ///
    /// # tokio_test::block_on(async {
    /// let verify = Verify {
    ///     client: TwilioRestClient {
    ///         account_sid: "<Your Twilio Account SID>".to_string(),
    ///         auth_token: "<Your Twilio Auth Token>".to_string(),
    ///     },
    ///     ..Default::default()
    /// };
    /// verify.send_verification_token(
    ///     "VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ///     "+61123456789",
    ///     "sms"
    /// );
    /// # })
    /// ```
    ///
    /// For more information, [check out the Start New Verification
    /// documentation](https://www.twilio.com/docs/verify/api/verification#start-new-verification).
    pub async fn send_verification_token(
        &self,
        verify_service_sid: &str,
        send_to: &str,
        channel: &str,
    ) -> Result<SendTokenResponse, ApiError> {
        let request_url = self.get_verify_base_uri(verify_service_sid, "Verifications");
        let request_params = HashMap::from([
            ("To".to_string(), send_to.to_string()),
            ("Channel".to_string(), channel.to_string()),
        ]);
        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params)
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<SendTokenResponse>().await?;
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

    /// Checks a verification (OTP) token to validate that the provided token is correct
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// use rustlio::RequestValue;
    /// use rustlio::TwilioRestClient;
    /// use rustlio::verify::Verify;
    /// use rustlio::verify::VerificationCheckRequestParams;
    ///
    /// # tokio_test::block_on(async {
    /// let verify = Verify {
    ///     client: TwilioRestClient {
    ///         account_sid: "<Your Twilio Account SID>".to_string(),
    ///         auth_token: "<Your Twilio Auth Token>".to_string(),
    ///     },
    ///     ..Default::default()
    /// };
    ///
    /// let check_params = VerificationCheckRequestParams {
    ///     code: "611234".to_string(),
    ///     to: "+61123456789".to_string(),
    ///     verification_sid: "".to_string(),
    ///     amount: "".to_string(),
    ///     payee: "".to_string(),
    ///     sna_client_token: "".to_string(),
    /// };
    ///
    /// verify.check_verification_token("VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", check_params);
    /// # })
    /// ```
    ///
    /// For more information, [check out the Start New Verification
    /// documentation](https://www.twilio.com/docs/verify/api/verification#start-new-verification).
    pub async fn check_verification_token(
        &self,
        verify_service_sid: &str,
        check_params: VerificationCheckRequestParams,
    ) -> Result<VerificationCheckResponse, ApiError> {
        let request_url = self.get_verify_base_uri(verify_service_sid, "VerificationCheck");
        let request_params: HashMap<String, String> = check_params.into();

        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params)
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<VerificationCheckResponse>().await?;
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

    /// Creates a new Factor
    pub async fn create_factor(
        &self,
        identity: &str,
        check_params: CreateNewFactorRequestParams,
    ) -> Result<FactorResource, ApiError> {
        let request_url = self
            .get_verify_uri(
                &HashMap::from([("Identity", identity)]),
                "create-new-factor-resource",
            )
            .map_err(|e| ApiError::ServerError(e.to_string()))?;

        let request_params: HashMap<String, String> = check_params.into();

        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params)
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<FactorResource>().await?;
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

    async fn handle_response(response: Response) -> Result<FactorResource, ApiError> {
        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<FactorResource>().await?;
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

    /// Updates an existing factor
    pub async fn update_factor(
        &self,
        verify_service_sid: &str,
        check_params: UpdateFactorRequestParams,
    ) -> Result<FactorResource, ApiError> {
        let request_url = self.get_verify_base_uri(verify_service_sid, "VerificationCheck");
        let request_params: HashMap<String, String> = check_params.into();

        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params)
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<FactorResource>().await?;
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

    /// Creates a challenge (and verifies it)
    pub async fn create_challenge(
        &self,
        verify_service_sid: &str,
        check_params: CreateChallengeRequestParams,
    ) -> Result<ChallengeResource, ApiError> {
        let request_url = self.get_verify_base_uri(verify_service_sid, "VerificationCheck");
        let request_params: HashMap<String, String> = check_params.into();

        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params)
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let token_response = response.json::<ChallengeResource>().await?;
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

    /// A small utility function for retrieving the Verify API base URL
    fn get_verify_base_uri(&self, verify_service_sid: &str, path: &str) -> Url {
        Url::parse(&format!(
            "{base_uri}/{verify_service_sid}/{path}",
            base_uri = self.base_uri,
        ))
        .expect("Unable to parse the provided URL")
    }

    /// Retrieves the full URI for making a request to a given Verify service
    pub fn get_verify_uri(
        &self,
        uri_params: &HashMap<&str, &str>,
        endpoint: &str,
    ) -> Result<Url, VerifyUriConstructionError> {
        match endpoint {
            "create-new-factor-resource" => {
                let url = self.get_verify_base_uri(
                    self.verify_service_sid.as_str(),
                    format!(
                        "Entities/{Identity}/Factors",
                        Identity = uri_params.get("identity").expect("Identity not provided")
                    )
                    .as_str(),
                );
                Ok(url)
            }
            unknown => Err(VerifyUriConstructionError {
                uri_type: unknown.to_string(),
            }),
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

    #[test]
    fn can_get_verify_base_url() {
        let verify = Verify {
            ..Default::default()
        };
        assert_eq!(
            "https://verify.twilio.com/v2/Services/VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/VerificationCheck",
            verify
                .get_verify_base_uri("VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "VerificationCheck")
                .as_str()
        );
    }

    #[test]
    fn can_get_verify_uri() {
        let identity = "DACB3AA9B96AAFB35E6FEE7C525FFCA18282AFF39E286DE22325AB6";
        let verify_service_sid = "VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let verify = Verify {
            verify_service_sid: verify_service_sid.to_string(),
            ..Default::default()
        };

        let test_data = [(
            "create-new-factor-resource",
            HashMap::from([("identity", identity)]),
            format!(
                "https://verify.twilio.com/v2/Services/{verify_service_sid}/Entities/{identity}/Factors"
            ),
        )];

        for data in test_data.iter() {
            let (endpoint, uri_params, expected_uri) = data;
            let url = verify.get_verify_uri(uri_params, endpoint);
            assert_eq!(*expected_uri, url.unwrap().to_string());
        }
    }

    #[tokio::test]
    async fn can_send_verification_token() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Verifications"))
            .respond_with(ResponseTemplate::new(201).set_body_raw(
                r##"{"sid":"VEaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","service_sid":"VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","account_sid":"ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","to":"+61123456789","channel":"sms","status":"approved","valid":false,"date_created":"2015-07-30T20:00:00Z","date_updated":"2015-07-30T20:00:00Z","lookup":{},"amount":null,"payee":null,"send_code_attempts":[{"time":"2015-07-30T20:00:00Z","channel":"SMS","attempt_sid":"VLaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}],"sna":null,"url":"https://verify.twilio.com/v2/Services/VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Verifications/VEaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"##,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        let verify = Verify {
            base_uri: mock_server.uri() + "/v2/Services",
            ..Default::default()
        };
        let verification_sid = "VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let response = verify
            .send_verification_token(verification_sid, "+61123456789", "sms")
            .await
            .expect("Should have returned a result");
        let status = match response.status {
            Some(status) => status,
            None => "".to_string(),
        };

        assert_eq!(status, "approved".to_string());
    }

    #[tokio::test]
    async fn can_check_verification_token() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/VerificationCheck"))
            .respond_with(ResponseTemplate::new(201).set_body_raw(
                r##"{"sid":"VEaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","service_sid":"VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","account_sid":"ACaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","to":"+15017122661","channel":"sms","status":"approved","valid":true,"amount":null,"payee":null,"sna_attempts_error_codes":[],"date_created":"2015-07-30T20:00:00Z","date_updated":"2015-07-30T20:00:00Z"}"##,
                "application/json",
            ))
            .mount(&mock_server)
            .await;

        let verify = Verify {
            base_uri: mock_server.uri() + "/v2/Services",
            ..Default::default()
        };
        let check_params = VerificationCheckRequestParams {
            code: "611234".to_string(),
            to: "+61123456789".to_string(),
            verification_sid: "".to_string(),
            amount: "".to_string(),
            payee: "".to_string(),
            sna_client_token: "".to_string(),
        };
        let verification_sid = "VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let response = verify
            .check_verification_token(verification_sid, check_params)
            .await
            .expect("Should have returned a result");
        let status = match response.status {
            Some(status) => status,
            None => "".to_string(),
        };

        assert_eq!(status, "approved".to_string());
    }
}
