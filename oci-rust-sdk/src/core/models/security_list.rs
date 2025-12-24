use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// A set of virtual firewall rules for your VCN. Security lists are configured at the subnet level, but the rules are applied to the ingress and egress traffic for the individual instances in the subnet. The rules can be stateful or stateless. For more information, see [Security Lists](https://docs.oracle.com/iaas/Content/Network/Concepts/securitylists.htm). **Note:** Compare security lists to {@link NetworkSecurityGroup}s, which let you apply a set of security rules to a *specific set of VNICs* instead of an entire subnet. Oracle recommends using network security groups instead of security lists, although you can use either or both together. <p> *Important:** Oracle Cloud Infrastructure Compute service images automatically include firewall rules (for example, Linux iptables, Windows firewall). If there are issues with some type of access to an instance, make sure both the security lists associated with the instance's subnet and the instance's firewall rules are set correctly. <p> To use any of the API operations, you must be authorized in an IAM policy. If you're not authorized, talk to an administrator. If you're an administrator who needs to write policies to give users access, see [Getting Started with Policies](https://docs.oracle.com/iaas/Content/Identity/Concepts/policygetstarted.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityList {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the security list.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// Rules for allowing egress IP packets.
    pub egress_security_rules: Vec<EgressSecurityRule>,

    /// The security list's Oracle Cloud ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// Rules for allowing ingress IP packets.
    pub ingress_security_rules: Vec<IngressSecurityRule>,

    /// The security list's current state.
    pub lifecycle_state: SecurityListLifecycleState,

    /// The date and time the security list was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the security list belongs to.
    pub vcn_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,
}

/// Required fields for SecurityList
pub struct SecurityListRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the security list.
    pub compartment_id: String,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    pub display_name: String,

    /// Rules for allowing egress IP packets.
    pub egress_security_rules: Vec<EgressSecurityRule>,

    /// The security list's Oracle Cloud ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub id: String,

    /// Rules for allowing ingress IP packets.
    pub ingress_security_rules: Vec<IngressSecurityRule>,

    /// The security list's current state.
    pub lifecycle_state: SecurityListLifecycleState,

    /// The date and time the security list was created, in the format defined by [RFC3339](https://tools.ietf.org/html/rfc3339). <p> Example: {@code 2016-08-25T21:10:29.600Z}
    pub time_created: DateTime<Utc>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN the security list belongs to.
    pub vcn_id: String,
}

impl SecurityList {
    /// Create a new SecurityList with required fields
    pub fn new(required: SecurityListRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            display_name: required.display_name,

            egress_security_rules: required.egress_security_rules,

            id: required.id,

            ingress_security_rules: required.ingress_security_rules,

            lifecycle_state: required.lifecycle_state,

            time_created: required.time_created,

            vcn_id: required.vcn_id,

            defined_tags: None,

            freeform_tags: None,
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
    pub fn set_display_name(mut self, value: String) -> Self {
        self.display_name = value;
        self
    }

    /// Set egress_security_rules
    pub fn set_egress_security_rules(mut self, value: Vec<EgressSecurityRule>) -> Self {
        self.egress_security_rules = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: String) -> Self {
        self.id = value;
        self
    }

    /// Set ingress_security_rules
    pub fn set_ingress_security_rules(mut self, value: Vec<IngressSecurityRule>) -> Self {
        self.ingress_security_rules = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(mut self, value: SecurityListLifecycleState) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set time_created
    pub fn set_time_created(mut self, value: DateTime<Utc>) -> Self {
        self.time_created = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
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

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }
}
