use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Defines the secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)s held in Vault that represent the MACsec key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMacsecKey {
    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity association Key Name (CKN) of this MACsec key. <p> NOTE: Only the latest secret version will be used.
    pub connectivity_association_name_secret_id: String,

    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key (CAK) of this MACsec key. <p> NOTE: Only the latest secret version will be used.
    pub connectivity_association_key_secret_id: String,
}

/// Required fields for CreateMacsecKey
pub struct CreateMacsecKeyRequired {
    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity association Key Name (CKN) of this MACsec key. <p> NOTE: Only the latest secret version will be used.
    pub connectivity_association_name_secret_id: String,

    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key (CAK) of this MACsec key. <p> NOTE: Only the latest secret version will be used.
    pub connectivity_association_key_secret_id: String,
}

impl CreateMacsecKey {
    /// Create a new CreateMacsecKey with required fields
    pub fn new(required: CreateMacsecKeyRequired) -> Self {
        Self {
            connectivity_association_name_secret_id: required
                .connectivity_association_name_secret_id,

            connectivity_association_key_secret_id: required.connectivity_association_key_secret_id,
        }
    }

    /// Set connectivity_association_name_secret_id
    pub fn set_connectivity_association_name_secret_id(mut self, value: String) -> Self {
        self.connectivity_association_name_secret_id = value;
        self
    }

    /// Set connectivity_association_key_secret_id
    pub fn set_connectivity_association_key_secret_id(mut self, value: String) -> Self {
        self.connectivity_association_key_secret_id = value;
        self
    }
}
