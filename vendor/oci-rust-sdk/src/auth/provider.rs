use std::sync::Arc;

/// Authentication provider trait for OCI SDK
///
/// Implementations of this trait provide the necessary credentials
/// to sign HTTP requests to OCI services.
pub trait AuthProvider: Send + Sync {
    /// Get the key ID (e.g., "ocid1.tenancy.oc1..xxxxx/ocid1.user.oc1..xxxxx/fingerprint")
    fn get_key_id(&self) -> String;

    /// Get the private key in PEM format
    fn get_private_key(&self) -> &str;

    /// Get the passphrase for the private key (if any)
    fn get_passphrase(&self) -> Option<&str>;
}

/// Type alias for Arc-wrapped AuthProvider
pub type AuthProviderRef = Arc<dyn AuthProvider>;
