pub mod provider;
pub mod config;
pub mod simple;
pub mod signer;
pub mod instance_principals;
pub mod imds;
pub mod federation;
pub mod x509_utils;

pub use provider::{AuthProvider, AuthProviderRef};
pub use config::ConfigFileAuthProvider;
pub use simple::{SimpleAuthProvider, SimpleAuthProviderBuilder, SimpleAuthProviderRequiredFields};
pub use signer::RequestSigner;
pub use instance_principals::InstancePrincipalsAuthProvider;
