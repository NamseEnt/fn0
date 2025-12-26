use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchInstanceResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: Option<String>,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the work request. Use [GetWorkRequest](https://docs.oracle.com/iaas/api/#/en/workrequests/latest/WorkRequest/GetWorkRequest) with this ID to track the status of the request.
    pub opc_work_request_id: Option<String>,

    /// The returned model.Instance instance.
    pub instance: Instance,
}

/// Required fields for LaunchInstanceResponse
pub struct LaunchInstanceResponseRequired {
    /// The returned model.Instance instance.
    pub instance: Instance,
}

impl LaunchInstanceResponse {
    /// Create a new LaunchInstanceResponse with required fields
    pub fn new(required: LaunchInstanceResponseRequired) -> Self {
        Self {
            etag: None,
            opc_request_id: None,
            opc_work_request_id: None,
            instance: required.instance,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: Option<String>) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_work_request_id
    pub fn set_opc_work_request_id(mut self, value: Option<String>) -> Self {
        self.opc_work_request_id = value;
        self
    }

    /// Set instance
    pub fn set_instance(mut self, value: Instance) -> Self {
        self.instance = value;
        self
    }

    /// Set etag (unwraps Option)
    pub fn with_etag(mut self, value: impl Into<String>) -> Self {
        self.etag = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }

    /// Set opc_work_request_id (unwraps Option)
    pub fn with_opc_work_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_work_request_id = Some(value.into());
        self
    }
}
