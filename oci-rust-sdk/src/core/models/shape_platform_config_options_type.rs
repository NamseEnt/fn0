use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapePlatformConfigOptionsType {
    #[serde(rename = "AMD_MILAN_BM")]
    AmdMilanBm,

    #[serde(rename = "AMD_MILAN_BM_GPU")]
    AmdMilanBmGpu,

    #[serde(rename = "AMD_ROME_BM")]
    AmdRomeBm,

    #[serde(rename = "AMD_ROME_BM_GPU")]
    AmdRomeBmGpu,

    #[serde(rename = "GENERIC_BM")]
    GenericBm,

    #[serde(rename = "INTEL_ICELAKE_BM")]
    IntelIcelakeBm,

    #[serde(rename = "INTEL_SKYLAKE_BM")]
    IntelSkylakeBm,

    #[serde(rename = "AMD_VM")]
    AmdVm,

    #[serde(rename = "INTEL_VM")]
    IntelVm,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
