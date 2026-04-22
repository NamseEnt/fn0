use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The information about the VCN and the DNS resolver in the association.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VcnDnsResolverAssociation {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN in the association.
    pub vcn_id: String,

    /// The current state of the association.
    pub lifecycle_state: VcnDnsResolverAssociationLifecycleState,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the DNS resolver in the association.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_resolver_id: Option<String>,
}

/// Required fields for VcnDnsResolverAssociation
pub struct VcnDnsResolverAssociationRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN in the association.
    pub vcn_id: String,

    /// The current state of the association.
    pub lifecycle_state: VcnDnsResolverAssociationLifecycleState,
}

impl VcnDnsResolverAssociation {
    /// Create a new VcnDnsResolverAssociation with required fields
    pub fn new(required: VcnDnsResolverAssociationRequired) -> Self {
        Self {
            vcn_id: required.vcn_id,

            lifecycle_state: required.lifecycle_state,

            dns_resolver_id: None,
        }
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set dns_resolver_id
    pub fn set_dns_resolver_id(mut self, value: Option<String>) -> Self {
        self.dns_resolver_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: VcnDnsResolverAssociationLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set dns_resolver_id (unwraps Option)
    pub fn with_dns_resolver_id(mut self, value: impl Into<String>) -> Self {
        self.dns_resolver_id = Some(value.into());
        self
    }
}
