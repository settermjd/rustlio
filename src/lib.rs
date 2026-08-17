//! rustlio simplifies interacting with Twilio's APIs in Rust applications.

pub mod lookup;
pub mod messaging;
pub mod security;

use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

trait ApiRequest {
    async fn make_post_request(
        &self,
        request_url: &str,
        request_params: &HashMap<String, String>,
    ) -> Result<Response, reqwest::Error>;
}

/// TwilioRestClient encapsulates a Reqwest Client that makes requests to Twilio's APIs
#[derive(Debug, Default)]
pub struct TwilioRestClient<'a> {
    pub account_sid: &'a str,
    pub auth_token: &'a str,
}

#[derive(Debug, Serialize)]
pub enum RequestValue {
    Str(String),
    Int(i32),
}

impl<'a> ApiRequest for TwilioRestClient<'a> {
    /// make_post_request makes async POST requests using a Reqwest Client
    ///
    /// As each of Twilio's API endpoints can have different domains and/or different paths, the
    /// request URL must be provided to the function. That aside, the function takes an optional
    /// HashMap of request parameters to be used with the request and returns the response from
    /// making the request.
    async fn make_post_request(
        &self,
        request_url: &str,
        request_params: &HashMap<String, String>,
    ) -> Result<Response, reqwest::Error> {
        let client = &Client::new();
        let mut request_builder = client
            .post(request_url)
            .basic_auth(self.account_sid, Some(self.auth_token));

        if !request_params.is_empty() {
            println!("Request params: {:?}", request_params);
            request_builder = request_builder.form(&request_params);
        }

        request_builder.send().await
    }
}
