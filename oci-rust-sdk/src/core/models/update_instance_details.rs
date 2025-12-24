use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstanceDetails {
    /// Whether to enable AI enterprise on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_a_i_enterprise_enabled: Option<bool>,

    /// The OCID of the compute capacity reservation this instance is launched under. You can remove the instance from a reservation by specifying an empty string as input for this field. For more information, see [Capacity Reservations](https://docs.oracle.com/iaas/Content/Compute/Tasks/reserve-capacity.htm#default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<UpdateInstanceAgentConfigDetails>,

    /// Custom metadata key/value string pairs that you provide. Any set of key/value pairs provided here will completely replace the current set of key/value pairs in the {@code metadata} field on the instance. <p> The \"user_data\" field and the \"ssh_authorized_keys\" field cannot be changed after an instance has launched. Any request that updates, removes, or adds either of these fields will be rejected. You must provide the same values for \"user_data\" and \"ssh_authorized_keys\" that already exist on the instance. <p> The combined size of the {@code metadata} and {@code extendedMetadata} objects can be a maximum of 32,000 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// Additional metadata key/value pairs that you provide. They serve the same purpose and functionality as fields in the {@code metadata} object. <p> They are distinguished from {@code metadata} fields in that these can be nested JSON objects (whereas {@code metadata} fields are string/string maps only). <p> The \"user_data\" field and the \"ssh_authorized_keys\" field cannot be changed after an instance has launched. Any request that updates, removes, or adds either of these fields will be rejected. You must provide the same values for \"user_data\" and \"ssh_authorized_keys\" that already exist on the instance. <p> The combined size of the {@code metadata} and {@code extendedMetadata} objects can be a maximum of 32,000 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_metadata: Option<HashMap<String, serde_json::Value>>,

    /// The shape of the instance. The shape determines the number of CPUs and the amount of memory allocated to the instance. For more information about how to change shapes, and a list of shapes that are supported, see [Editing an Instance](https://docs.oracle.com/iaas/Content/Compute/Tasks/resizinginstances.htm). <p> For details about the CPUs, memory, and other properties of each shape, see [Compute Shapes](https://docs.oracle.com/iaas/Content/Compute/References/computeshapes.htm). <p> The new shape must be compatible with the image that was used to launch the instance. You can enumerate all available shapes and determine image compatibility by calling {@link #listShapes(ListShapesRequest) listShapes}. <p> To determine whether capacity is available for a specific shape before you change the shape of an instance, use the {@link #createComputeCapacityReport(CreateComputeCapacityReportRequest) createComputeCapacityReport} operation. <p> If the instance is running when you change the shape, the instance is rebooted. <p> Example: {@code VM.Standard2.1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_config: Option<UpdateInstanceShapeConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<UpdateInstanceSourceViaBootVolumeDetails>,

    /// The parameter acts as a fail-safe to prevent unwanted downtime when updating a running instance. The default is ALLOW_DOWNTIME. * {@code ALLOW_DOWNTIME} - Compute might reboot the instance while updating the instance if a reboot is required. * {@code AVOID_DOWNTIME} - If the instance is in running state, Compute tries to update the instance without rebooting it. If the instance requires a reboot to be updated, an error is returned and the instance is not updated. If the instance is stopped, it is updated and remains in the stopped state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_operation_constraint: Option<UpdateInstanceDetailsUpdateOperationConstraint>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_options: Option<InstanceOptions>,

    /// A fault domain is a grouping of hardware and infrastructure within an availability domain. Each availability domain contains three fault domains. Fault domains let you distribute your instances so that they are not on the same physical hardware within a single availability domain. A hardware failure or Compute hardware maintenance that affects one fault domain does not affect instances in other fault domains. <p> To get a list of fault domains, use the {@link #listFaultDomains(ListFaultDomainsRequest) listFaultDomains} operation in the Identity and Access Management Service API. <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_options: Option<UpdateLaunchOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_config: Option<UpdateInstanceAvailabilityConfigDetails>,

    /// For a VM instance, resets the scheduled time that the instance will be reboot migrated for infrastructure maintenance, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). If the instance hasn't been rebooted after this date, Oracle reboots the instance within 24 hours of the time and date that maintenance is due. <p> To get the maximum possible date that a maintenance reboot can be extended, use {@link #getInstanceMaintenanceReboot(GetInstanceMaintenanceRebootRequest) getInstanceMaintenanceReboot}. <p> Regardless of how the instance is stopped, this flag is reset to empty as soon as the instance reaches the Stopped state. <p> To reboot migrate a bare metal instance, use the {@link #instanceAction(InstanceActionRequest) instanceAction} operation. <p> For more information, see [Infrastructure Maintenance](https://docs.oracle.com/iaas/Content/Compute/References/infrastructure-maintenance.htm). <p> Example: {@code 2018-05-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_maintenance_reboot_due: Option<DateTime<Utc>>,

    /// The OCID of the dedicated virtual machine host to place the instance on. Supported only if this VM instance was already placed on a dedicated virtual machine host - that is, you can't move an instance from on-demand capacity to dedicated capacity, nor can you move an instance from dedicated capacity to on-demand capacity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_vm_host_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_config: Option<AmdVmUpdateInstancePlatformConfig>,

    /// The list of liscensing configurations with target update values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licensing_configs: Option<Vec<UpdateInstanceLicensingConfig>>,
}

