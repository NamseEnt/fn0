use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the details for switching module streams on the managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchModuleStreamOnManagedInstanceGroupDetails {
    /// The name of the module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    pub stream_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the software source that provides the module stream
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for SwitchModuleStreamOnManagedInstanceGroupDetails
pub struct SwitchModuleStreamOnManagedInstanceGroupDetailsRequired {
    /// The name of the module.
    pub module_name: String,

    /// The name of a stream of the specified module.
    pub stream_name: String,
}

impl SwitchModuleStreamOnManagedInstanceGroupDetails {
    /// Create a new SwitchModuleStreamOnManagedInstanceGroupDetails with required fields
    pub fn new(required: SwitchModuleStreamOnManagedInstanceGroupDetailsRequired) -> Self {
        Self {
            module_name: required.module_name,

            stream_name: required.stream_name,

            software_source_id: None,

            work_request_details: None,
        }
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

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set software_source_id (unwraps Option)
    pub fn with_software_source_id(mut self, value: impl Into<String>) -> Self {
        self.software_source_id = Some(value.into());
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}
