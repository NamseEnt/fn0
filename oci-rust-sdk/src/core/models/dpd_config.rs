use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// These configuration details are used for dead peer detection (DPD). DPD periodically checks the stability of the connection to the customer premises (CPE), and may be used to detect that the link to the CPE has gone down.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DpdConfig {
    /// This option defines whether DPD can be initiated from the Oracle side of the connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpd_mode: Option<DpdConfigDpdMode>,

    /// DPD timeout in seconds. This sets the longest interval between CPE device health messages before the IPSec connection indicates it has lost contact with the CPE. The default is 20 seconds. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dpd_timeout_in_sec: Option<i64>,
}

impl DpdConfig {
    /// Create a new DpdConfig
    pub fn new() -> Self {
        Self {
            dpd_mode: None,

            dpd_timeout_in_sec: None,
        }
    }

    /// Set dpd_mode
    pub fn set_dpd_mode(mut self, value: Option<DpdConfigDpdMode>) -> Self {
        self.dpd_mode = value;
        self
    }

    /// Set dpd_timeout_in_sec
    pub fn set_dpd_timeout_in_sec(mut self, value: Option<i64>) -> Self {
        self.dpd_timeout_in_sec = value;
        self
    }

    /// Set dpd_mode (unwraps Option)
    pub fn with_dpd_mode(mut self, value: DpdConfigDpdMode) -> Self {
        self.dpd_mode = Some(value);
        self
    }

    /// Set dpd_timeout_in_sec (unwraps Option)
    pub fn with_dpd_timeout_in_sec(mut self, value: i64) -> Self {
        self.dpd_timeout_in_sec = Some(value);
        self
    }
}

impl Default for DpdConfig {
    fn default() -> Self {
        Self::new()
    }
}
