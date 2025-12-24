use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An individual question that the customer can answer about the CPE device. <p> The customer provides answers to these questions in {@link #updateTunnelCpeDeviceConfig(UpdateTunnelCpeDeviceConfigRequest) updateTunnelCpeDeviceConfig}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpeDeviceConfigQuestion {
    /// A string that identifies the question.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// A descriptive label for the question (for example, to display in a form in a graphical interface). Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// A description or explanation of the question, to help the customer answer accurately.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

impl CpeDeviceConfigQuestion {
    /// Create a new CpeDeviceConfigQuestion
    pub fn new() -> Self {
        Self {
            key: None,

            display_name: None,

            explanation: None,
        }
    }

    /// Set key
    pub fn set_key(mut self, value: Option<String>) -> Self {
        self.key = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set explanation
    pub fn set_explanation(mut self, value: Option<String>) -> Self {
        self.explanation = value;
        self
    }

    /// Set key (unwraps Option)
    pub fn with_key(mut self, value: impl Into<String>) -> Self {
        self.key = Some(value.into());
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set explanation (unwraps Option)
    pub fn with_explanation(mut self, value: impl Into<String>) -> Self {
        self.explanation = Some(value.into());
        self
    }
}

impl Default for CpeDeviceConfigQuestion {
    fn default() -> Self {
        Self::new()
    }
}
