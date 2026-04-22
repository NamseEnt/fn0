use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Data related to the sysadmin event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysadminEventData {
    /// The commands executed by the agent that caused the error.
    pub error_cause: String,

    /// The output log of the error.
    pub error_log: String,

    /// The actions used to attempt fixing the error.
    pub attempted_resolutions: Vec<String>,

    /// Indicates if the event succeeded.
    pub resolution_status: EventStatus,

    /// The log output after the resolutions.
    pub resolution_log: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_details: Option<WorkRequestEventDataAdditionalDetails>,
}

/// Required fields for SysadminEventData
pub struct SysadminEventDataRequired {
    /// The commands executed by the agent that caused the error.
    pub error_cause: String,

    /// The output log of the error.
    pub error_log: String,

    /// The actions used to attempt fixing the error.
    pub attempted_resolutions: Vec<String>,

    /// Indicates if the event succeeded.
    pub resolution_status: EventStatus,

    /// The log output after the resolutions.
    pub resolution_log: String,
}

impl SysadminEventData {
    /// Create a new SysadminEventData with required fields
    pub fn new(required: SysadminEventDataRequired) -> Self {
        Self {
            error_cause: required.error_cause,

            error_log: required.error_log,

            attempted_resolutions: required.attempted_resolutions,

            resolution_status: required.resolution_status,

            resolution_log: required.resolution_log,

            additional_details: None,
        }
    }

    /// Set error_cause
    pub fn set_error_cause(mut self, value: String) -> Self {
        self.error_cause = value;
        self
    }

    /// Set error_log
    pub fn set_error_log(mut self, value: String) -> Self {
        self.error_log = value;
        self
    }

    /// Set attempted_resolutions
    pub fn set_attempted_resolutions(mut self, value: Vec<String>) -> Self {
        self.attempted_resolutions = value;
        self
    }

    /// Set resolution_status
    pub fn set_resolution_status(mut self, value: EventStatus) -> Self {
        self.resolution_status = value;
        self
    }

    /// Set resolution_log
    pub fn set_resolution_log(mut self, value: String) -> Self {
        self.resolution_log = value;
        self
    }

    /// Set additional_details
    pub fn set_additional_details(
        mut self,
        value: Option<WorkRequestEventDataAdditionalDetails>,
    ) -> Self {
        self.additional_details = value;
        self
    }

    /// Set additional_details (unwraps Option)
    pub fn with_additional_details(mut self, value: WorkRequestEventDataAdditionalDetails) -> Self {
        self.additional_details = Some(value);
        self
    }
}
