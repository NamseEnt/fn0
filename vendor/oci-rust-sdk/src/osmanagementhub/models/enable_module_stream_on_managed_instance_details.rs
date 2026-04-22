use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to enable a module stream on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableModuleStreamOnManagedInstanceDetails {
    /// The name of a module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for EnableModuleStreamOnManagedInstanceDetails
pub struct EnableModuleStreamOnManagedInstanceDetailsRequired {
    /// The name of a module.
    pub module_name: String,
}

impl EnableModuleStreamOnManagedInstanceDetails {
    /// Create a new EnableModuleStreamOnManagedInstanceDetails with required fields
    pub fn new(required: EnableModuleStreamOnManagedInstanceDetailsRequired) -> Self {
        Self {
            module_name: required.module_name,

            stream_name: None,

            work_request_details: None,
        }
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set stream_name
    pub fn set_stream_name(mut self, value: Option<String>) -> Self {
        self.stream_name = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set stream_name (unwraps Option)
    pub fn with_stream_name(mut self, value: impl Into<String>) -> Self {
        self.stream_name = Some(value.into());
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}
