use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Settings for the Autonomous Linux service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutonomousSettings {
    /// Indicates whether Autonomous Linux will collect crash files. This setting can be changed by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_data_collection_authorized: Option<bool>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the restricted scheduled job associated with this instance. This value cannot be deleted by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduled_job_id: Option<String>,
}

impl AutonomousSettings {
    /// Create a new AutonomousSettings
    pub fn new() -> Self {
        Self {
            is_data_collection_authorized: None,

            scheduled_job_id: None,
        }
    }

    /// Set is_data_collection_authorized
    pub fn set_is_data_collection_authorized(mut self, value: Option<bool>) -> Self {
        self.is_data_collection_authorized = value;
        self
    }

    /// Set scheduled_job_id
    pub fn set_scheduled_job_id(mut self, value: Option<String>) -> Self {
        self.scheduled_job_id = value;
        self
    }

    /// Set is_data_collection_authorized (unwraps Option)
    pub fn with_is_data_collection_authorized(mut self, value: bool) -> Self {
        self.is_data_collection_authorized = Some(value);
        self
    }

    /// Set scheduled_job_id (unwraps Option)
    pub fn with_scheduled_job_id(mut self, value: impl Into<String>) -> Self {
        self.scheduled_job_id = Some(value.into());
        self
    }
}

impl Default for AutonomousSettings {
    fn default() -> Self {
        Self::new()
    }
}
