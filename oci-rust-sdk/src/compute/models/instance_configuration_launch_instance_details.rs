use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Instance launch details for creating an instance from an instance configuration.
/// Use the sourceDetails parameter to specify whether a boot volume or an image
/// should be used to launch a new instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchInstanceDetails {
    /// The availability domain of the instance.
    /// Example: `Uocm:PHX-AD-1`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The OCID of the compute capacity reservation this instance is launched under.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// Whether to enable AI enterprise on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_ai_enterprise_enabled: Option<bool>,

    /// Placement constraint details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_constraint_details: Option<super::InstanceConfigurationHostGroupPlacementConstraintDetails>,

    /// The OCID of the compute cluster that the instance will be created in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_cluster_id: Option<String>,

    /// The OCID of the compartment containing the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The OCID of the cluster placement group of the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// VNIC creation details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_vnic_details: Option<super::InstanceConfigurationCreateVnicDetails>,

    /// Defined tags for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Security attributes for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Additional metadata key/value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_metadata: Option<HashMap<String, serde_json::Value>>,

    /// Free-form tags for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Custom iPXE script for boot process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipxe_script: Option<String>,

    /// Custom metadata key/value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// The shape of an instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    /// Shape configuration details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_config: Option<super::InstanceConfigurationLaunchInstanceShapeConfigDetails>,

    /// Platform configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_config: Option<super::InstanceConfigurationLaunchInstancePlatformConfig>,

    /// Source details for the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<super::InstanceConfigurationInstanceSourceDetails>,

    /// A fault domain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// The OCID of the dedicated virtual machine host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_vm_host_id: Option<String>,

    /// Configuration mode for launching VM instances.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<LaunchMode>,

    /// Launch options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_options: Option<super::InstanceConfigurationLaunchOptions>,

    /// Agent configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<super::InstanceConfigurationLaunchInstanceAgentConfigDetails>,

    /// Whether to enable in-transit encryption for paravirtualized attachment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,

    /// The preferred maintenance action for an instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_maintenance_action: Option<PreferredMaintenanceAction>,

    /// Instance options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_options: Option<super::InstanceConfigurationInstanceOptions>,

    /// Availability configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_config: Option<super::InstanceConfigurationAvailabilityConfig>,

    /// Preemptible instance configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preemptible_instance_config: Option<super::PreemptibleInstanceConfigDetails>,

    /// List of licensing configurations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licensing_configs: Option<Vec<super::LaunchInstanceLicensingConfig>>,
}

/// Launch mode for VM instances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LaunchMode {
    /// VM instances launch with iSCSI boot and VFIO devices.
    Native,
    /// VM instances launch with emulated devices.
    Emulated,
    /// VM instances launch with paravirtualized devices using VirtIO drivers.
    Paravirtualized,
    /// VM instances launch with custom configuration settings.
    Custom,
}

/// Preferred maintenance action for an instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PreferredMaintenanceAction {
    /// Run maintenance using a live migration.
    LiveMigrate,
    /// Run maintenance using a reboot.
    Reboot,
}

impl InstanceConfigurationLaunchInstanceDetails {
    pub fn new() -> Self {
        Self {
            availability_domain: None,
            capacity_reservation_id: None,
            is_ai_enterprise_enabled: None,
            placement_constraint_details: None,
            compute_cluster_id: None,
            compartment_id: None,
            cluster_placement_group_id: None,
            create_vnic_details: None,
            defined_tags: None,
            security_attributes: None,
            display_name: None,
            extended_metadata: None,
            freeform_tags: None,
            ipxe_script: None,
            metadata: None,
            shape: None,
            shape_config: None,
            platform_config: None,
            source_details: None,
            fault_domain: None,
            dedicated_vm_host_id: None,
            launch_mode: None,
            launch_options: None,
            agent_config: None,
            is_pv_encryption_in_transit_enabled: None,
            preferred_maintenance_action: None,
            instance_options: None,
            availability_config: None,
            preemptible_instance_config: None,
            licensing_configs: None,
        }
    }

    // Builder methods
    pub fn with_availability_domain(mut self, availability_domain: impl Into<String>) -> Self {
        self.availability_domain = Some(availability_domain.into());
        self
    }

    pub fn with_compartment_id(mut self, compartment_id: impl Into<String>) -> Self {
        self.compartment_id = Some(compartment_id.into());
        self
    }

    pub fn with_shape(mut self, shape: impl Into<String>) -> Self {
        self.shape = Some(shape.into());
        self
    }

    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_freeform_tags(mut self, tags: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(tags);
        self
    }

    pub fn with_defined_tags(mut self, tags: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.defined_tags = Some(tags);
        self
    }

    pub fn with_fault_domain(mut self, fault_domain: impl Into<String>) -> Self {
        self.fault_domain = Some(fault_domain.into());
        self
    }

    pub fn with_launch_mode(mut self, launch_mode: LaunchMode) -> Self {
        self.launch_mode = Some(launch_mode);
        self
    }

    pub fn with_preferred_maintenance_action(mut self, action: PreferredMaintenanceAction) -> Self {
        self.preferred_maintenance_action = Some(action);
        self
    }
}

impl Default for InstanceConfigurationLaunchInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
