use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The measured boot report for a shielded instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeasuredBootReport {
    /// Whether the verification succeeded, and the new values match the expected values.
    pub is_policy_verification_successful: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub measurements: Option<MeasuredBootReportMeasurements>,
}

/// Required fields for MeasuredBootReport
pub struct MeasuredBootReportRequired {
    /// Whether the verification succeeded, and the new values match the expected values.
    pub is_policy_verification_successful: bool,
}

impl MeasuredBootReport {
    /// Create a new MeasuredBootReport with required fields
    pub fn new(required: MeasuredBootReportRequired) -> Self {
        Self {
            is_policy_verification_successful: required.is_policy_verification_successful,

            measurements: None,
        }
    }

    /// Set is_policy_verification_successful
    pub fn set_is_policy_verification_successful(mut self, value: bool) -> Self {
        self.is_policy_verification_successful = value;
        self
    }

    /// Set measurements
    pub fn set_measurements(mut self, value: Option<MeasuredBootReportMeasurements>) -> Self {
        self.measurements = value;
        self
    }

    /// Set measurements (unwraps Option)
    pub fn with_measurements(mut self, value: MeasuredBootReportMeasurements) -> Self {
        self.measurements = Some(value);
        self
    }
}
