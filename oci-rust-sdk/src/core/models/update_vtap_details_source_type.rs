use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateVtapDetailsSourceType {
    #[serde(rename = "VNIC")]
    Vnic,

    #[serde(rename = "SUBNET")]
    Subnet,

    #[serde(rename = "LOAD_BALANCER")]
    LoadBalancer,

    #[serde(rename = "DB_SYSTEM")]
    DbSystem,

    #[serde(rename = "EXADATA_VM_CLUSTER")]
    ExadataVmCluster,

    #[serde(rename = "AUTONOMOUS_DATA_WAREHOUSE")]
    AutonomousDataWarehouse,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
