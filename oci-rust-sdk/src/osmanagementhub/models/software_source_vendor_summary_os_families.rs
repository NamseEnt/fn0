use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoftwareSourceVendorSummaryOsFamilies {
    #[serde(rename = "ORACLE_LINUX_9")]
    OracleLinux9,

    #[serde(rename = "ORACLE_LINUX_8")]
    OracleLinux8,

    #[serde(rename = "ORACLE_LINUX_7")]
    OracleLinux7,

    #[serde(rename = "ORACLE_LINUX_6")]
    OracleLinux6,

    #[serde(rename = "WINDOWS_SERVER_2016")]
    WindowsServer2016,

    #[serde(rename = "WINDOWS_SERVER_2019")]
    WindowsServer2019,

    #[serde(rename = "WINDOWS_SERVER_2022")]
    WindowsServer2022,

    #[serde(rename = "ALL")]
    All,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
