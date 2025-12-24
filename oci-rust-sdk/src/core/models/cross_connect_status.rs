use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// The status of the cross-connect.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossConnectStatus {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,

    /// Indicates whether Oracle's side of the interface is up or down.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_state: Option<CrossConnectStatusInterfaceState>,

    /// The light level of the cross-connect (in dBm). <p> Example: {@code 14.0} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_level_ind_bm: Option<i64>,

    /// Status indicator corresponding to the light level. <p> **NO_LIGHT:** No measurable light * **LOW_WARN:** There's measurable light but it's too low * **HIGH_WARN:** Light level is too high * **BAD:** There's measurable light but the signal-to-noise ratio is bad * **GOOD:** Good light level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_level_indicator: Option<CrossConnectStatusLightLevelIndicator>,

    /// Encryption status of this cross connect. <p> Possible values: * **UP:** Traffic is encrypted over this cross-connect * **DOWN:** Traffic is not encrypted over this cross-connect * **CIPHER_MISMATCH:** The MACsec encryption cipher doesn't match the cipher on the CPE * **CKN_MISMATCH:** The MACsec Connectivity association Key Name (CKN) doesn't match the CKN on the CPE * **CAK_MISMATCH:** The MACsec Connectivity Association Key (CAK) doesn't match the CAK on the CPE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_status: Option<CrossConnectStatusEncryptionStatus>,

    /// The light levels of the cross-connect (in dBm). <p> Example: {@code [14.0, -14.0, 2.1, -10.1]}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_levels_in_d_bm: Option<Vec<i64>>,
}

/// Required fields for CrossConnectStatus
pub struct CrossConnectStatusRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the cross-connect.
    pub cross_connect_id: String,
}

impl CrossConnectStatus {
    /// Create a new CrossConnectStatus with required fields
    pub fn new(required: CrossConnectStatusRequired) -> Self {
        Self {
            cross_connect_id: required.cross_connect_id,

            interface_state: None,

            light_level_ind_bm: None,

            light_level_indicator: None,

            encryption_status: None,

            light_levels_in_d_bm: None,
        }
    }

    /// Set cross_connect_id
    pub fn set_cross_connect_id(mut self, value: String) -> Self {
        self.cross_connect_id = value;
        self
    }

    /// Set interface_state
    pub fn set_interface_state(mut self, value: Option<CrossConnectStatusInterfaceState>) -> Self {
        self.interface_state = value;
        self
    }

    /// Set light_level_ind_bm
    pub fn set_light_level_ind_bm(mut self, value: Option<i64>) -> Self {
        self.light_level_ind_bm = value;
        self
    }

    /// Set light_level_indicator
    pub fn set_light_level_indicator(
        mut self,
        value: Option<CrossConnectStatusLightLevelIndicator>,
    ) -> Self {
        self.light_level_indicator = value;
        self
    }

    /// Set encryption_status
    pub fn set_encryption_status(
        mut self,
        value: Option<CrossConnectStatusEncryptionStatus>,
    ) -> Self {
        self.encryption_status = value;
        self
    }

    /// Set light_levels_in_d_bm
    pub fn set_light_levels_in_d_bm(mut self, value: Option<Vec<i64>>) -> Self {
        self.light_levels_in_d_bm = value;
        self
    }

    /// Set interface_state (unwraps Option)
    pub fn with_interface_state(mut self, value: CrossConnectStatusInterfaceState) -> Self {
        self.interface_state = Some(value);
        self
    }

    /// Set light_level_ind_bm (unwraps Option)
    pub fn with_light_level_ind_bm(mut self, value: i64) -> Self {
        self.light_level_ind_bm = Some(value);
        self
    }

    /// Set light_level_indicator (unwraps Option)
    pub fn with_light_level_indicator(
        mut self,
        value: CrossConnectStatusLightLevelIndicator,
    ) -> Self {
        self.light_level_indicator = Some(value);
        self
    }

    /// Set encryption_status (unwraps Option)
    pub fn with_encryption_status(mut self, value: CrossConnectStatusEncryptionStatus) -> Self {
        self.encryption_status = Some(value);
        self
    }

    /// Set light_levels_in_d_bm (unwraps Option)
    pub fn with_light_levels_in_d_bm(mut self, value: Vec<i64>) -> Self {
        self.light_levels_in_d_bm = Some(value);
        self
    }
}
