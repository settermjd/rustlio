//! Structs, functions, etc for validating inbound webhooks from Twilio
use std::collections::{BTreeMap, HashMap};

use base64::prelude::*;
use constant_time_eq::constant_time_eq;
use hex;
use hmac::{Hmac, KeyInit, Mac};
use log::info;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use url::Url;

/// WebhookValidator is used to verify the request signature included with Twilio requests to
/// [webhooks][webhooks]. Doing so ensures that the request actually came from Twilio and helps
/// with [securing your webhooks][webhook_security].
///
/// [webhooks]: https://www.twilio.com/docs/usage/webhooks
/// [webhook_security]: https://www.twilio.com/docs/usage/webhooks/webhooks-security
#[derive(Clone, Debug)]
pub struct WebhookValidator {
    /// Your [Twilio Auth Token][twilio_auth_token]
    ///
    /// This is the signing key when generating a SHA1 HMAC of the incoming request. It can be
    /// retrieved from [the Twilio Console][twilio_console] dashboard, or from the Workbench,
    /// available on all pages within the Console.
    ///
    /// [twilio_auth_token]: https://www.twilio.com/docs/iam/api/authtoken
    /// [twilio_console]: https://1console.twilio.com
    pub auth_token: String,
}

impl WebhookValidator {
    /// Validates the request body for webhooks configured for POST calls
    ///
    /// It returns true if the computed signature matches the expected signature. body is
    /// the HTTP request body from the webhook call as a byte string.
    ///
    /// # Examples
    ///
    /// Validate a request sent with a JSON body:
    ///
    /// ```rust,no_run
    /// use rustlio::security;
    /// use url::Url;
    ///
    /// let webhook_url = "https://96e5-165-225-114-134.ngrok-free.app/webhook?bodySHA256=e6ca4452daa092f8b0ecb9cdd24328f9b565196e0a25bc4e612bf198ad77fbd5";
    /// let mut url = match Url::parse(webhook_url) {
    ///     Ok(v) => v,
    ///     Err(_) => panic!("Could not parse URL"),
    /// };
    /// // For the purposes of this example, let's assume that this was the JSON body of
    /// // the request sent by Twilio to your application.
    /// let request_body = r#"[{"specversion":"1.0","type":"com.twilio.eventstreams.test-event","source":"Sink","id":"AC11111111111111111111111111111111","dataschema":"https://events-schemas.twilio.com/EventStreams.TestSink/1.json","datacontenttype":"application/json","time":"2026-06-10T06:02:54.377Z","data":{"test_id":"cae2f9e2-c277-4612-8ad3-93c1a7a3ef88"}}]"#;
    ///
    /// let validator = security::WebhookValidator {
    ///     auth_token: "<YOUR TWILIO AUTH TOKEN>".to_string(),
    /// };
    /// validator.validate_body(
    ///     &mut url,
    ///     &"aU96RJE2IgIwrbsBNwQT5eaT1tM=".to_string(),
    ///     request_body.as_bytes()
    /// );
    /// ```
    ///
    /// Validate a request sent with an [x-www-form-urlencoded](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Methods/POST) body:
    ///
    /// ```rust,no_run
    /// use rustlio::security;
    /// use url::Url;
    ///
    /// let mut url = match Url::parse("https://96e5-165-225-114-134.ngrok-free.app/webhook") {
    ///     Ok(v) => v,
    ///     Err(_) => panic!("Could not parse URL"),
    /// };
    /// // For the purposes of this example, let's assume that this was the JSON body of
    /// // the request sent by Twilio to your application.
    /// let request_body = "name=matthew&day=thursday";
    ///
    /// let validator = security::WebhookValidator {
    ///     auth_token: "<YOUR TWILIO AUTH TOKEN>".to_string(),
    /// };
    /// validator.validate_body(
    ///     &mut url,
    ///     &"cfJGwe55Ypzn7ffL4OFzJLzhkuc=".to_string(),
    ///     request_body.as_bytes()
    /// );
    /// ```
    pub fn validate_body(&self, url: &mut Url, expected_signature: &str, body: &[u8]) -> bool {
        let mut query_params = url.query_pairs();
        let body_sha256_query_pair = query_params
            .find(|pair| pair.0 == "bodySHA256")
            .unwrap_or_default();

        let (_, body_sha256) = body_sha256_query_pair;

        // If a SHA256 hash of the body has not been provided, then the body is assumed to
        // contain an x-www-form-urlencoded request.
        if body_sha256.is_empty() {
            let Ok(body) = str::from_utf8(body) else {
                return false;
            };
            let parsed_body = form_urlencoded::parse(body.as_bytes())
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect();
            return self.validate(url, &parsed_body, expected_signature);
        }

        self.validate(&mut url.clone(), &HashMap::new(), expected_signature)
            && self.validate_body_content(body, &body_sha256)
    }

