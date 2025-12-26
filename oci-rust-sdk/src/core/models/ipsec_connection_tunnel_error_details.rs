use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Details for an error on an IPSec tunnel.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IPSecConnectionTunnelErrorDetails {
    /// Unique ID generated for each error report.
    pub id: String,

    /// Unique code describes the error type.
    pub error_code: String,

    /// A detailed description of the error.
    pub error_description: String,

    /// Resolution for the error.
    pub solution: String,

    /// Link to more Oracle resources or relevant documentation.
    pub oci_resources_link: String,

    /// Timestamp when the error occurred.
    pub timestamp: DateTime<Utc>,
}

/// Required fields for IPSecConnectionTunnelErrorDetails
pub struct IPSecConnectionTunnelErrorDetailsRequired {
    /// Unique ID generated for each error report.
    pub id: String,

    /// Unique code describes the error type.
    pub error_code: String,

    /// A detailed description of the error.
    pub error_description: String,

    /// Resolution for the error.
    pub solution: String,

    /// Link to more Oracle resources or relevant documentation.
    pub oci_resources_link: String,

    /// Timestamp when the error occurred.
    pub timestamp: DateTime<Utc>,
}

impl IPSecConnectionTunnelErrorDetails {
    /// Create a new IPSecConnectionTunnelErrorDetails with required fields
    pub fn new(required: IPSecConnectionTunnelErrorDetailsRequired) -> Self {
        Self {
            id: required.id,

            error_code: required.error_code,

            error_description: required.error_description,

            solution: required.solution,

            oci_resources_link: required.oci_resources_link,

            timestamp: required.timestamp,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set error_code
    pub fn set_error_code(mut self, value: String) -> Self {
        self.error_code = value;
        self
    }

    /// Set error_description
    pub fn set_error_description(mut self, value: String) -> Self {
        self.error_description = value;
        self
    }

    /// Set solution
    pub fn set_solution(mut self, value: String) -> Self {
        self.solution = value;
        self
    }

    /// Set oci_resources_link
    pub fn set_oci_resources_link(mut self, value: String) -> Self {
        self.oci_resources_link = value;
        self
    }

    /// Set timestamp
    pub fn set_timestamp(mut self, value: DateTime<Utc>) -> Self {
        self.timestamp = value;
        self
    }
}
