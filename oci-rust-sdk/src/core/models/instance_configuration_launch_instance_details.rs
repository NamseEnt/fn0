use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// Instance launch details for creating an instance from an instance configuration. Use the {@code sourceDetails} parameter to specify whether a boot volume or an image should be used to launch a new instance. <p> See {@link LaunchInstanceDetails} for more information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConfigurationLaunchInstanceDetails {
    /// The availability domain of the instance. <p> Example: {@code Uocm:PHX-AD-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_domain: Option<String>,

    /// The OCID of the compute capacity reservation this instance is launched under.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity_reservation_id: Option<String>,

    /// Whether to enable AI enterprise on the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_a_i_enterprise_enabled: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_constraint_details:
        Option<InstanceConfigurationHostGroupPlacementConstraintDetails>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the [compute cluster](https://docs.oracle.com/iaas/Content/Compute/Tasks/compute-clusters.htm) that the instance will be created in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_cluster_id: Option<String>,

    /// The OCID of the compartment containing the instance. Instances created from instance configurations are placed in the same compartment as the instance that was used to create the instance configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The OCID of the cluster placement group of the instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_placement_group_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_vnic_details: Option<InstanceConfigurationCreateVnicDetails>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// [Security attributes](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/zpr-artifacts.htm#security-attributes) are labels for a resource that can be referenced in a [Zero Trust Packet Routing](https://docs.oracle.com/iaas/Content/zero-trust-packet-routing/overview.htm) (ZPR) policy to control access to ZPR-supported resources. <p> Example: {@code {\"Oracle-DataSecurity-ZPR\": {\"MaxEgressCount\": {\"value\":\"42\",\"mode\":\"audit\"}}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_attributes: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Additional metadata key/value pairs that you provide. They serve the same purpose and functionality as fields in the {@code metadata} object. <p> They are distinguished from {@code metadata} fields in that these can be nested JSON objects (whereas {@code metadata} fields are string/string maps only). <p> The combined size of the {@code metadata} and {@code extendedMetadata} objects can be a maximum of 32,000 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extended_metadata: Option<HashMap<String, serde_json::Value>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// This is an advanced option. <p> When a bare metal or virtual machine instance boots, the iPXE firmware that runs on the instance is configured to run an iPXE script to continue the boot process. <p> If you want more control over the boot process, you can provide your own custom iPXE script that will run when the instance boots; however, you should be aware that the same iPXE script will run every time an instance boots; not only after the initial LaunchInstance call. <p> The default iPXE script connects to the instance's local boot volume over iSCSI and performs a network boot. If you use a custom iPXE script and want to network-boot from the instance's local boot volume over iSCSI the same way as the default iPXE script, you should use the following iSCSI IP address: 169.254.0.2, and boot volume IQN: iqn.2015-02.oracle.boot. <p> For more information about the Bring Your Own Image feature of Oracle Cloud Infrastructure, see [Bring Your Own Image](https://docs.oracle.com/iaas/Content/Compute/References/bringyourownimage.htm). <p> For more information about iPXE, see http://ipxe.org.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipxe_script: Option<String>,

    /// Custom metadata key/value pairs that you provide, such as the SSH public key required to connect to the instance. <p> A metadata service runs on every launched instance. The service is an HTTP endpoint listening on 169.254.169.254. You can use the service to: <p> Provide information to [Cloud-Init](https://cloudinit.readthedocs.org/en/latest/) to be used for various system initialization tasks. <p> Get information about the instance, including the custom metadata that you provide when you launch the instance. <p> *Providing Cloud-Init Metadata** <p> You can use the following metadata key names to provide information to Cloud-Init: <p> *\"ssh_authorized_keys\"** - Provide one or more public SSH keys to be included in the {@code ~/.ssh/authorized_keys} file for the default user on the instance. Use a newline character to separate multiple keys. The SSH keys must be in the format necessary for the {@code authorized_keys} file, as shown in the example below. <p> *\"user_data\"** - Provide your own base64-encoded data to be used by Cloud-Init to run custom scripts or provide custom Cloud-Init configuration. For information about how to take advantage of user data, see the [Cloud-Init Documentation](http://cloudinit.readthedocs.org/en/latest/topics/format.html). <p> *Metadata Example** <p> \"metadata\" : { \"quake_bot_level\" : \"Severe\", \"ssh_authorized_keys\" : \"ssh-rsa <your_public_SSH_key>== rsa-key-20160227\", \"user_data\" : \"<your_public_SSH_key>==\" } **Getting Metadata on the Instance** <p> To get information about your instance, connect to the instance using SSH and issue any of the following GET requests: <p> curl -H \"Authorization: Bearer Oracle\" http://169.254.169.254/opc/v2/instance/ curl -H \"Authorization: Bearer Oracle\" http://169.254.169.254/opc/v2/instance/metadata/ curl -H \"Authorization: Bearer Oracle\" http://169.254.169.254/opc/v2/instance/metadata/<any-key-name> <p> You'll get back a response that includes all the instance information; only the metadata information; or the metadata information for the specified key name, respectively. <p> The combined size of the {@code metadata} and {@code extendedMetadata} objects can be a maximum of 32,000 bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// The shape of an instance. The shape determines the number of CPUs, amount of memory, and other resources allocated to the instance. <p> You can enumerate all available shapes by calling {@link #listShapes(ListShapesRequest) listShapes}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_config: Option<InstanceConfigurationLaunchInstanceShapeConfigDetails>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_config: Option<InstanceConfigurationAmdMilanBmLaunchInstancePlatformConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_details: Option<InstanceConfigurationInstanceSourceViaImageDetails>,

    /// A fault domain is a grouping of hardware and infrastructure within an availability domain. Each availability domain contains three fault domains. Fault domains let you distribute your instances so that they are not on the same physical hardware within a single availability domain. A hardware failure or Compute hardware maintenance that affects one fault domain does not affect instances in other fault domains. <p> If you do not specify the fault domain, the system selects one for you. <p> To get a list of fault domains, use the {@link #listFaultDomains(ListFaultDomainsRequest) listFaultDomains} operation in the Identity and Access Management Service API. <p> Example: {@code FAULT-DOMAIN-1}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fault_domain: Option<String>,

    /// The OCID of the dedicated virtual machine host to place the instance on. <p> Dedicated VM hosts can be used when launching individual instances from an instance configuration. They cannot be used to launch instance pools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dedicated_vm_host_id: Option<String>,

    /// Specifies the configuration mode for launching virtual machine (VM) instances. The configuration modes are: * {@code NATIVE} - VM instances launch with iSCSI boot and VFIO devices. The default value for platform images. * {@code EMULATED} - VM instances launch with emulated devices, such as the E1000 network driver and emulated SCSI disk controller. * {@code PARAVIRTUALIZED} - VM instances launch with paravirtualized devices using VirtIO drivers. * {@code CUSTOM} - VM instances launch with custom configuration settings specified in the {@code LaunchOptions} parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_mode: Option<InstanceConfigurationLaunchInstanceDetailsLaunchMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_options: Option<InstanceConfigurationLaunchOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_config: Option<InstanceConfigurationLaunchInstanceAgentConfigDetails>,

    /// Whether to enable in-transit encryption for the data volume's paravirtualized attachment. The default value is false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pv_encryption_in_transit_enabled: Option<bool>,

    /// The preferred maintenance action for an instance. The default is LIVE_MIGRATE, if live migration is supported. * {@code LIVE_MIGRATE} - Run maintenance using a live migration. * {@code REBOOT} - Run maintenance using a reboot.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_maintenance_action:
        Option<InstanceConfigurationLaunchInstanceDetailsPreferredMaintenanceAction>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_options: Option<InstanceConfigurationInstanceOptions>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability_config: Option<InstanceConfigurationAvailabilityConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub preemptible_instance_config: Option<PreemptibleInstanceConfigDetails>,

    /// List of licensing configurations associated with target launch values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub licensing_configs: Option<Vec<LaunchInstanceLicensingConfig>>,
}

