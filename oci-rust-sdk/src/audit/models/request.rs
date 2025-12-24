use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A container object for request attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The opc-request-id of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The full path of the API request. <p> Example: {@code /20160918/instances/ocid1.instance.oc1.phx.<unique_ID>}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// The HTTP method of the request. <p> Example: {@code GET}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// The parameters supplied by the caller during this operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, Vec<String>>>,

    /// The HTTP header fields and values in the request. <p> Example: <p> ----- { \"opc-principal\": [ \"{\\\"tenantId\\\":\\\"ocid1.tenancy.oc1..<unique_ID>\\\",\\\"subjectId\\\":\\\"ocid1.user.oc1..<unique_ID>\\\",\\\"claims\\\":[{\\\"key\\\":\\\"pstype\\\",\\\"value\\\":\\\"natv\\\",\\\"issuer\\\":\\\"authService.oracle.com\\\"},{\\\"key\\\":\\\"h_host\\\",\\\"value\\\":\\\"iaas.r2.oracleiaas.com\\\",\\\"issuer\\\":\\\"h\\\"},{\\\"key\\\":\\\"h_opc-request-id\\\",\\\"value\\\":\\\"<unique_ID>\\\",\\\"issuer\\\":\\\"h\\\"},{\\\"key\\\":\\\"ptype\\\",\\\"value\\\":\\\"user\\\",\\\"issuer\\\":\\\"authService.oracle.com\\\"},{\\\"key\\\":\\\"h_date\\\",\\\"value\\\":\\\"Wed, 18 Sep 2019 00:10:58 UTC\\\",\\\"issuer\\\":\\\"h\\\"},{\\\"key\\\":\\\"h_accept\\\",\\\"value\\\":\\\"application/json\\\",\\\"issuer\\\":\\\"h\\\"},{\\\"key\\\":\\\"authorization\\\",\\\"value\\\":\\\"Signature headers=\\\\\\\"date (request-target) host accept opc-request-id\\\\\\\",keyId=\\\\\\\"ocid1.tenancy.oc1..<unique_ID>/ocid1.user.oc1..<unique_ID>/8c:b4:5f:18:e7:ec:db:08:b8:fa:d2:2a:7d:11:76:ac\\\\\\\",algorithm=\\\\\\\"rsa-pss-sha256\\\\\\\",signature=\\\\\\\"<unique_ID>\\\\\\\",version=\\\\\\\"1\\\\\\\"\\\",\\\"issuer\\\":\\\"h\\\"},{\\\"key\\\":\\\"h_(request-target)\\\",\\\"value\\\":\\\"get /20160918/instances/ocid1.instance.oc1.phx.<unique_ID>\\\",\\\"issuer\\\":\\\"h\\\"}]}\" ], \"Accept\": [ \"application/json\" ], \"X-Oracle-Auth-Client-CN\": [ \"splat-proxy-se-02302.node.ad2.r2\" ], \"X-Forwarded-Host\": [ \"compute-api.svc.ad1.r2\" ], \"Connection\": [ \"close\" ], \"User-Agent\": [ \"Jersey/2.23 (HttpUrlConnection 1.8.0_212)\" ], \"X-Forwarded-For\": [ \"172.24.80.88\" ], \"X-Real-IP\": [ \"172.24.80.88\" ], \"oci-original-url\": [ \"https://iaas.r2.oracleiaas.com/20160918/instances/ocid1.instance.oc1.phx.<unique_ID>\" ], \"opc-request-id\": [ \"<unique_ID>\" ], \"Date\": [ \"Wed, 18 Sep 2019 00:10:58 UTC\" ] } -----
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, Vec<String>>>,
}

impl Request {
    /// Create a new Request
    pub fn new() -> Self {
        Self {
            id: None,

            path: None,

            action: None,

            parameters: None,

            headers: None,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set path
    pub fn set_path(mut self, value: Option<String>) -> Self {
        self.path = value;
        self
    }

    /// Set action
    pub fn set_action(mut self, value: Option<String>) -> Self {
        self.action = value;
        self
    }

    /// Set parameters
    pub fn set_parameters(mut self, value: Option<HashMap<String, Vec<String>>>) -> Self {
        self.parameters = value;
        self
    }

    /// Set headers
    pub fn set_headers(mut self, value: Option<HashMap<String, Vec<String>>>) -> Self {
        self.headers = value;
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set path (unwraps Option)
    pub fn with_path(mut self, value: impl Into<String>) -> Self {
        self.path = Some(value.into());
        self
    }

    /// Set action (unwraps Option)
    pub fn with_action(mut self, value: impl Into<String>) -> Self {
        self.action = Some(value.into());
        self
    }

    /// Set parameters (unwraps Option)
    pub fn with_parameters(mut self, value: HashMap<String, Vec<String>>) -> Self {
        self.parameters = Some(value);
        self
    }

    /// Set headers (unwraps Option)
    pub fn with_headers(mut self, value: HashMap<String, Vec<String>>) -> Self {
        self.headers = Some(value);
        self
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}
