use serde::{Deserialize, Serialize};

/// Status of a managed instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManagedInstanceStatus {
    Normal,
    Unreachable,
    Error,
    Warning,
    Registration,
    Deleting,
    Deleted,
    Onboarding,
}

/// CPU architecture type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArchType {
    X8664,
    Aarch64,
    I686,
    Noarch,
    Src,
}

/// Operating system family
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OsFamily {
    OracleLinux9,
    OracleLinux8,
    OracleLinux7,
    OracleLinux6,
    Windows,
    All,
}

/// Location of managed instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManagedInstanceLocation {
    OnPremise,
    OciCompute,
    Azure,
    Ec2,
    Gcp,
}

/// Sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Sort by field for list managed instances
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListManagedInstancesSortBy {
    TimeCreated,
    DisplayName,
}

impl std::fmt::Display for ListManagedInstancesSortBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TimeCreated => write!(f, "timeCreated"),
            Self::DisplayName => write!(f, "displayName"),
        }
    }
}
