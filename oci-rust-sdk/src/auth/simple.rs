use crate::auth::provider::AuthProvider;
use crate::core::region::Region;

/// Required fields for SimpleAuthProvider
pub struct SimpleAuthProviderRequiredFields {
    /// Tenancy OCID
    pub tenancy: String,
    /// User OCID
    pub user: String,
    /// Public key fingerprint
    pub fingerprint: String,
    /// Private key in PEM format
    pub private_key: String,
}

/// Simple authentication provider that holds credentials directly in memory
///
/// This is the Rust equivalent of TypeScript SDK's SimpleAuthenticationDetailsProvider.
/// It provides a straightforward way to create an authentication provider with
/// credentials provided directly, without reading from configuration files.
///
/// # Example
///
/// ```
/// use oci_rust_sdk::core::auth::{SimpleAuthProvider, SimpleAuthProviderRequiredFields};
/// use oci_rust_sdk::core::region::Region;
///
/// let required = SimpleAuthProviderRequiredFields {
///     tenancy: "ocid1.tenancy.oc1..aaa...".to_string(),
///     user: "ocid1.user.oc1..aaa...".to_string(),
///     fingerprint: "aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66:77:88:99".to_string(),
///     private_key: "-----BEGIN RSA PRIVATE KEY-----\n...".to_string(),
/// };
///
/// let provider = SimpleAuthProvider::builder(required)
///     .region(Region::ApSeoul1)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SimpleAuthProvider {
    tenancy: String,
    user: String,
    fingerprint: String,
    private_key: String,
    passphrase: Option<String>,
    region: Option<Region>,
    auth_type: Option<String>,
    delegation_token: Option<String>,
    session_token: Option<String>,
}

impl SimpleAuthProvider {
    /// Create a new SimpleAuthProvider with required fields
    ///
    /// # Arguments
    ///
    /// * `tenancy` - Tenancy OCID
    /// * `user` - User OCID
    /// * `fingerprint` - Public key fingerprint
    /// * `private_key` - Private key in PEM format
    pub fn new(
        tenancy: impl Into<String>,
        user: impl Into<String>,
        fingerprint: impl Into<String>,
        private_key: impl Into<String>,
    ) -> Self {
        Self {
            tenancy: tenancy.into(),
            user: user.into(),
            fingerprint: fingerprint.into(),
            private_key: private_key.into(),
            passphrase: None,
            region: None,
            auth_type: None,
            delegation_token: None,
            session_token: None,
        }
    }

    /// Create a builder for constructing a SimpleAuthProvider
    pub fn builder(required: SimpleAuthProviderRequiredFields) -> SimpleAuthProviderBuilder {
        SimpleAuthProviderBuilder {
            tenancy: required.tenancy,
            user: required.user,
            fingerprint: required.fingerprint,
            private_key: required.private_key,
            passphrase: None,
            region: None,
            auth_type: None,
            delegation_token: None,
            session_token: None,
        }
    }

    /// Get the tenancy OCID
    pub fn tenancy(&self) -> &str {
        &self.tenancy
    }

    /// Get the user OCID
    pub fn user(&self) -> &str {
        &self.user
    }

    /// Get the fingerprint
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Get the region (if set)
    pub fn region(&self) -> Option<Region> {
        self.region
    }

    /// Set the region
    pub fn set_region(&mut self, region: Region) {
        self.region = Some(region);
    }

    /// Set the region (builder style)
    pub fn with_region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Get the authentication type
    pub fn auth_type(&self) -> Option<&str> {
        self.auth_type.as_deref()
    }

    /// Set the authentication type
    pub fn set_auth_type(&mut self, auth_type: impl Into<String>) {
        self.auth_type = Some(auth_type.into());
    }

    /// Set the authentication type (builder style)
    pub fn with_auth_type(mut self, auth_type: impl Into<String>) -> Self {
        self.auth_type = Some(auth_type.into());
        self
    }

    /// Get the delegation token
    pub fn delegation_token(&self) -> Option<&str> {
        self.delegation_token.as_deref()
    }

    /// Set the delegation token
    pub fn set_delegation_token(&mut self, token: impl Into<String>) {
        self.delegation_token = Some(token.into());
    }

    /// Set the delegation token (builder style)
    pub fn with_delegation_token(mut self, token: impl Into<String>) -> Self {
        self.delegation_token = Some(token.into());
        self
    }

    /// Get the session token
    pub fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }

    /// Set the session token
    pub fn set_session_token(&mut self, token: impl Into<String>) {
        self.session_token = Some(token.into());
    }

    /// Set the session token (builder style)
    pub fn with_session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }
}

impl AuthProvider for SimpleAuthProvider {
    fn get_key_id(&self) -> String {
        format!("{}/{}/{}", self.tenancy, self.user, self.fingerprint)
    }

    fn get_private_key(&self) -> &str {
        &self.private_key
    }

    fn get_passphrase(&self) -> Option<&str> {
        self.passphrase.as_deref()
    }
}

