use serde::{Deserialize, Serialize};

/// The type of resources that a work request can act on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetResourceEntityType {
    #[serde(rename = "INSTANCE")]
    Instance,

    #[serde(rename = "GROUP")]
    Group,

    #[serde(rename = "COMPARTMENT")]
    Compartment,

    #[serde(rename = "LIFECYCLE_ENVIRONMENT")]
    LifecycleEnvironment,

    #[serde(rename = "SOFTWARE_SOURCE")]
    SoftwareSource,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
