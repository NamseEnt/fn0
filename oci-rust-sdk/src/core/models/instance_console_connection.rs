use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// The {@code InstanceConsoleConnection} API provides you with console access to Compute instances, enabling you to troubleshoot malfunctioning instances remotely. <p> For more information about instance console connections, see [Troubleshooting Instances Using Instance Console Connections](https://docs.oracle.com/iaas/Content/Compute/References/serialconsole.htm).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceConsoleConnection {
    /// The OCID of the compartment to contain the console connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,

    /// The SSH connection string for the console connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_string: Option<String>,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// The SSH public key's fingerprint for client authentication to the console connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The OCID of the console connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The OCID of the instance the console connection connects to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,

    /// The current state of the console connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<InstanceConsoleConnectionLifecycleState>,

    /// The SSH public key's fingerprint for the console connection service host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_host_key_fingerprint: Option<String>,

    /// The SSH connection string for the SSH tunnel used to connect to the console connection over VNC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnc_connection_string: Option<String>,
}

impl InstanceConsoleConnection {
    /// Create a new InstanceConsoleConnection
    pub fn new() -> Self {
        Self {
            compartment_id: None,

            connection_string: None,

            defined_tags: None,

            fingerprint: None,

            freeform_tags: None,

            id: None,

            instance_id: None,

            lifecycle_state: None,

            service_host_key_fingerprint: None,

            vnc_connection_string: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: Option<String>) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set connection_string
    pub fn set_connection_string(mut self, value: Option<String>) -> Self {
        self.connection_string = value;
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

    /// Set fingerprint
    pub fn set_fingerprint(mut self, value: Option<String>) -> Self {
        self.fingerprint = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set id
    pub fn set_id(mut self, value: Option<String>) -> Self {
        self.id = value;
        self
    }

    /// Set instance_id
    pub fn set_instance_id(mut self, value: Option<String>) -> Self {
        self.instance_id = value;
        self
    }

    /// Set lifecycle_state
    pub fn set_lifecycle_state(
        mut self,
        value: Option<InstanceConsoleConnectionLifecycleState>,
    ) -> Self {
        self.lifecycle_state = value;
        self
    }

    /// Set service_host_key_fingerprint
    pub fn set_service_host_key_fingerprint(mut self, value: Option<String>) -> Self {
        self.service_host_key_fingerprint = value;
        self
    }

    /// Set vnc_connection_string
    pub fn set_vnc_connection_string(mut self, value: Option<String>) -> Self {
        self.vnc_connection_string = value;
        self
    }

    /// Set compartment_id (unwraps Option)
    pub fn with_compartment_id(mut self, value: impl Into<String>) -> Self {
        self.compartment_id = Some(value.into());
        self
    }

    /// Set connection_string (unwraps Option)
    pub fn with_connection_string(mut self, value: impl Into<String>) -> Self {
        self.connection_string = Some(value.into());
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

    /// Set fingerprint (unwraps Option)
    pub fn with_fingerprint(mut self, value: impl Into<String>) -> Self {
        self.fingerprint = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set id (unwraps Option)
    pub fn with_id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    /// Set instance_id (unwraps Option)
    pub fn with_instance_id(mut self, value: impl Into<String>) -> Self {
        self.instance_id = Some(value.into());
        self
    }

    /// Set lifecycle_state (unwraps Option)
    pub fn with_lifecycle_state(mut self, value: InstanceConsoleConnectionLifecycleState) -> Self {
        self.lifecycle_state = Some(value);
        self
    }

    /// Set service_host_key_fingerprint (unwraps Option)
    pub fn with_service_host_key_fingerprint(mut self, value: impl Into<String>) -> Self {
        self.service_host_key_fingerprint = Some(value.into());
        self
    }

    /// Set vnc_connection_string (unwraps Option)
    pub fn with_vnc_connection_string(mut self, value: impl Into<String>) -> Self {
        self.vnc_connection_string = Some(value.into());
        self
    }
}

impl Default for InstanceConsoleConnection {
    fn default() -> Self {
        Self::new()
    }
}
