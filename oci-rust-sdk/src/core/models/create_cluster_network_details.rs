use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The data to create a [cluster network with instance pools](https://docs.oracle.com/iaas/Content/Compute/Tasks/managingclusternetworks.htm). <p> Use cluster networks with instance pools when you want predictable capacity for a specific number of identical instances that are managed as a group. <p> For details about creating compute clusters, which let you manage instances in the RDMA network independently of each other or use different types of instances in the network group, see {@link #createComputeClusterDetails(CreateComputeClusterDetailsRequest) createComputeClusterDetails}.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClusterNetworkDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the cluster network.
    pub compartment_id: String,

    /// The data to create the instance pools in the cluster network. <p> Each cluster network can have one instance pool.
    pub instance_pools: Vec<CreateClusterNetworkInstancePoolDetails>,

    pub placement_configuration: ClusterNetworkPlacementConfigurationDetails,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_configuration: Option<ClusterConfigurationDetails>,
}

/// Required fields for CreateClusterNetworkDetails
pub struct CreateClusterNetworkDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the cluster network.
    pub compartment_id: String,

    /// The data to create the instance pools in the cluster network. <p> Each cluster network can have one instance pool.
    pub instance_pools: Vec<CreateClusterNetworkInstancePoolDetails>,

    pub placement_configuration: ClusterNetworkPlacementConfigurationDetails,
}

impl CreateClusterNetworkDetails {
    /// Create a new CreateClusterNetworkDetails with required fields
    pub fn new(required: CreateClusterNetworkDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            instance_pools: required.instance_pools,

            placement_configuration: required.placement_configuration,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            cluster_configuration: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
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

    /// Set instance_pools
    pub fn set_instance_pools(
        mut self,
        value: Vec<CreateClusterNetworkInstancePoolDetails>,
    ) -> Self {
        self.instance_pools = value;
        self
    }

    /// Set placement_configuration
    pub fn set_placement_configuration(
        mut self,
        value: ClusterNetworkPlacementConfigurationDetails,
    ) -> Self {
        self.placement_configuration = value;
        self
    }

    /// Set cluster_configuration
    pub fn set_cluster_configuration(mut self, value: Option<ClusterConfigurationDetails>) -> Self {
        self.cluster_configuration = value;
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

    /// Set cluster_configuration (unwraps Option)
    pub fn with_cluster_configuration(mut self, value: ClusterConfigurationDetails) -> Self {
        self.cluster_configuration = Some(value);
        self
    }
}