    /// Validates Twilio request signatures sent with GET webhooks
    ///
    /// It returns `true` if the computed signature matches the expected signature. params is an
    /// ordered map containing all the query params Twilio added to the webhook URL.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// use rustlio::security;
    /// use std::collections::HashMap;
    /// use url::Url;
    ///
    /// let webhook_url = "https://96e5-165-225-114-134.ngrok-free.app/webhook?bodySHA256=e6ca4452daa092f8b0ecb9cdd24328f9b565196e0a25bc4e612bf198ad77fbd5";
    /// let mut url = match Url::parse(webhook_url) {
    ///     Ok(v) => v,
    ///     Err(_) => panic!("Could not parse URL"),
    /// };
    /// let params = HashMap::new();
    ///
    /// let validator = security::WebhookValidator {
    ///     auth_token: "<YOUR TWILIO AUTH TOKEN>".to_string(),
    /// };
    /// validator.validate(&mut url, &params, &"aU96RJE2IgIwrbsBNwQT5eaT1tM=".to_string());
    /// ```
    pub fn validate(
        &self,
        url: &mut Url,
        params: &HashMap<String, String>,
        expected_signature: &str,
    ) -> bool {
        if !params.is_empty() {
            let combined_params = self.sort_and_combine_params(params);
            url.set_query(Some(String::from_iter(combined_params).as_str()));
        }
        let computed_signature = self.compute_signature(self.remove_port(&mut url.clone()));

        self.compare(&computed_signature, expected_signature)
    }

    /// Validates a webhook's body
    ///
    /// It returns true if the computed SHA256 hash of the provided body matches the expected
    /// hash.
    fn validate_body_content(&self, body: &[u8], expected_hash: &str) -> bool {
        self.compare(&self.compute_body_hash(body), expected_hash)
    }

    /// Returns the provided params sorted by key and concatenated as a vector of key+value strings
    ///
    /// This is a utility function used by compute_signature. That function appends each of the
    /// returned params onto the end of the received webhook URI.
    fn sort_and_combine_params(&self, params: &HashMap<String, String>) -> Vec<String> {
        let mut sorted_params = BTreeMap::new();
        for (key, value) in params {
            sorted_params.insert(key.to_lowercase(), value);
        }

        sorted_params
            .iter()
            .map(|(k, v)| format!("{k}{v}"))
            .collect::<Vec<String>>()
    }

    /// Creates a SHA256 hash of the provided body and returns it as a lowercase hex string
    fn compute_body_hash(&self, body: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(body);
        hex::encode(hasher.finalize())
    }

    /// Creates the actual base64 encoded signature of the sha1 hash of the concatenated URL and your auth token
    ///
    /// The url is the full URL of the incoming (received) request.
    fn compute_signature(&self, url: &Url) -> String {
        let mut mac = Hmac::<Sha1>::new_from_slice(self.auth_token.as_bytes())
            .inspect_err(|e| info!("could not create mac: {e}"))
            .unwrap();
        mac.update(url.as_str().as_bytes());

        BASE64_STANDARD.encode(mac.finalize().into_bytes())
    }

