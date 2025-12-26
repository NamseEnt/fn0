use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstancePoolInstanceLifecycleState {
    #[serde(rename = "ATTACHING")]
    Attaching,

    #[serde(rename = "ACTIVE")]
    Active,

    #[serde(rename = "DETACHING")]
    Detaching,

    #[serde(rename = "TERMINATION_AWAIT")]
    TerminationAwait,

    #[serde(rename = "TERMINATION_PROCEED")]
    TerminationProceed,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
