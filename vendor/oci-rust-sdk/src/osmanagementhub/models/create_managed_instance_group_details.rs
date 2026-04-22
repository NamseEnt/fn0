use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Provides the information used to create a new managed instance group.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManagedInstanceGroupDetails {
    /// A user-friendly name for the managed instance group. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance group.
    pub compartment_id: String,

    /// The operating system type of the managed instances that will be attached to this group.
    pub os_family: OsFamily,

    /// The CPU architecture type of the managed instances that will be attached to this group.
    pub arch_type: ArchType,

    /// The vendor of the operating system that will be used by the managed instances in the group.
    pub vendor_name: VendorName,

    /// User-specified description of the managed instance group. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The location of managed instances attached to the group. If no location is provided, the default is on premises.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ManagedInstanceLocation>,

    /// The list of software source [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) available to the managed instances in the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_source_ids: Option<Vec<String>>,

    /// The list of managed instance [OCIDs](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) to be added to the group.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_instance_ids: Option<Vec<String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) for the Oracle Notifications service (ONS) topic. ONS is the channel used to send notifications to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notification_topic_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_settings: Option<UpdatableAutonomousSettings>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
}

/// Required fields for CreateManagedInstanceGroupDetails
pub struct CreateManagedInstanceGroupDetailsRequired {
    /// A user-friendly name for the managed instance group. Does not have to be unique and you can change the name later. Avoid entering confidential information.
    pub display_name: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment that contains the managed instance group.
    pub compartment_id: String,

    /// The operating system type of the managed instances that will be attached to this group.
    pub os_family: OsFamily,

    /// The CPU architecture type of the managed instances that will be attached to this group.
    pub arch_type: ArchType,

    /// The vendor of the operating system that will be used by the managed instances in the group.
    pub vendor_name: VendorName,
}

impl CreateManagedInstanceGroupDetails {
    /// Create a new CreateManagedInstanceGroupDetails with required fields
    pub fn new(required: CreateManagedInstanceGroupDetailsRequired) -> Self {
        Self {
            display_name: required.display_name,

            compartment_id: required.compartment_id,

            os_family: required.os_family,

            arch_type: required.arch_type,

            vendor_name: required.vendor_name,

            description: None,

            location: None,

            software_source_ids: None,

            managed_instance_ids: None,

            notification_topic_id: None,

            autonomous_settings: None,

            freeform_tags: None,

            defined_tags: None,
        }
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set description
    pub fn set_description(mut self, value: Option<String>) -> Self {
        self.description = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set os_family
    pub fn set_os_family(mut self, value: OsFamily) -> Self {
        self.os_family = value;
        self
    }

    /// Set arch_type
    pub fn set_arch_type(mut self, value: ArchType) -> Self {
        self.arch_type = value;
        self
    }

    /// Set vendor_name
    pub fn set_vendor_name(mut self, value: VendorName) -> Self {
        self.vendor_name = value;
        self
    }

    /// Set location
    pub fn set_location(mut self, value: Option<ManagedInstanceLocation>) -> Self {
        self.location = value;
        self
    }

    /// Set software_source_ids
    pub fn set_software_source_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.software_source_ids = value;
        self
    }

    /// Set managed_instance_ids
    pub fn set_managed_instance_ids(mut self, value: Option<Vec<String>>) -> Self {
        self.managed_instance_ids = value;
        self
    }

    /// Set notification_topic_id
    pub fn set_notification_topic_id(mut self, value: Option<String>) -> Self {
        self.notification_topic_id = value;
        self
    }

    /// Set autonomous_settings
    pub fn set_autonomous_settings(mut self, value: Option<UpdatableAutonomousSettings>) -> Self {
        self.autonomous_settings = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set description (unwraps Option)
    pub fn with_description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set location (unwraps Option)
    pub fn with_location(mut self, value: ManagedInstanceLocation) -> Self {
        self.location = Some(value);
        self
    }

    /// Set software_source_ids (unwraps Option)
    pub fn with_software_source_ids(mut self, value: Vec<String>) -> Self {
        self.software_source_ids = Some(value);
        self
    }

    /// Set managed_instance_ids (unwraps Option)
    pub fn with_managed_instance_ids(mut self, value: Vec<String>) -> Self {
        self.managed_instance_ids = Some(value);
        self
    }

    /// Set notification_topic_id (unwraps Option)
    pub fn with_notification_topic_id(mut self, value: impl Into<String>) -> Self {
        self.notification_topic_id = Some(value.into());
        self
    }

    /// Set autonomous_settings (unwraps Option)
    pub fn with_autonomous_settings(mut self, value: UpdatableAutonomousSettings) -> Self {
        self.autonomous_settings = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }
}
