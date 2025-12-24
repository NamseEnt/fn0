use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LicensingConfigLicenseType {
    #[serde(rename = "OCI_PROVIDED")]
    OciProvided,

    #[serde(rename = "BRING_YOUR_OWN_LICENSE")]
    BringYourOwnLicense,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
