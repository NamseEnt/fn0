pub mod config;
pub mod federation;
pub mod imds;
pub mod instance_principals;
pub mod provider;
pub mod signer;
pub mod simple;
pub mod x509_utils;

pub use config::ConfigFileAuthProvider;
pub use instance_principals::InstancePrincipalsAuthProvider;
pub use provider::{AuthProvider, AuthProviderRef};
pub use signer::RequestSigner;
pub use simple::{SimpleAuthProvider, SimpleAuthProviderBuilder, SimpleAuthProviderRequiredFields};
