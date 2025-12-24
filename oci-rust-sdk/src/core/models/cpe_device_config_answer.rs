use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An individual answer to a CPE device question. <p> The answers correlate to the questions that are specific to the CPE device type (see the {@code parameters} attribute of {@link CpeDeviceShapeDetail}).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpeDeviceConfigAnswer {
    /// A string that identifies the question to be answered. See the {@code key} attribute in {@link #cpeDeviceConfigQuestion(CpeDeviceConfigQuestionRequest) cpeDeviceConfigQuestion}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// The answer to the question.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

impl CpeDeviceConfigAnswer {
    /// Create a new CpeDeviceConfigAnswer
    pub fn new() -> Self {
        Self {
            key: None,

            value: None,
        }
    }

    /// Set key
    pub fn set_key(mut self, value: Option<String>) -> Self {
        self.key = value;
        self
    }

    /// Set value
    pub fn set_value(mut self, value: Option<String>) -> Self {
        self.value = value;
        self
    }

    /// Set key (unwraps Option)
    pub fn with_key(mut self, value: impl Into<String>) -> Self {
        self.key = Some(value.into());
        self
    }

    /// Set value (unwraps Option)
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

impl Default for CpeDeviceConfigAnswer {
    fn default() -> Self {
        Self::new()
    }
}
