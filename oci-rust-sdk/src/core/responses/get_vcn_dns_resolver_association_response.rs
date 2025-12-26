use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
#[allow(unused_imports)]
use super::super::models::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVcnDnsResolverAssociationResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VcnDnsResolverAssociation instance.
    pub vcn_dns_resolver_association: VcnDnsResolverAssociation,
}


/// Required fields for GetVcnDnsResolverAssociationResponse
pub struct GetVcnDnsResolverAssociationResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.VcnDnsResolverAssociation instance.
    pub vcn_dns_resolver_association: VcnDnsResolverAssociation,
}

impl GetVcnDnsResolverAssociationResponse {
    /// Create a new GetVcnDnsResolverAssociationResponse with required fields
    pub fn new(required: GetVcnDnsResolverAssociationResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            vcn_dns_resolver_association: required.vcn_dns_resolver_association,
}
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set vcn_dns_resolver_association
    pub fn set_vcn_dns_resolver_association(mut self, value: VcnDnsResolverAssociation) -> Self {
        self.vcn_dns_resolver_association = value;
        self
    }
}


