use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastConnectProviderServiceBandwithShapeManagement {
    #[serde(rename = "CUSTOMER_MANAGED")]
    CustomerManaged,

    #[serde(rename = "PROVIDER_MANAGED")]
    ProviderManaged,

    #[serde(rename = "ORACLE_MANAGED")]
    OracleManaged,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