    /// Perfoms a time-insensitive (constant time) comparison of the provided hashes
    ///
    /// ## Further reading
    ///
    /// - [Timing attack][timing_attack]
    /// - [Preventing Timing Attacks on String Comparison with a Double HMAC
    ///   Strategy][prevent_timing_attacks]
    /// - [Timing attacks in password hash comparisons][timing_attaks_password_hash_comparisons]
    ///
    /// [timing_attack]: https://en.wikipedia.org/wiki/Timing_attack
    /// [prevent_timing_attacks]: https://paragonie.com/blog/2015/11/preventing-timing-attacks-on-string-comparison-with-double-hmac-strategy
    /// [timing_attaks_password_hash_comparisons]: https://security.stackexchange.com/questions/239054/timing-attacks-in-password-hash-comparisons
    fn compare(&self, a: &str, b: &str) -> bool {
        if a.is_empty() || b.is_empty() {
            return false;
        }

        constant_time_eq(a.as_bytes(), b.as_bytes())
    }

    /// Removes the port value from the provided URL
    fn remove_port<'arr>(&self, url: &'arr mut Url) -> &'arr mut Url {
        url.set_port(None).expect("Unable to remove port from URL");

        url
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const AUTH_TOKEN: &str = "11111111111111111111111111111111";
    const VALID_EXPECTED_SIGNATURE: &str = "aU96RJE2IgIwrbsBNwQT5eaT1tM=";
    const URL: &str = "https://96e5-165-225-114-134.ngrok-free.app/webhook?bodySHA256=e6ca4452daa092f8b0ecb9cdd24328f9b565196e0a25bc4e612bf198ad77fbd5";

    #[test]
    fn can_successfully_validate_valid_request_which_has_a_json_request_body() {
        let validator = WebhookValidator {
            auth_token: AUTH_TOKEN.to_string(),
        };
        let mut url = match Url::parse(
            "https://96e5-165-225-114-134.ngrok-free.app/webhook?bodySHA256=f4dc494f1e604a19df36cdd069f7956b1f261dc954ae1c698ed8b4939fd9ab54",
        ) {
            Ok(v) => v,
            Err(_) => panic!("Could not parse URL"),
        };
        let request_body = r#"[{"specversion":"1.0","type":"com.twilio.eventstreams.test-event","source":"Sink","id":"AC11111111111111111111111111111111","dataschema":"https://events-schemas.twilio.com/EventStreams.TestSink/1.json","datacontenttype":"application/json","time":"2026-06-10T06:02:54.377Z","data":{"test_id":"cae2f9e2-c277-4612-8ad3-93c1a7a3ef88"}}]"#;
        assert!(validator.validate_body(
            &mut url,
            "qovYYTCoTFG3Ga6KmSrRVvSgwm8=",
            request_body.as_bytes()
        ));
    }

    #[test]
    fn can_successfully_validate_valid_request_which_has_a_form_request_body() {
        let request_url = "https://96e5-165-225-114-134.ngrok-free.app/webhook";
        let mut url = match Url::parse(request_url) {
            Ok(v) => v,
            Err(_) => panic!("Could not parse URL"),
        };
        let request_body = "name=matthew&day=thursday";
        let expected_signature = "UFAfJa118s+LlBrspFJHbn19Rtg=";

        let validator = WebhookValidator {
            auth_token: AUTH_TOKEN.to_string(),
        };
        assert!(validator.validate_body(&mut url, expected_signature, request_body.as_bytes()));
    }

    #[test]
    fn can_test_that_requests_with_no_body_with_invalid_signatures_fail_validation() {
        let validator = WebhookValidator {
            auth_token: AUTH_TOKEN.to_string(),
        };
        let mut url = match Url::parse(URL) {
            Ok(v) => v,
            Err(_) => panic!("Could not parse URL"),
        };

        let params = HashMap::new();
        assert!(!validator.validate(&mut url, &params, "aU96RJE2IgIwrbsBNw111111111="));
    }

    #[test]
    fn can_successfully_validate_valid_request_which_does_not_have_a_request_body() {
        let validator = WebhookValidator {
            auth_token: AUTH_TOKEN.to_string(),
        };
        let mut url = match Url::parse(URL) {
            Ok(v) => v,
            Err(_) => panic!("Could not parse URL"),
        };

        let params = HashMap::new();
        assert!(validator.validate(&mut url, &params, VALID_EXPECTED_SIGNATURE));
    }
}
