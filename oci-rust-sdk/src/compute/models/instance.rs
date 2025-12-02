use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::enums::*;

/// A compute instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    /// The OCID of the instance
    pub id: String,

    /// The OCID of the compartment that contains the instance
    pub compartment_id: String,

    /// The availability domain the instance is running in
    pub availability_domain: String,

    /// The shape of the instance
    pub shape: String,

    /// The region that contains the availability domain the instance is running in
    pub region: String,

    /// The current lifecycle state of the instance
    pub lifecycle_state: LifecycleState,

    /// The date and time the instance was created
    pub time_created: DateTime<Utc>,

    /// A user-friendly name for the instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// The OCID of the dedicated VM host that the instance is placed on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_vm_host_id: Option<String>,

    /// Custom metadata key/value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Additional metadata key/value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_metadata: Option<HashMap<String, serde_json::Value>>,

    /// The OCID of the image that the instance is running
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// The OCID of the boot volume attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boot_volume_id: Option<String>,

    /// Defined tags for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The OCID of the compute capacity reservation this instance is launched under
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// Fault domain for the instance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,
}
