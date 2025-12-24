use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A compute host. The image used to launch the instance determines its operating system and other software. The shape specified during the launch process determines the number of CPUs and memory allocated to the instance. <p> When you launch an instance, it is automatically attached to a virtual network interface card (VNIC), called the *primary VNIC*. The VNIC has a private IP address from the subnet's CIDR. You can either assign a private IP address of your choice or let Oracle automatically assign one. You can choose whether the instance has a public IP address. To retrieve the addresses, use the {@link #listVnicAttachments(ListVnicAttachmentsRequest) listVnicAttachments} operation to get the VNIC ID for the instance, and then call {@link #getVnic(GetVnicRequest) getVnic} with the VNIC ID. <p> For more information, see [Overview of the Compute Service](https://docs.oracle.com/iaas/Content/Compute/Concepts/computeoverview.htm). <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm). <p> *Warning:** Oracle recommends that you avoid using any confidential information when you supply string values using the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    /// The availability domain the instance is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// The OCID of the instance.
    pub id: String,

    /// The current state of the instance.
    pub lifecycle_state: InstanceLifecycleState,

    /// The region that contains the availability domain the instance is running in. <p> For the us-phoenix-1 and us-ashburn-1 regions, {@code phx} and {@code iad} are returned, respectively. For all other regions, the full region name is returned. <p> Examples: {@code phx}, {@code eu-frankfurt-1}
    pub region: String,

    /// The shape of the instance. The shape determines the number of CPUs and the amount of memory allocated to the instance. You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    pub shape: String,

    /// The date and time the instance was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The OCID of the compute capacity reservation this instance is launched under. When this field contains an empty string or is null, the instance is not currently in a capacity reservation. For more information, see [Capacity Reservations](https://docs.oracle.com/iaas/Content/Compute/Tasks/reserve-capacity.htm#default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_constraint_details: Option<HostGroupPlacementConstraintDetails>,

    /// Whether AI enterprise is enabled on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_a_i_enterprise_enabled: Option<bool>,

    /// The OCID of the cluster placement group of the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    /// The OCID of the dedicated virtual machine host that the instance is placed on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_vm_host_id: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The lifecycle state of the {@code securityAttributes}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes_state: Option<InstanceSecurityAttributesState>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Additional metadata key/value pairs that you provide. They serve the same purpose and functionality as fields in the {@code metadata} object. <p> They are distinguished from {@code metadata} fields in that these can be nested JSON objects (whereas {@code metadata} fields are string/string maps only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_metadata: Option<HashMap<String, serde_json::Value>>,

    /// The name of the fault domain the instance is running in. <p> A fault domain is a grouping of hardware and infrastructure within an availability domain. Each availability domain contains three fault domains. Fault domains let you distribute your instances so that they are not on the same physical hardware within a single availability domain. A hardware failure or Compute hardware maintenance that affects one fault domain does not affect instances in other fault domains. <p> If you do not specify the fault domain, the system selects one for you. <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// Deprecated. Use {@code sourceDetails} instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_id: Option<String>,

    /// When a bare metal or virtual machine instance boots, the iPXE firmware that runs on the instance is configured to run an iPXE script to continue the boot process. <p> If you want more control over the boot process, you can provide your own custom iPXE script that will run when the instance boots. Be aware that the same iPXE script will run every time an instance boots, not only after the initial LaunchInstance call. <p> The default iPXE script connects to the instance's local boot volume over iSCSI and performs a network boot. If you use a custom iPXE script and want to network-boot from the instance's local boot volume over iSCSI the same way as the default iPXE script, use the following iSCSI IP address: 169.254.0.2, and boot volume IQN: iqn.2015-02.oracle.boot. <p> If your instance boot volume attachment type is paravirtualized, the boot volume is attached to the instance through virtio-scsi and no iPXE script is used. If your instance boot volume attachment type is paravirtualized and you use custom iPXE to network boot into your instance, the primary boot volume is attached as a data volume through virtio-scsi drive. <p> For more information about the Bring Your Own Image feature of Oracle Cloud Infrastructure, see [Bring Your Own Image](https://docs.oracle.com/iaas/Content/Compute/References/bringyourownimage.htm). <p> For more information about iPXE, see http://ipxe.org.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipxe_script: Option<String>,

    /// Specifies the configuration mode for launching virtual machine (VM) instances. The configuration modes are: * {@code NATIVE} - VM instances launch with iSCSI boot and VFIO devices. The default value for platform images. * {@code EMULATED} - VM instances launch with emulated devices, such as the E1000 network driver and emulated SCSI disk controller. * {@code PARAVIRTUALIZED} - VM instances launch with paravirtualized devices using VirtIO drivers. * {@code CUSTOM} - VM instances launch with custom configuration settings specified in the {@code LaunchOptions} parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<InstanceLaunchMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_options: Option<LaunchOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_options: Option<InstanceOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_config: Option<InstanceAvailabilityConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preemptible_instance_config: Option<PreemptibleInstanceConfigDetails>,

    /// Custom metadata that you provide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_config: Option<InstanceShapeConfig>,

    /// Whether the instance\u2019s OCPUs and memory are distributed across multiple NUMA nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cross_numa_node: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<InstanceSourceViaImageDetails>,

    /// System tags for this resource. Each key is predefined and scoped to a namespace. Example: {@code {\"foo-namespace\": {\"bar-key\": \"value\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<InstanceAgentConfig>,

    /// The date and time the instance is expected to be stopped / started,  in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). After that time if instance hasn't been rebooted, Oracle will reboot the instance within 24 hours of the due time. Regardless of how the instance was stopped, the flag will be reset to empty as soon as instance reaches Stopped state. Example: {@code 2018-05-25T21:10:29.600Z}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_maintenance_reboot_due: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_config: Option<AmdMilanBmPlatformConfig>,

    /// The OCID of the Instance Configuration used to source launch details for this instance. Any other fields supplied in the instance launch request override the details stored in the Instance Configuration for this instance launch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_configuration_id: Option<String>,

    /// List of licensing configurations associated with the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licensing_configs: Option<Vec<LicensingConfig>>,
}

