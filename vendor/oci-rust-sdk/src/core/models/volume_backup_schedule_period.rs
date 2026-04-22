use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VolumeBackupSchedulePeriod {
    #[serde(rename = "ONE_HOUR")]
    OneHour,

    #[serde(rename = "ONE_DAY")]
    OneDay,

    #[serde(rename = "ONE_WEEK")]
    OneWeek,

    #[serde(rename = "ONE_MONTH")]
    OneMonth,

    #[serde(rename = "ONE_YEAR")]
    OneYear,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
