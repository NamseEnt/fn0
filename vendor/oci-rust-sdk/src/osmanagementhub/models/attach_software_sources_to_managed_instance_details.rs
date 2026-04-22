use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the information used to attach software sources to a managed instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachSoftwareSourcesToManagedInstanceDetails {
    /// The list of software source OCIDs to be attached/detached.
    pub software_sources: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_request_details: Option<WorkRequestDetails>,
}

/// Required fields for AttachSoftwareSourcesToManagedInstanceDetails
pub struct AttachSoftwareSourcesToManagedInstanceDetailsRequired {
    /// The list of software source OCIDs to be attached/detached.
    pub software_sources: Vec<String>,
}

impl AttachSoftwareSourcesToManagedInstanceDetails {
    /// Create a new AttachSoftwareSourcesToManagedInstanceDetails with required fields
    pub fn new(required: AttachSoftwareSourcesToManagedInstanceDetailsRequired) -> Self {
        Self {
            software_sources: required.software_sources,

            work_request_details: None,
        }
    }

    /// Set software_sources
    pub fn set_software_sources(mut self, value: Vec<String>) -> Self {
        self.software_sources = value;
        self
    }

    /// Set work_request_details
    pub fn set_work_request_details(mut self, value: Option<WorkRequestDetails>) -> Self {
        self.work_request_details = value;
        self
    }

    /// Set work_request_details (unwraps Option)
    pub fn with_work_request_details(mut self, value: WorkRequestDetails) -> Self {
        self.work_request_details = Some(value);
        self
    }
}
