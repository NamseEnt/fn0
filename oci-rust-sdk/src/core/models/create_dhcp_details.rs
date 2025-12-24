use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDhcpDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the set of DHCP options.
    pub compartment_id: String,

    /// A set of DHCP options.
    pub options: Vec<DhcpOption>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the set of DHCP options belongs to.
    pub vcn_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The search domain name type of DHCP options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_name_type: Option<CreateDhcpDetailsDomainNameType>,
}

/// Required fields for CreateDhcpDetails
pub struct CreateDhcpDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment to contain the set of DHCP options.
    pub compartment_id: String,

    /// A set of DHCP options.
    pub options: Vec<DhcpOption>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the set of DHCP options belongs to.
    pub vcn_id: String,
}

impl CreateDhcpDetails {
    /// Create a new CreateDhcpDetails with required fields
    pub fn new(required: CreateDhcpDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            options: required.options,

            vcn_id: required.vcn_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            domain_name_type: None,
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

    /// Set options
    pub fn set_options(mut self, value: Vec<DhcpOption>) -> Self {
        self.options = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set domain_name_type
    pub fn set_domain_name_type(mut self, value: Option<CreateDhcpDetailsDomainNameType>) -> Self {
        self.domain_name_type = value;
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

    /// Set domain_name_type (unwraps Option)
    pub fn with_domain_name_type(mut self, value: CreateDhcpDetailsDomainNameType) -> Self {
        self.domain_name_type = Some(value);
        self
    }
}
