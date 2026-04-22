use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Provides the name and description of the job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkRequestDetails {
    /// A user-friendly name for the job. The name does not have to be unique. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// User-specified information about the job. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl WorkRequestDetails {
    /// Create a new WorkRequestDetails
    pub fn new() -> Self {
        Self {
            display_name: None,

            description: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
}

impl Default for WorkRequestDetails {
    fn default() -> Self {
        Self::new()
    }
}
