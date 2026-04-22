use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A container object for response attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// The status code of the response. <p> Example: {@code 200}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// The time of the response to the audited request, expressed in [RFC 3339](https://tools.ietf.org/html/rfc3339) timestamp format. <p> Example: {@code 2019-09-18T00:10:59.278Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time: Option<DateTime<Utc>>,

    /// The headers of the response. <p> Example: <p> ----- { \"ETag\": [ \"<unique_ID>\" ], \"Connection\": [ \"close\" ], \"Content-Length\": [ \"1828\" ], \"opc-request-id\": [ \"<unique_ID>\" ], \"Date\": [ \"Wed, 18 Sep 2019 00:10:59 GMT\" ], \"Content-Type\": [ \"application/json\" ] } -----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Vec<String>>>,

    /// This value is included for backward compatibility with the Audit version 1 schema, where it contained metadata of interest from the response payload. <p> Example: <p> ----- { \"resourceName\": \"my_instance\", \"id\": \"ocid1.instance.oc1.phx.<unique_ID>\" } -----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<HashMap<String, serde_json::Value>>,

    /// A friendly description of what happened during the operation. Use this for troubleshooting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Response {
    /// Create a new Response
    pub fn new() -> Self {
        Self {
            status: None,

            response_time: None,

            headers: None,

            payload: None,

            message: None,
        }
    }

    /// Set status
    pub fn set_status(mut self, value: Option<String>) -> Self {
        self.status = value;
        self
    }

    /// Set response_time
    pub fn set_response_time(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.response_time = value;
        self
    }

    /// Set headers
    pub fn set_headers(mut self, value: Option<HashMap<String, Vec<String>>>) -> Self {
        self.headers = value;
        self
    }

    /// Set payload
    pub fn set_payload(mut self, value: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.payload = value;
        self
    }

    /// Set message
    pub fn set_message(mut self, value: Option<String>) -> Self {
        self.message = value;
        self
    }

    /// Set status (unwraps Option)
    pub fn with_status(mut self, value: impl Into<String>) -> Self {
        self.status = Some(value.into());
        self
    }

    /// Set response_time (unwraps Option)
    pub fn with_response_time(mut self, value: DateTime<Utc>) -> Self {
        self.response_time = Some(value);
        self
    }

    /// Set headers (unwraps Option)
    pub fn with_headers(mut self, value: HashMap<String, Vec<String>>) -> Self {
        self.headers = Some(value);
        self
    }

    /// Set payload (unwraps Option)
    pub fn with_payload(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.payload = Some(value);
        self
    }

    /// Set message (unwraps Option)
    pub fn with_message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}
