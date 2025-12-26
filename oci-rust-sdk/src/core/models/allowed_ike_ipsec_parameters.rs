use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Lists the current allowed and default IPSec tunnel parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedIkeIPSecParameters {
    pub allowed_phase_one_parameters: AllowedPhaseOneParameters,

    pub allowed_phase_two_parameters: AllowedPhaseTwoParameters,

    pub default_phase_one_parameters: DefaultPhaseOneParameters,

    pub default_phase_two_parameters: DefaultPhaseTwoParameters,
}

/// Required fields for AllowedIkeIPSecParameters
pub struct AllowedIkeIPSecParametersRequired {
    pub allowed_phase_one_parameters: AllowedPhaseOneParameters,

    pub allowed_phase_two_parameters: AllowedPhaseTwoParameters,

    pub default_phase_one_parameters: DefaultPhaseOneParameters,

    pub default_phase_two_parameters: DefaultPhaseTwoParameters,
}

impl AllowedIkeIPSecParameters {
    /// Create a new AllowedIkeIPSecParameters with required fields
    pub fn new(required: AllowedIkeIPSecParametersRequired) -> Self {
        Self {
            allowed_phase_one_parameters: required.allowed_phase_one_parameters,

            allowed_phase_two_parameters: required.allowed_phase_two_parameters,

            default_phase_one_parameters: required.default_phase_one_parameters,

            default_phase_two_parameters: required.default_phase_two_parameters,
        }
    }

    /// Set allowed_phase_one_parameters
    pub fn set_allowed_phase_one_parameters(mut self, value: AllowedPhaseOneParameters) -> Self {
        self.allowed_phase_one_parameters = value;
        self
    }

    /// Set allowed_phase_two_parameters
    pub fn set_allowed_phase_two_parameters(mut self, value: AllowedPhaseTwoParameters) -> Self {
        self.allowed_phase_two_parameters = value;
        self
    }

    /// Set default_phase_one_parameters
    pub fn set_default_phase_one_parameters(mut self, value: DefaultPhaseOneParameters) -> Self {
        self.default_phase_one_parameters = value;
        self
    }

    /// Set default_phase_two_parameters
    pub fn set_default_phase_two_parameters(mut self, value: DefaultPhaseTwoParameters) -> Self {
        self.default_phase_two_parameters = value;
        self
    }
}
