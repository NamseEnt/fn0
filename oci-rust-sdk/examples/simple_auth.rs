use oci_rust_sdk::core::auth::{AuthProvider, SimpleAuthProvider, SimpleAuthProviderRequiredFields};
use oci_rust_sdk::core::region::Region;

fn main() {
    println!("SimpleAuthProvider Example\n");

    // Example 1: Using the constructor
    println!("=== Example 1: Using new() constructor ===");
    let provider1 = SimpleAuthProvider::new(
        "ocid1.tenancy.oc1..aaaaaaaexample",
        "ocid1.user.oc1..aaaaaaaexample",
        "aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66:77:88:99",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----",
    );

    println!("Tenancy: {}", provider1.tenancy());
    println!("User: {}", provider1.user());
    println!("Key ID: {}", provider1.get_key_id());
    println!("Region: {:?}\n", provider1.region());

    // Example 2: Using the builder pattern (recommended)
    println!("=== Example 2: Using builder pattern ===");
    let required = SimpleAuthProviderRequiredFields {
        tenancy: "ocid1.tenancy.oc1..aaaaaaaexample".to_string(),
        user: "ocid1.user.oc1..aaaaaaaexample".to_string(),
        fingerprint: "aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66:77:88:99".to_string(),
        private_key: "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----".to_string(),
    };

    let provider2 = SimpleAuthProvider::builder(required)
        .passphrase("my-secret-passphrase")
        .region(Region::ApSeoul1)
        .auth_type("api_key")
        .build();

    println!("Tenancy: {}", provider2.tenancy());
    println!("User: {}", provider2.user());
    println!("Key ID: {}", provider2.get_key_id());
    println!("Region: {:?}", provider2.region());
    println!("Auth Type: {:?}", provider2.auth_type());
    println!("Has Passphrase: {}\n", provider2.get_passphrase().is_some());

    // Example 3: Setting region after creation
    println!("=== Example 3: Setting region dynamically ===");
    let mut provider3 = SimpleAuthProvider::new(
        "ocid1.tenancy.oc1..aaaaaaaexample",
        "ocid1.user.oc1..aaaaaaaexample",
        "aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66:77:88:99",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----",
    );

    println!("Initial region: {:?}", provider3.region());
    provider3.set_region(Region::UsPhoenix1);
    println!("After set_region: {:?}\n", provider3.region());

    // Example 4: Using with_* methods (fluent API)
    println!("=== Example 4: Fluent API with with_* methods ===");
    let provider4 = SimpleAuthProvider::new(
        "ocid1.tenancy.oc1..aaaaaaaexample",
        "ocid1.user.oc1..aaaaaaaexample",
        "aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66:77:88:99",
        "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----",
    )
    .with_region(Region::EuFrankfurt1)
    .with_auth_type("instance_principal")
    .with_delegation_token("delegation-token-here")
    .with_session_token("session-token-here");

    println!("Region: {:?}", provider4.region());
    println!("Auth Type: {:?}", provider4.auth_type());
    println!("Delegation Token: {:?}", provider4.delegation_token());
    println!("Session Token: {:?}\n", provider4.session_token());
}
