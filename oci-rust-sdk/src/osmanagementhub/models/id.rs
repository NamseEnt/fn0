use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Id and name of a resource to simplify the display for the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Id {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource that is immutable on creation.
    pub id: String,

    /// User-friendly name.
    pub display_name: String,
}

/// Required fields for Id
pub struct IdRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the resource that is immutable on creation.
    pub id: String,

    /// User-friendly name.
    pub display_name: String,
}

impl Id {
    /// Create a new Id with required fields
    pub fn new(required: IdRequired) -> Self {
        Self {
            id: required.id,

            display_name: required.display_name,
        }
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }
}
