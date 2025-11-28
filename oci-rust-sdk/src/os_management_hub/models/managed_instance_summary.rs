use serde::{Deserialize, Serialize};
use super::enums::*;

/// Summary information for a managed instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedInstanceSummary {
    /// The OCID of the managed instance
    pub id: String,

    /// User-friendly name for the managed instance
    pub display_name: String,

    /// User-specified description of the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The OCID of the tenancy this managed instance resides in
    pub tenancy_id: String,

    /// The OCID of the compartment that contains the managed instance
    pub compartment_id: String,

    /// The location of the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// The CPU architecture type of the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ArchType>,

    /// The operating system type of the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub os_family: Option<OsFamily>,

    /// Current status of the managed instance
    pub status: ManagedInstanceStatus,

    /// Managed instance group details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_group: Option<ResourceId>,

    /// Lifecycle environment details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_environment: Option<ResourceId>,

    /// Lifecycle stage details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_stage: Option<ResourceId>,

    /// Indicates whether a reboot is required to complete installation of updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_reboot_required: Option<bool>,

    /// Number of updates available for installation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates_available: Option<i32>,

    /// Whether this managed instance is acting as an on-premises management station
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_management_station: Option<bool>,

    /// The OCID for the Oracle Notifications service (ONS) topic
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_topic_id: Option<String>,

    /// Autonomous settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_settings: Option<AutonomousSettings>,

    /// Indicates whether Autonomous Linux manages this instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_managed_by_autonomous_linux: Option<bool>,

    /// The version of osmh-agent running on the managed instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_version: Option<String>,
}

/// Resource ID reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceId {
    /// The OCID of the resource
    pub id: String,

    /// Display name of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Autonomous settings for a managed instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutonomousSettings {
    /// Indicates whether Autonomous Linux will manage the instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_data_collection_authorized: Option<bool>,
}
