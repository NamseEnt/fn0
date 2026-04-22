use serde::{Deserialize, Serialize};

/// The attachment type of a BM volume. If the attachment is in-transit encryption, the field is BM_ENCRYPTION_IN_TRANSIT. Otherwise, the field is None.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EncryptionInTransitType {
    #[serde(rename = "NONE")]
    None,

    #[serde(rename = "BM_ENCRYPTION_IN_TRANSIT")]
    BmEncryptionInTransit,

    /// This value is used if a service returns a value for this enum that is not recognized by this version of the SDK.
    #[serde(other)]
    UnknownValue,
}