impl UpdateInstanceDetails {
    /// Create a new UpdateInstanceDetails
    pub fn new() -> Self {
        Self {
            is_a_i_enterprise_enabled: None,

            capacity_reservation_id: None,

            defined_tags: None,

            security_attributes: None,

            display_name: None,

            freeform_tags: None,

            agent_config: None,

            metadata: None,

            extended_metadata: None,

            shape: None,

            shape_config: None,

            source_details: None,

            update_operation_constraint: None,

            instance_options: None,

            fault_domain: None,

            launch_options: None,

            availability_config: None,

            time_maintenance_reboot_due: None,

            dedicated_vm_host_id: None,

            platform_config: None,

            licensing_configs: None,
        }
    }

    /// Set is_a_i_enterprise_enabled
    pub fn set_is_a_i_enterprise_enabled(mut self, value: Option<bool>) -> Self {
        self.is_a_i_enterprise_enabled = value;
        self
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: Option<String>) -> Self {
        self.capacity_reservation_id = value;
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

    /// Set security_attributes
    pub fn set_security_attributes(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.security_attributes = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set agent_config
    pub fn set_agent_config(mut self, value: Option<UpdateInstanceAgentConfigDetails>) -> Self {
        self.agent_config = value;
        self
    }

    /// Set metadata
    pub fn set_metadata(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.metadata = value;
        self
    }

    /// Set extended_metadata
    pub fn set_extended_metadata(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.extended_metadata = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: Option<String>) -> Self {
        self.shape = value;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(mut self, value: Option<UpdateInstanceShapeConfigDetails>) -> Self {
        self.shape_config = value;
        self
    }

    /// Set source_details
    pub fn set_source_details(
        mut self,
        value: Option<UpdateInstanceSourceViaBootVolumeDetails>,
    ) -> Self {
        self.source_details = value;
        self
    }

    /// Set update_operation_constraint
    pub fn set_update_operation_constraint(
        mut self,
        value: Option<UpdateInstanceDetailsUpdateOperationConstraint>,
    ) -> Self {
        self.update_operation_constraint = value;
        self
    }

    /// Set instance_options
    pub fn set_instance_options(mut self, value: Option<InstanceOptions>) -> Self {
        self.instance_options = value;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set launch_options
    pub fn set_launch_options(mut self, value: Option<UpdateLaunchOptions>) -> Self {
        self.launch_options = value;
        self
    }

    /// Set availability_config
    pub fn set_availability_config(
        mut self,
        value: Option<UpdateInstanceAvailabilityConfigDetails>,
    ) -> Self {
        self.availability_config = value;
        self
    }

    /// Set time_maintenance_reboot_due
    pub fn set_time_maintenance_reboot_due(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_maintenance_reboot_due = value;
        self
    }

    /// Set dedicated_vm_host_id
    pub fn set_dedicated_vm_host_id(mut self, value: Option<String>) -> Self {
        self.dedicated_vm_host_id = value;
        self
    }

    /// Set platform_config
    pub fn set_platform_config(mut self, value: Option<AmdVmUpdateInstancePlatformConfig>) -> Self {
        self.platform_config = value;
        self
    }

    /// Set licensing_configs
    pub fn set_licensing_configs(
        mut self,
        value: Option<Vec<UpdateInstanceLicensingConfig>>,
    ) -> Self {
        self.licensing_configs = value;
        self
    }

    /// Set is_a_i_enterprise_enabled (unwraps Option)
    pub fn with_is_a_i_enterprise_enabled(mut self, value: bool) -> Self {
        self.is_a_i_enterprise_enabled = Some(value);
        self
    }

    /// Set capacity_reservation_id (unwraps Option)
    pub fn with_capacity_reservation_id(mut self, value: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(value.into());
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

    /// Set security_attributes (unwraps Option)
    pub fn with_security_attributes(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.security_attributes = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set agent_config (unwraps Option)
    pub fn with_agent_config(mut self, value: UpdateInstanceAgentConfigDetails) -> Self {
        self.agent_config = Some(value);
        self
    }

    /// Set metadata (unwraps Option)
    pub fn with_metadata(mut self, value: HashMap<String, String>) -> Self {
        self.metadata = Some(value);
        self
    }

    /// Set extended_metadata (unwraps Option)
    pub fn with_extended_metadata(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.extended_metadata = Some(value);
        self
    }

    /// Set shape (unwraps Option)
    pub fn with_shape(mut self, value: impl Into<String>) -> Self {
        self.shape = Some(value.into());
        self
    }

    /// Set shape_config (unwraps Option)
    pub fn with_shape_config(mut self, value: UpdateInstanceShapeConfigDetails) -> Self {
        self.shape_config = Some(value);
        self
    }

    /// Set source_details (unwraps Option)
    pub fn with_source_details(mut self, value: UpdateInstanceSourceViaBootVolumeDetails) -> Self {
        self.source_details = Some(value);
        self
    }

    /// Set update_operation_constraint (unwraps Option)
    pub fn with_update_operation_constraint(
        mut self,
        value: UpdateInstanceDetailsUpdateOperationConstraint,
    ) -> Self {
        self.update_operation_constraint = Some(value);
        self
    }

    /// Set instance_options (unwraps Option)
    pub fn with_instance_options(mut self, value: InstanceOptions) -> Self {
        self.instance_options = Some(value);
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set launch_options (unwraps Option)
    pub fn with_launch_options(mut self, value: UpdateLaunchOptions) -> Self {
        self.launch_options = Some(value);
        self
    }

    /// Set availability_config (unwraps Option)
    pub fn with_availability_config(
        mut self,
        value: UpdateInstanceAvailabilityConfigDetails,
    ) -> Self {
        self.availability_config = Some(value);
        self
    }

    /// Set time_maintenance_reboot_due (unwraps Option)
    pub fn with_time_maintenance_reboot_due(mut self, value: DateTime<Utc>) -> Self {
        self.time_maintenance_reboot_due = Some(value);
        self
    }

    /// Set dedicated_vm_host_id (unwraps Option)
    pub fn with_dedicated_vm_host_id(mut self, value: impl Into<String>) -> Self {
        self.dedicated_vm_host_id = Some(value.into());
        self
    }

    /// Set platform_config (unwraps Option)
    pub fn with_platform_config(mut self, value: AmdVmUpdateInstancePlatformConfig) -> Self {
        self.platform_config = Some(value);
        self
    }

    /// Set licensing_configs (unwraps Option)
    pub fn with_licensing_configs(mut self, value: Vec<UpdateInstanceLicensingConfig>) -> Self {
        self.licensing_configs = Some(value);
        self
    }
}

impl Default for UpdateInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
