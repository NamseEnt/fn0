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

    pub fn with_source_details(mut self, details: super::InstanceConfigurationInstanceSourceDetails) -> Self {
        self.source_details = Some(details);
        self
    }

    pub fn with_shape_config(mut self, config: super::InstanceConfigurationLaunchInstanceShapeConfigDetails) -> Self {
        self.shape_config = Some(config);
        self
    }

    pub fn with_create_vnic_details(mut self, details: super::InstanceConfigurationCreateVnicDetails) -> Self {
        self.create_vnic_details = Some(details);
        self
    }

    pub fn with_capacity_reservation_id(mut self, capacity_reservation_id: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(capacity_reservation_id.into());
        self
    }

    pub fn with_is_ai_enterprise_enabled(mut self, is_ai_enterprise_enabled: bool) -> Self {
        self.is_ai_enterprise_enabled = Some(is_ai_enterprise_enabled);
        self
    }

    pub fn with_placement_constraint_details(mut self, placement_constraint_details: super::InstanceConfigurationHostGroupPlacementConstraintDetails) -> Self {
        self.placement_constraint_details = Some(placement_constraint_details);
        self
    }

    pub fn with_compute_cluster_id(mut self, compute_cluster_id: impl Into<String>) -> Self {
        self.compute_cluster_id = Some(compute_cluster_id.into());
        self
    }

    pub fn with_cluster_placement_group_id(mut self, cluster_placement_group_id: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(cluster_placement_group_id.into());
        self
    }

    pub fn with_security_attributes(mut self, security_attributes: HashMap<String, HashMap<String, serde_json::Value>>) -> Self {
        self.security_attributes = Some(security_attributes);
        self
    }

    pub fn with_extended_metadata(mut self, extended_metadata: HashMap<String, serde_json::Value>) -> Self {
        self.extended_metadata = Some(extended_metadata);
        self
    }

    pub fn with_ipxe_script(mut self, ipxe_script: impl Into<String>) -> Self {
        self.ipxe_script = Some(ipxe_script.into());
        self
    }

    pub fn with_platform_config(mut self, platform_config: super::InstanceConfigurationLaunchInstancePlatformConfig) -> Self {
        self.platform_config = Some(platform_config);
        self
    }

    pub fn with_dedicated_vm_host_id(mut self, dedicated_vm_host_id: impl Into<String>) -> Self {
        self.dedicated_vm_host_id = Some(dedicated_vm_host_id.into());
        self
    }

    pub fn with_launch_options(mut self, launch_options: super::InstanceConfigurationLaunchOptions) -> Self {
        self.launch_options = Some(launch_options);
        self
    }

    pub fn with_agent_config(mut self, agent_config: super::InstanceConfigurationLaunchInstanceAgentConfigDetails) -> Self {
        self.agent_config = Some(agent_config);
        self
    }

    pub fn with_is_pv_encryption_in_transit_enabled(mut self, is_pv_encryption_in_transit_enabled: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(is_pv_encryption_in_transit_enabled);
        self
    }

    pub fn with_instance_options(mut self, instance_options: super::InstanceConfigurationInstanceOptions) -> Self {
        self.instance_options = Some(instance_options);
        self
    }

    pub fn with_availability_config(mut self, availability_config: super::InstanceConfigurationAvailabilityConfig) -> Self {
        self.availability_config = Some(availability_config);
        self
    }

    pub fn with_preemptible_instance_config(mut self, preemptible_instance_config: super::PreemptibleInstanceConfigDetails) -> Self {
        self.preemptible_instance_config = Some(preemptible_instance_config);
        self
    }

    pub fn with_licensing_configs(mut self, licensing_configs: Vec<super::LaunchInstanceLicensingConfig>) -> Self {
        self.licensing_configs = Some(licensing_configs);
        self
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, availability_domain: Option<String>) -> Self {
        self.availability_domain = availability_domain;
        self
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, capacity_reservation_id: Option<String>) -> Self {
        self.capacity_reservation_id = capacity_reservation_id;
        self
    }

    /// Set is_ai_enterprise_enabled
    pub fn set_is_ai_enterprise_enabled(mut self, is_ai_enterprise_enabled: Option<bool>) -> Self {
        self.is_ai_enterprise_enabled = is_ai_enterprise_enabled;
        self
    }

    /// Set placement_constraint_details
    pub fn set_placement_constraint_details(mut self, placement_constraint_details: Option<super::InstanceConfigurationHostGroupPlacementConstraintDetails>) -> Self {
        self.placement_constraint_details = placement_constraint_details;
        self
    }

    /// Set compute_cluster_id
    pub fn set_compute_cluster_id(mut self, compute_cluster_id: Option<String>) -> Self {
        self.compute_cluster_id = compute_cluster_id;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, compartment_id: Option<String>) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, cluster_placement_group_id: Option<String>) -> Self {
        self.cluster_placement_group_id = cluster_placement_group_id;
        self
    }

    /// Set create_vnic_details
    pub fn set_create_vnic_details(mut self, create_vnic_details: Option<super::InstanceConfigurationCreateVnicDetails>) -> Self {
        self.create_vnic_details = create_vnic_details;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(mut self, defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.defined_tags = defined_tags;
        self
    }

    /// Set security_attributes
    pub fn set_security_attributes(mut self, security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>) -> Self {
        self.security_attributes = security_attributes;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    /// Set extended_metadata
    pub fn set_extended_metadata(mut self, extended_metadata: Option<HashMap<String, serde_json::Value>>) -> Self {
        self.extended_metadata = extended_metadata;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, freeform_tags: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = freeform_tags;
        self
    }

    /// Set ipxe_script
    pub fn set_ipxe_script(mut self, ipxe_script: Option<String>) -> Self {
        self.ipxe_script = ipxe_script;
        self
    }

    /// Set metadata
    pub fn set_metadata(mut self, metadata: Option<HashMap<String, String>>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, shape: Option<String>) -> Self {
        self.shape = shape;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(mut self, shape_config: Option<super::InstanceConfigurationLaunchInstanceShapeConfigDetails>) -> Self {
        self.shape_config = shape_config;
        self
    }

    /// Set platform_config
    pub fn set_platform_config(mut self, platform_config: Option<super::InstanceConfigurationLaunchInstancePlatformConfig>) -> Self {
        self.platform_config = platform_config;
        self
    }

    /// Set source_details
    pub fn set_source_details(mut self, source_details: Option<super::InstanceConfigurationInstanceSourceDetails>) -> Self {
        self.source_details = source_details;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, fault_domain: Option<String>) -> Self {
        self.fault_domain = fault_domain;
        self
    }

    /// Set dedicated_vm_host_id
    pub fn set_dedicated_vm_host_id(mut self, dedicated_vm_host_id: Option<String>) -> Self {
        self.dedicated_vm_host_id = dedicated_vm_host_id;
        self
    }

    /// Set launch_mode
    pub fn set_launch_mode(mut self, launch_mode: Option<LaunchMode>) -> Self {
        self.launch_mode = launch_mode;
        self
    }

    /// Set launch_options
    pub fn set_launch_options(mut self, launch_options: Option<super::InstanceConfigurationLaunchOptions>) -> Self {
        self.launch_options = launch_options;
        self
    }

    /// Set agent_config
    pub fn set_agent_config(mut self, agent_config: Option<super::InstanceConfigurationLaunchInstanceAgentConfigDetails>) -> Self {
        self.agent_config = agent_config;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, is_pv_encryption_in_transit_enabled: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = is_pv_encryption_in_transit_enabled;
        self
    }

    /// Set preferred_maintenance_action
    pub fn set_preferred_maintenance_action(mut self, preferred_maintenance_action: Option<PreferredMaintenanceAction>) -> Self {
        self.preferred_maintenance_action = preferred_maintenance_action;
        self
    }

    /// Set instance_options
    pub fn set_instance_options(mut self, instance_options: Option<super::InstanceConfigurationInstanceOptions>) -> Self {
        self.instance_options = instance_options;
        self
    }

    /// Set availability_config
    pub fn set_availability_config(mut self, availability_config: Option<super::InstanceConfigurationAvailabilityConfig>) -> Self {
        self.availability_config = availability_config;
        self
    }

    /// Set preemptible_instance_config
    pub fn set_preemptible_instance_config(mut self, preemptible_instance_config: Option<super::PreemptibleInstanceConfigDetails>) -> Self {
        self.preemptible_instance_config = preemptible_instance_config;
        self
    }

    /// Set licensing_configs
    pub fn set_licensing_configs(mut self, licensing_configs: Option<Vec<super::LaunchInstanceLicensingConfig>>) -> Self {
        self.licensing_configs = licensing_configs;
        self
    }
}

impl Default for InstanceConfigurationLaunchInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
