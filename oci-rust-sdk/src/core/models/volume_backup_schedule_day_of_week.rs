use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeBackupScheduleDayOfWeek {
    #[serde(rename = "MONDAY")]
    Monday,

    #[serde(rename = "TUESDAY")]
    Tuesday,

    #[serde(rename = "WEDNESDAY")]
    Wednesday,

    #[serde(rename = "THURSDAY")]
    Thursday,

    #[serde(rename = "FRIDAY")]
    Friday,

    #[serde(rename = "SATURDAY")]
    Saturday,

    #[serde(rename = "SUNDAY")]
    Sunday,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
