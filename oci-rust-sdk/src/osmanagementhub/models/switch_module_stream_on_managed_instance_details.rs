use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to switch module streams on a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchModuleStreamOnManagedInstanceDetails {
    /// The name of a module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    pub stream_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that contains the module stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,
}

/// Required fields for SwitchModuleStreamOnManagedInstanceDetails
pub struct SwitchModuleStreamOnManagedInstanceDetailsRequired {
    /// The name of a module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    pub stream_name: String,
}

impl SwitchModuleStreamOnManagedInstanceDetails {
    /// Create a new SwitchModuleStreamOnManagedInstanceDetails with required fields
    pub fn new(required: SwitchModuleStreamOnManagedInstanceDetailsRequired) -> Self {
        Self {
            module_name: required.module_name,

            stream_name: required.stream_name,

            work_request_details: None,

            software_source_id: None,
        }
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set module_name
    pub fn set_module_name(mut self, value: String) -> Self {
        self.module_name = value;
        self
    }

    /// Set stream_name
    pub fn set_stream_name(mut self, value: String) -> Self {
        self.stream_name = value;
        self
    }

    /// Set software_source_id
    pub fn set_software_source_id(mut self, value: Option<String>) -> Self {
        self.software_source_id = value;
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }
}