/// Builder for SimpleAuthProvider
///
/// Provides a convenient way to construct a SimpleAuthProvider with optional fields.
#[derive(Debug)]
pub struct SimpleAuthProviderBuilder {
    tenancy: String,
    user: String,
    fingerprint: String,
    private_key: String,
    passphrase: Option<String>,
    region: Option<Region>,
    auth_type: Option<String>,
    delegation_token: Option<String>,
    session_token: Option<String>,
}

impl SimpleAuthProviderBuilder {
    /// Set the passphrase for the private key
    pub fn passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    /// Set the region
    pub fn region(mut self, region: Region) -> Self {
        self.region = Some(region);
        self
    }

    /// Set the authentication type
    pub fn auth_type(mut self, auth_type: impl Into<String>) -> Self {
        self.auth_type = Some(auth_type.into());
        self
    }

    /// Set the delegation token
    pub fn delegation_token(mut self, token: impl Into<String>) -> Self {
        self.delegation_token = Some(token.into());
        self
    }

    /// Set the session token
    pub fn session_token(mut self, token: impl Into<String>) -> Self {
        self.session_token = Some(token.into());
        self
    }

    /// Build the SimpleAuthProvider
    pub fn build(self) -> SimpleAuthProvider {
        SimpleAuthProvider {
            tenancy: self.tenancy,
            user: self.user,
            fingerprint: self.fingerprint,
            private_key: self.private_key,
            passphrase: self.passphrase,
            region: self.region,
            auth_type: self.auth_type,
            delegation_token: self.delegation_token,
            session_token: self.session_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_auth_provider_new() {
        let provider = SimpleAuthProvider::new(
            "ocid1.tenancy.oc1..aaa",
            "ocid1.user.oc1..bbb",
            "aa:bb:cc:dd:ee:ff",
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        );

        assert_eq!(provider.tenancy(), "ocid1.tenancy.oc1..aaa");
        assert_eq!(provider.user(), "ocid1.user.oc1..bbb");
        assert_eq!(provider.fingerprint(), "aa:bb:cc:dd:ee:ff");
        assert_eq!(provider.region(), None);
    }

    #[test]
    fn test_simple_auth_provider_builder() {
        let required = SimpleAuthProviderRequiredFields {
            tenancy: "ocid1.tenancy.oc1..aaa".to_string(),
            user: "ocid1.user.oc1..bbb".to_string(),
            fingerprint: "aa:bb:cc:dd:ee:ff".to_string(),
            private_key: "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----"
                .to_string(),
        };

        let provider = SimpleAuthProvider::builder(required)
            .passphrase("mypassphrase")
            .region(Region::ApSeoul1)
            .auth_type("api_key")
            .build();

        assert_eq!(provider.tenancy(), "ocid1.tenancy.oc1..aaa");
        assert_eq!(provider.user(), "ocid1.user.oc1..bbb");
        assert_eq!(provider.fingerprint(), "aa:bb:cc:dd:ee:ff");
        assert_eq!(provider.region(), Some(Region::ApSeoul1));
        assert_eq!(provider.auth_type(), Some("api_key"));
        assert_eq!(provider.get_passphrase(), Some("mypassphrase"));
    }

    #[test]
    fn test_auth_provider_trait() {
        let provider = SimpleAuthProvider::new(
            "ocid1.tenancy.oc1..aaa",
            "ocid1.user.oc1..bbb",
            "aa:bb:cc:dd:ee:ff",
            "-----BEGIN RSA PRIVATE KEY-----\ntest\n-----END RSA PRIVATE KEY-----",
        );

        assert_eq!(
            provider.get_key_id(),
            "ocid1.tenancy.oc1..aaa/ocid1.user.oc1..bbb/aa:bb:cc:dd:ee:ff"
        );
        assert!(provider.get_private_key().contains("BEGIN RSA PRIVATE KEY"));
        assert_eq!(provider.get_passphrase(), None);
    }

    #[test]
    fn test_region_management() {
        let mut provider = SimpleAuthProvider::new(
            "ocid1.tenancy.oc1..aaa",
            "ocid1.user.oc1..bbb",
            "aa:bb:cc:dd:ee:ff",
            "key",
        );

        assert_eq!(provider.region(), None);

        provider.set_region(Region::ApSeoul1);
        assert_eq!(provider.region(), Some(Region::ApSeoul1));

        let provider = provider.with_region(Region::UsPhoenix1);
        assert_eq!(provider.region(), Some(Region::UsPhoenix1));
    }

    #[test]
    fn test_token_management() {
        let required = SimpleAuthProviderRequiredFields {
            tenancy: "t".to_string(),
            user: "u".to_string(),
            fingerprint: "f".to_string(),
            private_key: "k".to_string(),
        };

        let provider = SimpleAuthProvider::builder(required)
            .delegation_token("delegation123")
            .session_token("session456")
            .build();

        assert_eq!(provider.delegation_token(), Some("delegation123"));
        assert_eq!(provider.session_token(), Some("session456"));
    }
}
