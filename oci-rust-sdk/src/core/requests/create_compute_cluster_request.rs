use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateComputeClusterRequest {
    /// The data for creating a [compute cluster](https://docs.oracle.com/iaas/Content/Compute/Tasks/compute-clusters.htm). A compute cluster is an empty remote direct memory access (RDMA) network group. <p> After the compute cluster is created, you can use the compute cluster's OCID with the {@link #launchInstance(LaunchInstanceRequest) launchInstance} operation to create instances in the compute cluster. The instances must be created in the same compartment and availability domain as the cluster. <p> Use compute clusters when you want to manage instances in the cluster individually in the RDMA network group. <p> For details about creating a cluster network that uses instance pools to manage groups of identical instances, see {@link #createClusterNetworkDetails(CreateClusterNetworkDetailsRequest) createClusterNetworkDetails}.
    pub create_compute_cluster_details: CreateComputeClusterDetails,

    /// A token that uniquely identifies a request so it can be retried in case of a timeout or server error without risk of executing that same action again. Retry tokens expire after 24 hours, but can be invalidated before then due to conflicting operations (for example, if a resource has been deleted and purged from the system, then a retry of the original creation request may be rejected).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_retry_token: Option<String>,

    /// Unique identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opc_request_id: Option<String>,
}


/// Required fields for CreateComputeClusterRequest
pub struct CreateComputeClusterRequestRequired {
    /// The data for creating a [compute cluster](https://docs.oracle.com/iaas/Content/Compute/Tasks/compute-clusters.htm). A compute cluster is an empty remote direct memory access (RDMA) network group. <p> After the compute cluster is created, you can use the compute cluster's OCID with the {@link #launchInstance(LaunchInstanceRequest) launchInstance} operation to create instances in the compute cluster. The instances must be created in the same compartment and availability domain as the cluster. <p> Use compute clusters when you want to manage instances in the cluster individually in the RDMA network group. <p> For details about creating a cluster network that uses instance pools to manage groups of identical instances, see {@link #createClusterNetworkDetails(CreateClusterNetworkDetailsRequest) createClusterNetworkDetails}.
    pub create_compute_cluster_details: CreateComputeClusterDetails,
}

impl CreateComputeClusterRequest {
    /// Create a new CreateComputeClusterRequest with required fields
    pub fn new(required: CreateComputeClusterRequestRequired) -> Self {
        Self {
            create_compute_cluster_details: required.create_compute_cluster_details,

            opc_retry_token: None,

            opc_request_id: None,
}
    }

    /// Set create_compute_cluster_details
    pub fn set_create_compute_cluster_details(mut self, value: CreateComputeClusterDetails) -> Self {
        self.create_compute_cluster_details = value;
        self
    }

    /// Set opc_retry_token
    pub fn set_opc_retry_token(mut self, value: Option<String>) -> Self {
        self.opc_retry_token = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: Option<String>) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set opc_retry_token (unwraps Option)
    pub fn with_opc_retry_token(mut self, value: impl Into<String>) -> Self {
        self.opc_retry_token = Some(value.into());
        self
    }

    /// Set opc_request_id (unwraps Option)
    pub fn with_opc_request_id(mut self, value: impl Into<String>) -> Self {
        self.opc_request_id = Some(value.into());
        self
    }
}


