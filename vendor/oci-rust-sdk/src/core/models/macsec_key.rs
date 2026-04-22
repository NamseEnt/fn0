use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// An object defining the Secrets-in-Vault OCIDs representing the MACsec key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MacsecKey {
    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity association Key Name (CKN) of this MACsec key.
    pub connectivity_association_name_secret_id: String,

    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key (CAK) of this MACsec key.
    pub connectivity_association_key_secret_id: String,

    /// The secret version of the connectivity association name secret in Vault. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_association_name_secret_version: Option<i64>,

    /// The secret version of the {@code connectivityAssociationKey} secret in Vault. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_association_key_secret_version: Option<i64>,
}

/// Required fields for MacsecKey
pub struct MacsecKeyRequired {
    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity association Key Name (CKN) of this MACsec key.
    pub connectivity_association_name_secret_id: String,

    /// Secret [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) containing the Connectivity Association Key (CAK) of this MACsec key.
    pub connectivity_association_key_secret_id: String,
}

impl MacsecKey {
    /// Create a new MacsecKey with required fields
    pub fn new(required: MacsecKeyRequired) -> Self {
        Self {
            connectivity_association_name_secret_id: required
                .connectivity_association_name_secret_id,

            connectivity_association_key_secret_id: required.connectivity_association_key_secret_id,

            connectivity_association_name_secret_version: None,

            connectivity_association_key_secret_version: None,
        }
    }

    /// Set connectivity_association_name_secret_id
    pub fn set_connectivity_association_name_secret_id(mut self, value: String) -> Self {
        self.connectivity_association_name_secret_id = value;
        self
    }

    /// Set connectivity_association_name_secret_version
    pub fn set_connectivity_association_name_secret_version(mut self, value: Option<i64>) -> Self {
        self.connectivity_association_name_secret_version = value;
        self
    }

    /// Set connectivity_association_key_secret_id
    pub fn set_connectivity_association_key_secret_id(mut self, value: String) -> Self {
        self.connectivity_association_key_secret_id = value;
        self
    }

    /// Set connectivity_association_key_secret_version
    pub fn set_connectivity_association_key_secret_version(mut self, value: Option<i64>) -> Self {
        self.connectivity_association_key_secret_version = value;
        self
    }

    /// Set connectivity_association_name_secret_version (unwraps Option)
    pub fn with_connectivity_association_name_secret_version(mut self, value: i64) -> Self {
        self.connectivity_association_name_secret_version = Some(value);
        self
    }

    /// Set connectivity_association_key_secret_version (unwraps Option)
    pub fn with_connectivity_association_key_secret_version(mut self, value: i64) -> Self {
        self.connectivity_association_key_secret_version = Some(value);
        self
    }
}
