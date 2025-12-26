use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A list of Trusted Platform Module (TPM) Platform Configuration Register (PCR) entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasuredBootReportMeasurements {
    /// The list of expected PCR entries to use during verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<Vec<MeasuredBootEntry>>,

    /// The list of actual PCR entries measured during boot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<Vec<MeasuredBootEntry>>,
}

impl MeasuredBootReportMeasurements {
    /// Create a new MeasuredBootReportMeasurements
    pub fn new() -> Self {
        Self {
            policy: None,

            actual: None,
        }
    }

    /// Set policy
    pub fn set_policy(mut self, value: Option<Vec<MeasuredBootEntry>>) -> Self {
        self.policy = value;
        self
    }

    /// Set actual
    pub fn set_actual(mut self, value: Option<Vec<MeasuredBootEntry>>) -> Self {
        self.actual = value;
        self
    }

    /// Set policy (unwraps Option)
    pub fn with_policy(mut self, value: Vec<MeasuredBootEntry>) -> Self {
        self.policy = Some(value);
        self
    }

    /// Set actual (unwraps Option)
    pub fn with_actual(mut self, value: Vec<MeasuredBootEntry>) -> Self {
        self.actual = Some(value);
        self
    }
}

impl Default for MeasuredBootReportMeasurements {
    fn default() -> Self {
        Self::new()
    }
}