/// Required fields for Instance
pub struct InstanceRequired {
    /// The availability domain the instance is running in. <p> Example: {@code Uocm:PHX-AD-1}
    pub availability_domain: String,

    /// The OCID of the compartment that contains the instance.
    pub compartment_id: String,

    /// The OCID of the instance.
    pub id: String,

    /// The current state of the instance.
    pub lifecycle_state: InstanceLifecycleState,

    /// The region that contains the availability domain the instance is running in. <p> For the us-phoenix-1 and us-ashburn-1 regions, {@code phx} and {@code iad} are returned, respectively. For all other regions, the full region name is returned. <p> Examples: {@code phx}, {@code eu-frankfurt-1}
    pub region: String,

    /// The shape of the instance. The shape determines the number of CPUs and the amount of memory allocated to the instance. You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    pub shape: String,

    /// The date and time the instance was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,
}

impl Instance {
    /// Create a new Instance with required fields
    pub fn new(required: InstanceRequired) -> Self {
        Self {
            availability_domain: required.availability_domain,

            compartment_id: required.compartment_id,

            id: required.id,

            lifecycle_state: required.lifecycle_state,

            region: required.region,

            shape: required.shape,

            time_created: required.time_created,

            capacity_reservation_id: None,

            placement_constraint_details: None,

            is_a_i_enterprise_enabled: None,

            cluster_placement_group_id: None,

            dedicated_vm_host_id: None,

            defined_tags: None,

            security_attributes: None,

            security_attributes_state: None,

            display_name: None,

            extended_metadata: None,

            fault_domain: None,

            freeform_tags: None,

            image_id: None,

            ipxe_script: None,

            launch_mode: None,

            launch_options: None,

            instance_options: None,

            availability_config: None,

            preemptible_instance_config: None,

            metadata: None,

            shape_config: None,

            is_cross_numa_node: None,

            source_details: None,

            system_tags: None,

            agent_config: None,

            time_maintenance_reboot_due: None,

            platform_config: None,

            instance_configuration_id: None,

            licensing_configs: None,
        }
    }

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: String) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: Option<String>) -> Self {
        self.capacity_reservation_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set placement_constraint_details
    pub fn set_placement_constraint_details(
        mut self,
        value: Option<HostGroupPlacementConstraintDetails>,
    ) -> Self {
        self.placement_constraint_details = value;
        self
    }

    /// Set is_a_i_enterprise_enabled
    pub fn set_is_a_i_enterprise_enabled(mut self, value: Option<bool>) -> Self {
        self.is_a_i_enterprise_enabled = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
        self
    }

    /// Set dedicated_vm_host_id
    pub fn set_dedicated_vm_host_id(mut self, value: Option<String>) -> Self {
        self.dedicated_vm_host_id = value;
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

    /// Set security_attributes_state
    pub fn set_security_attributes_state(
        mut self,
        value: Option<InstanceSecurityAttributesState>,
    ) -> Self {
        self.security_attributes_state = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
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

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set image_id
    pub fn set_image_id(mut self, value: Option<String>) -> Self {
        self.image_id = value;
        self
    }

    /// Set ipxe_script
    pub fn set_ipxe_script(mut self, value: Option<String>) -> Self {
        self.ipxe_script = value;
        self
    }

    /// Set launch_mode
    pub fn set_launch_mode(mut self, value: Option<InstanceLaunchMode>) -> Self {
        self.launch_mode = value;
        self
    }

    /// Set launch_options
    pub fn set_launch_options(mut self, value: Option<LaunchOptions>) -> Self {
        self.launch_options = value;
        self
    }

    /// Set instance_options
    pub fn set_instance_options(mut self, value: Option<InstanceOptions>) -> Self {
        self.instance_options = value;
        self
    }

    /// Set availability_config
    pub fn set_availability_config(mut self, value: Option<InstanceAvailabilityConfig>) -> Self {
        self.availability_config = value;
        self
    }

    /// Set preemptible_instance_config
    pub fn set_preemptible_instance_config(
        mut self,
        value: Option<PreemptibleInstanceConfigDetails>,
    ) -> Self {
        self.preemptible_instance_config = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: InstanceLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set metadata
    pub fn set_metadata(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.metadata = value;
        self
    }

    /// Set region
    pub fn set_region(mut self, value: String) -> Self {
        self.region = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: String) -> Self {
        self.shape = value;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(mut self, value: Option<InstanceShapeConfig>) -> Self {
        self.shape_config = value;
        self
    }

    /// Set is_cross_numa_node
    pub fn set_is_cross_numa_node(mut self, value: Option<bool>) -> Self {
        self.is_cross_numa_node = value;
        self
    }

    /// Set source_details
    pub fn set_source_details(mut self, value: Option<InstanceSourceViaImageDetails>) -> Self {
        self.source_details = value;
        self
    }

    /// Set system_tags
    pub fn set_system_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.system_tags = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set agent_config
    pub fn set_agent_config(mut self, value: Option<InstanceAgentConfig>) -> Self {
        self.agent_config = value;
        self
    }

    /// Set time_maintenance_reboot_due
    pub fn set_time_maintenance_reboot_due(mut self, value: Option<DateTime<Utc>>) -> Self {
        self.time_maintenance_reboot_due = value;
        self
    }

    /// Set platform_config
    pub fn set_platform_config(mut self, value: Option<AmdMilanBmPlatformConfig>) -> Self {
        self.platform_config = value;
        self
    }

    /// Set instance_configuration_id
    pub fn set_instance_configuration_id(mut self, value: Option<String>) -> Self {
        self.instance_configuration_id = value;
        self
    }

    /// Set licensing_configs
    pub fn set_licensing_configs(mut self, value: Option<Vec<LicensingConfig>>) -> Self {
        self.licensing_configs = value;
        self
    }

    /// Set capacity_reservation_id (unwraps Option)
    pub fn with_capacity_reservation_id(mut self, value: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(value.into());
        self
    }

    /// Set placement_constraint_details (unwraps Option)
    pub fn with_placement_constraint_details(
        mut self,
        value: HostGroupPlacementConstraintDetails,
    ) -> Self {
        self.placement_constraint_details = Some(value);
        self
    }

    /// Set is_a_i_enterprise_enabled (unwraps Option)
    pub fn with_is_a_i_enterprise_enabled(mut self, value: bool) -> Self {
        self.is_a_i_enterprise_enabled = Some(value);
        self
    }

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }

    /// Set dedicated_vm_host_id (unwraps Option)
    pub fn with_dedicated_vm_host_id(mut self, value: impl Into<String>) -> Self {
        self.dedicated_vm_host_id = Some(value.into());
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

    /// Set security_attributes_state (unwraps Option)
    pub fn with_security_attributes_state(
        mut self,
        value: InstanceSecurityAttributesState,
    ) -> Self {
        self.security_attributes_state = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set extended_metadata (unwraps Option)
    pub fn with_extended_metadata(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.extended_metadata = Some(value);
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set image_id (unwraps Option)
    pub fn with_image_id(mut self, value: impl Into<String>) -> Self {
        self.image_id = Some(value.into());
        self
    }

    /// Set ipxe_script (unwraps Option)
    pub fn with_ipxe_script(mut self, value: impl Into<String>) -> Self {
        self.ipxe_script = Some(value.into());
        self
    }

    /// Set launch_mode (unwraps Option)
    pub fn with_launch_mode(mut self, value: InstanceLaunchMode) -> Self {
        self.launch_mode = Some(value);
        self
    }

    /// Set launch_options (unwraps Option)
    pub fn with_launch_options(mut self, value: LaunchOptions) -> Self {
        self.launch_options = Some(value);
        self
    }

    /// Set instance_options (unwraps Option)
    pub fn with_instance_options(mut self, value: InstanceOptions) -> Self {
        self.instance_options = Some(value);
        self
    }

    /// Set availability_config (unwraps Option)
    pub fn with_availability_config(mut self, value: InstanceAvailabilityConfig) -> Self {
        self.availability_config = Some(value);
        self
    }

    /// Set preemptible_instance_config (unwraps Option)
    pub fn with_preemptible_instance_config(
        mut self,
        value: PreemptibleInstanceConfigDetails,
    ) -> Self {
        self.preemptible_instance_config = Some(value);
        self
    }

    /// Set metadata (unwraps Option)
    pub fn with_metadata(mut self, value: HashMap<String, String>) -> Self {
        self.metadata = Some(value);
        self
    }

    /// Set shape_config (unwraps Option)
    pub fn with_shape_config(mut self, value: InstanceShapeConfig) -> Self {
        self.shape_config = Some(value);
        self
    }

    /// Set is_cross_numa_node (unwraps Option)
    pub fn with_is_cross_numa_node(mut self, value: bool) -> Self {
        self.is_cross_numa_node = Some(value);
        self
    }

    /// Set source_details (unwraps Option)
    pub fn with_source_details(mut self, value: InstanceSourceViaImageDetails) -> Self {
        self.source_details = Some(value);
        self
    }

    /// Set system_tags (unwraps Option)
    pub fn with_system_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.system_tags = Some(value);
        self
    }

    /// Set agent_config (unwraps Option)
    pub fn with_agent_config(mut self, value: InstanceAgentConfig) -> Self {
        self.agent_config = Some(value);
        self
    }

    /// Set time_maintenance_reboot_due (unwraps Option)
    pub fn with_time_maintenance_reboot_due(mut self, value: DateTime<Utc>) -> Self {
        self.time_maintenance_reboot_due = Some(value);
        self
    }

    /// Set platform_config (unwraps Option)
    pub fn with_platform_config(mut self, value: AmdMilanBmPlatformConfig) -> Self {
        self.platform_config = Some(value);
        self
    }

    /// Set instance_configuration_id (unwraps Option)
    pub fn with_instance_configuration_id(mut self, value: impl Into<String>) -> Self {
        self.instance_configuration_id = Some(value.into());
        self
    }

    /// Set licensing_configs (unwraps Option)
    pub fn with_licensing_configs(mut self, value: Vec<LicensingConfig>) -> Self {
        self.licensing_configs = Some(value);
        self
    }
}
