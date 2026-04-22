use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// A container object for identity attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    /// The name of the user or service. This value is the friendly name associated with {@code principalId}. <p> Example: {@code ExampleName}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the principal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,

    /// The type of authentication used. <p> Example: {@code natv}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<String>,

    /// The name of the user or service. This value is the friendly name associated with {@code callerId}.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller_name: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the caller. The caller that made a request on behalf of the prinicpal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller_id: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the tenant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,

    /// The IP address of the source of the request. <p> Example: {@code 172.24.80.88}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_address: Option<String>,

    /// The credential ID of the user. This value is extracted from the HTTP 'Authorization' request header. It consists of the tenantId, userId, and user fingerprint, all delimited by a slash (/).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<String>,

    /// The user agent of the client that made the request. <p> Example: {@code Jersey/2.23 (HttpUrlConnection 1.8.0_212)}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,

    /// This value identifies any Console session associated with this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub console_session_id: Option<String>,
}

impl Identity {
    /// Create a new Identity
    pub fn new() -> Self {
        Self {
            principal_name: None,

            principal_id: None,

            auth_type: None,

            caller_name: None,

            caller_id: None,

            tenant_id: None,

            ip_address: None,

            credentials: None,

            user_agent: None,

            console_session_id: None,
        }
    }

    /// Set principal_name
    pub fn set_principal_name(mut self, value: Option<String>) -> Self {
        self.principal_name = value;
        self
    }

    /// Set principal_id
    pub fn set_principal_id(mut self, value: Option<String>) -> Self {
        self.principal_id = value;
        self
    }

    /// Set auth_type
    pub fn set_auth_type(mut self, value: Option<String>) -> Self {
        self.auth_type = value;
        self
    }

    /// Set caller_name
    pub fn set_caller_name(mut self, value: Option<String>) -> Self {
        self.caller_name = value;
        self
    }

    /// Set caller_id
    pub fn set_caller_id(mut self, value: Option<String>) -> Self {
        self.caller_id = value;
        self
    }

    /// Set tenant_id
    pub fn set_tenant_id(mut self, value: Option<String>) -> Self {
        self.tenant_id = value;
        self
    }

    /// Set ip_address
    pub fn set_ip_address(mut self, value: Option<String>) -> Self {
        self.ip_address = value;
        self
    }

    /// Set credentials
    pub fn set_credentials(mut self, value: Option<String>) -> Self {
        self.credentials = value;
        self
    }

    /// Set user_agent
    pub fn set_user_agent(mut self, value: Option<String>) -> Self {
        self.user_agent = value;
        self
    }

    /// Set console_session_id
    pub fn set_console_session_id(mut self, value: Option<String>) -> Self {
        self.console_session_id = value;
        self
    }

    /// Set principal_name (unwraps Option)
    pub fn with_principal_name(mut self, value: impl Into<String>) -> Self {
        self.principal_name = Some(value.into());
        self
    }

    /// Set principal_id (unwraps Option)
    pub fn with_principal_id(mut self, value: impl Into<String>) -> Self {
        self.principal_id = Some(value.into());
        self
    }

    /// Set auth_type (unwraps Option)
    pub fn with_auth_type(mut self, value: impl Into<String>) -> Self {
        self.auth_type = Some(value.into());
        self
    }

    /// Set caller_name (unwraps Option)
    pub fn with_caller_name(mut self, value: impl Into<String>) -> Self {
        self.caller_name = Some(value.into());
        self
    }

    /// Set caller_id (unwraps Option)
    pub fn with_caller_id(mut self, value: impl Into<String>) -> Self {
        self.caller_id = Some(value.into());
        self
    }

    /// Set tenant_id (unwraps Option)
    pub fn with_tenant_id(mut self, value: impl Into<String>) -> Self {
        self.tenant_id = Some(value.into());
        self
    }

    /// Set ip_address (unwraps Option)
    pub fn with_ip_address(mut self, value: impl Into<String>) -> Self {
        self.ip_address = Some(value.into());
        self
    }

    /// Set credentials (unwraps Option)
    pub fn with_credentials(mut self, value: impl Into<String>) -> Self {
        self.credentials = Some(value.into());
        self
    }

    /// Set user_agent (unwraps Option)
    pub fn with_user_agent(mut self, value: impl Into<String>) -> Self {
        self.user_agent = Some(value.into());
        self
    }

    /// Set console_session_id (unwraps Option)
    pub fn with_console_session_id(mut self, value: impl Into<String>) -> Self {
        self.console_session_id = Some(value.into());
        self
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}
