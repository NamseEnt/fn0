use serde::{Deserialize, Serialize};

/// Location of the managed instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManagedInstanceLocation {
    #[serde(rename = "ON_PREMISE")]
    OnPremise,

    #[serde(rename = "OCI_COMPUTE")]
    OciCompute,

    #[serde(rename = "AZURE")]
    Azure,

    #[serde(rename = "EC2")]
    Ec2,

    #[serde(rename = "GCP")]
    Gcp,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
