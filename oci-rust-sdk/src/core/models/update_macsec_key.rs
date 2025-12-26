use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object defining the OCID of the Secret held in Vault that represent the MACsec key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMacsecKey {
    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key Name (CKN) of this MACsec key.
    pub connectivity_association_name_secret_id: String,

    /// The secret version of the connectivity association name secret in Vault. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub connectivity_association_name_secret_version: i64,

    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key (CAK) of this MACsec key.
    pub connectivity_association_key_secret_id: String,

    /// The secret version of the connectivityAssociationKey secret in Vault. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub connectivity_association_key_secret_version: i64,
}

/// Required fields for UpdateMacsecKey
pub struct UpdateMacsecKeyRequired {
    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key Name (CKN) of this MACsec key.
    pub connectivity_association_name_secret_id: String,

    /// The secret version of the connectivity association name secret in Vault. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub connectivity_association_name_secret_version: i64,

    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key (CAK) of this MACsec key.
    pub connectivity_association_key_secret_id: String,

    /// The secret version of the connectivityAssociationKey secret in Vault. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    pub connectivity_association_key_secret_version: i64,
}

impl UpdateMacsecKey {
    /// Create a new UpdateMacsecKey with required fields
    pub fn new(required: UpdateMacsecKeyRequired) -> Self {
        Self {
            connectivity_association_name_secret_id: required
                .connectivity_association_name_secret_id,

            connectivity_association_name_secret_version: required
                .connectivity_association_name_secret_version,

            connectivity_association_key_secret_id: required.connectivity_association_key_secret_id,

            connectivity_association_key_secret_version: required
                .connectivity_association_key_secret_version,
        }
    }

    /// Set connectivity_association_name_secret_id
    pub fn set_connectivity_association_name_secret_id(mut self, value: String) -> Self {
        self.connectivity_association_name_secret_id = value;
        self
    }

    /// Set connectivity_association_name_secret_version
    pub fn set_connectivity_association_name_secret_version(mut self, value: i64) -> Self {
        self.connectivity_association_name_secret_version = value;
        self
    }

    /// Set connectivity_association_key_secret_id
    pub fn set_connectivity_association_key_secret_id(mut self, value: String) -> Self {
        self.connectivity_association_key_secret_id = value;
        self
    }

    /// Set connectivity_association_key_secret_version
    pub fn set_connectivity_association_key_secret_version(mut self, value: i64) -> Self {
        self.connectivity_association_key_secret_version = value;
        self
    }
}
