//! Structs, functions, etc for working with Twilio's Verify API endpoint

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{ApiRequest, TwilioRestClient};

const VERIFY_BASE_URI: &str = "https://verify.twilio.com/v2/Services";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SendCodeAttempt {
    pub time: Option<String>,
    pub channel: Option<String>,
    pub attempt_sid: Option<String>,
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
    pub sid: Option<String>,
    pub service_sid: Option<String>,
    pub account_sid: Option<String>,
    pub to: Option<String>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub date_created: Option<String>,
    pub date_updated: Option<String>,
    pub amount: Option<String>,
    pub payee: Option<String>,
    pub send_code_attempts: Option<Vec<SendCodeAttempt>>,
    pub sna: Option<Sna>,
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

/// Verify simplifies interacting with Twilio's Verify API
///
/// It requires a TwilioRestClient for making calls to the Verify API and a
/// base_uri for knowing the base URI of the Verify API, on which each different
/// type of request can be based.
#[derive(Debug, Clone)]
pub struct Verify<'a> {
    pub client: &'a TwilioRestClient<'a>,
    pub base_uri: String,
}

/// Provides a default implementation of a Verify struct
impl<'a> Default for Verify<'a> {
    fn default() -> Self {
        Self {
            base_uri: VERIFY_BASE_URI.to_string(),
            client: &TwilioRestClient {
                account_sid: "",
                auth_token: "",
            },
        }
    }
}

impl<'a> Verify<'a> {
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
    ///     client: &TwilioRestClient {
    ///         account_sid: "<Your Twilio Account SID>",
    ///         auth_token: "<Your Twilio Auth Token>",
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
    ) -> Result<SendTokenResponse, reqwest::Error> {
        let request_url = self.get_verify_base_uri(verify_service_sid, "Verifications");
        let request_params = HashMap::from([
            ("To".to_string(), send_to.to_string()),
            ("Channel".to_string(), channel.to_string()),
        ]);
        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params);

        match response.await {
            Ok(data) => {
                let record = data.json::<SendTokenResponse>().await?;
                println!("{:?}", record);
                Ok(record)
            }
            Err(e) => Err(e),
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
    ///     client: &TwilioRestClient {
    ///         account_sid: "<Your Twilio Account SID>",
    ///         auth_token: "<Your Twilio Auth Token>",
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
    ) -> Result<VerificationCheckResponse, reqwest::Error> {
        let request_url = self.get_verify_base_uri(verify_service_sid, "VerificationCheck");
        let request_params: HashMap<String, String> = check_params.into();
        println!("{:?}", request_params);

        let response = self
            .client
            .make_post_request(request_url.as_str(), &request_params);

        match response.await {
            Ok(data) => {
                let record = data.json::<VerificationCheckResponse>().await?;
                Ok(record)
            }
            Err(e) => Err(e),
        }
    }

    /// A small utility function for retrieving the Verify API base URL
    fn get_verify_base_uri(&self, verify_service_sid: &str, endpoint: &str) -> Url {
        Url::parse(&format!(
            "{base_uri}/{verify_service_sid}/{endpoint}",
            base_uri = self.base_uri,
        ))
        .expect("Unable to parse the provided URL")
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

    #[tokio::test]
    async fn can_send_verification_token() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v2/Services/VAaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/Verifications"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
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
            .respond_with(ResponseTemplate::new(200).set_body_raw(
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