impl InstanceConfigurationLaunchInstanceDetails {
    /// Create a new InstanceConfigurationLaunchInstanceDetails
    pub fn new() -> Self {
        Self {
            availability_domain: None,

            capacity_reservation_id: None,

            is_a_i_enterprise_enabled: None,

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

    /// Set availability_domain
    pub fn set_availability_domain(mut self, value: Option<String>) -> Self {
        self.availability_domain = value;
        self
    }

    /// Set capacity_reservation_id
    pub fn set_capacity_reservation_id(mut self, value: Option<String>) -> Self {
        self.capacity_reservation_id = value;
        self
    }

    /// Set is_a_i_enterprise_enabled
    pub fn set_is_a_i_enterprise_enabled(mut self, value: Option<bool>) -> Self {
        self.is_a_i_enterprise_enabled = value;
        self
    }

    /// Set placement_constraint_details
    pub fn set_placement_constraint_details(
        mut self,
        value: Option<InstanceConfigurationHostGroupPlacementConstraintDetails>,
    ) -> Self {
        self.placement_constraint_details = value;
        self
    }

    /// Set compute_cluster_id
    pub fn set_compute_cluster_id(mut self, value: Option<String>) -> Self {
        self.compute_cluster_id = value;
        self
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set cluster_placement_group_id
    pub fn set_cluster_placement_group_id(mut self, value: Option<String>) -> Self {
        self.cluster_placement_group_id = value;
        self
    }

    /// Set create_vnic_details
    pub fn set_create_vnic_details(
        mut self,
        value: Option<InstanceConfigurationCreateVnicDetails>,
    ) -> Self {
        self.create_vnic_details = value;
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

    /// Set extended_metadata
    pub fn set_extended_metadata(
        mut self,
        value: Option<HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.extended_metadata = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set ipxe_script
    pub fn set_ipxe_script(mut self, value: Option<String>) -> Self {
        self.ipxe_script = value;
        self
    }

    /// Set metadata
    pub fn set_metadata(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.metadata = value;
        self
    }

    /// Set shape
    pub fn set_shape(mut self, value: Option<String>) -> Self {
        self.shape = value;
        self
    }

    /// Set shape_config
    pub fn set_shape_config(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceShapeConfigDetails>,
    ) -> Self {
        self.shape_config = value;
        self
    }

    /// Set platform_config
    pub fn set_platform_config(
        mut self,
        value: Option<InstanceConfigurationAmdMilanBmLaunchInstancePlatformConfig>,
    ) -> Self {
        self.platform_config = value;
        self
    }

    /// Set source_details
    pub fn set_source_details(
        mut self,
        value: Option<InstanceConfigurationInstanceSourceViaImageDetails>,
    ) -> Self {
        self.source_details = value;
        self
    }

    /// Set fault_domain
    pub fn set_fault_domain(mut self, value: Option<String>) -> Self {
        self.fault_domain = value;
        self
    }

    /// Set dedicated_vm_host_id
    pub fn set_dedicated_vm_host_id(mut self, value: Option<String>) -> Self {
        self.dedicated_vm_host_id = value;
        self
    }

    /// Set launch_mode
    pub fn set_launch_mode(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceDetailsLaunchMode>,
    ) -> Self {
        self.launch_mode = value;
        self
    }

    /// Set launch_options
    pub fn set_launch_options(mut self, value: Option<InstanceConfigurationLaunchOptions>) -> Self {
        self.launch_options = value;
        self
    }

    /// Set agent_config
    pub fn set_agent_config(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceAgentConfigDetails>,
    ) -> Self {
        self.agent_config = value;
        self
    }

    /// Set is_pv_encryption_in_transit_enabled
    pub fn set_is_pv_encryption_in_transit_enabled(mut self, value: Option<bool>) -> Self {
        self.is_pv_encryption_in_transit_enabled = value;
        self
    }

    /// Set preferred_maintenance_action
    pub fn set_preferred_maintenance_action(
        mut self,
        value: Option<InstanceConfigurationLaunchInstanceDetailsPreferredMaintenanceAction>,
    ) -> Self {
        self.preferred_maintenance_action = value;
        self
    }

    /// Set instance_options
    pub fn set_instance_options(
        mut self,
        value: Option<InstanceConfigurationInstanceOptions>,
    ) -> Self {
        self.instance_options = value;
        self
    }

    /// Set availability_config
    pub fn set_availability_config(
        mut self,
        value: Option<InstanceConfigurationAvailabilityConfig>,
    ) -> Self {
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

    /// Set licensing_configs
    pub fn set_licensing_configs(
        mut self,
        value: Option<Vec<LaunchInstanceLicensingConfig>>,
    ) -> Self {
        self.licensing_configs = value;
        self
    }

    /// Set availability_domain (unwraps Option)
    pub fn with_availability_domain(mut self, value: impl Into<String>) -> Self {
        self.availability_domain = Some(value.into());
        self
    }

    /// Set capacity_reservation_id (unwraps Option)
    pub fn with_capacity_reservation_id(mut self, value: impl Into<String>) -> Self {
        self.capacity_reservation_id = Some(value.into());
        self
    }

    /// Set is_a_i_enterprise_enabled (unwraps Option)
    pub fn with_is_a_i_enterprise_enabled(mut self, value: bool) -> Self {
        self.is_a_i_enterprise_enabled = Some(value);
        self
    }

    /// Set placement_constraint_details (unwraps Option)
    pub fn with_placement_constraint_details(
        mut self,
        value: InstanceConfigurationHostGroupPlacementConstraintDetails,
    ) -> Self {
        self.placement_constraint_details = Some(value);
        self
    }

    /// Set compute_cluster_id (unwraps Option)
    pub fn with_compute_cluster_id(mut self, value: impl Into<String>) -> Self {
        self.compute_cluster_id = Some(value.into());
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set cluster_placement_group_id (unwraps Option)
    pub fn with_cluster_placement_group_id(mut self, value: impl Into<String>) -> Self {
        self.cluster_placement_group_id = Some(value.into());
        self
    }

    /// Set create_vnic_details (unwraps Option)
    pub fn with_create_vnic_details(
        mut self,
        value: InstanceConfigurationCreateVnicDetails,
    ) -> Self {
        self.create_vnic_details = Some(value);
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

    /// Set extended_metadata (unwraps Option)
    pub fn with_extended_metadata(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.extended_metadata = Some(value);
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set ipxe_script (unwraps Option)
    pub fn with_ipxe_script(mut self, value: impl Into<String>) -> Self {
        self.ipxe_script = Some(value.into());
        self
    }

    /// Set metadata (unwraps Option)
    pub fn with_metadata(mut self, value: HashMap<String, String>) -> Self {
        self.metadata = Some(value);
        self
    }

    /// Set shape (unwraps Option)
    pub fn with_shape(mut self, value: impl Into<String>) -> Self {
        self.shape = Some(value.into());
        self
    }

    /// Set shape_config (unwraps Option)
    pub fn with_shape_config(
        mut self,
        value: InstanceConfigurationLaunchInstanceShapeConfigDetails,
    ) -> Self {
        self.shape_config = Some(value);
        self
    }

    /// Set platform_config (unwraps Option)
    pub fn with_platform_config(
        mut self,
        value: InstanceConfigurationAmdMilanBmLaunchInstancePlatformConfig,
    ) -> Self {
        self.platform_config = Some(value);
        self
    }

    /// Set source_details (unwraps Option)
    pub fn with_source_details(
        mut self,
        value: InstanceConfigurationInstanceSourceViaImageDetails,
    ) -> Self {
        self.source_details = Some(value);
        self
    }

    /// Set fault_domain (unwraps Option)
    pub fn with_fault_domain(mut self, value: impl Into<String>) -> Self {
        self.fault_domain = Some(value.into());
        self
    }

    /// Set dedicated_vm_host_id (unwraps Option)
    pub fn with_dedicated_vm_host_id(mut self, value: impl Into<String>) -> Self {
        self.dedicated_vm_host_id = Some(value.into());
        self
    }

    /// Set launch_mode (unwraps Option)
    pub fn with_launch_mode(
        mut self,
        value: InstanceConfigurationLaunchInstanceDetailsLaunchMode,
    ) -> Self {
        self.launch_mode = Some(value);
        self
    }

    /// Set launch_options (unwraps Option)
    pub fn with_launch_options(mut self, value: InstanceConfigurationLaunchOptions) -> Self {
        self.launch_options = Some(value);
        self
    }

    /// Set agent_config (unwraps Option)
    pub fn with_agent_config(
        mut self,
        value: InstanceConfigurationLaunchInstanceAgentConfigDetails,
    ) -> Self {
        self.agent_config = Some(value);
        self
    }

    /// Set is_pv_encryption_in_transit_enabled (unwraps Option)
    pub fn with_is_pv_encryption_in_transit_enabled(mut self, value: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(value);
        self
    }

    /// Set preferred_maintenance_action (unwraps Option)
    pub fn with_preferred_maintenance_action(
        mut self,
        value: InstanceConfigurationLaunchInstanceDetailsPreferredMaintenanceAction,
    ) -> Self {
        self.preferred_maintenance_action = Some(value);
        self
    }

    /// Set instance_options (unwraps Option)
    pub fn with_instance_options(mut self, value: InstanceConfigurationInstanceOptions) -> Self {
        self.instance_options = Some(value);
        self
    }

    /// Set availability_config (unwraps Option)
    pub fn with_availability_config(
        mut self,
        value: InstanceConfigurationAvailabilityConfig,
    ) -> Self {
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

    /// Set licensing_configs (unwraps Option)
    pub fn with_licensing_configs(mut self, value: Vec<LaunchInstanceLicensingConfig>) -> Self {
        self.licensing_configs = Some(value);
        self
    }
}

impl Default for InstanceConfigurationLaunchInstanceDetails {
    fn default() -> Self {
        Self::new()
    }
}
