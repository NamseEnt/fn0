use crate::auth::provider::AuthProvider;
use base64::{Engine as _, engine::general_purpose};
use rsa::{Pkcs1v15Sign, RsaPrivateKey, pkcs8::DecodePrivateKey};
use sha2::{Digest, Sha256};
use std::sync::Arc;

/// Request signer for OCI API calls
///
/// Implements HTTP Signature authentication as required by Oracle Cloud Infrastructure
pub struct RequestSigner {
    auth_provider: Arc<dyn AuthProvider>,
    private_key: RsaPrivateKey,
}

impl RequestSigner {
    /// Create a new request signer
    pub fn new(auth_provider: Arc<dyn AuthProvider>) -> crate::core::Result<Self> {
        let private_key_pem = auth_provider.get_private_key();

        // Parse RSA private key (PKCS#8 format)
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem).map_err(|e| {
            crate::core::OciError::SigningError(format!("Failed to parse private key: {}", e))
        })?;

        Ok(Self {
            auth_provider,
            private_key,
        })
    }

    /// Sign an HTTP request
    ///
    /// Adds the following headers to the request:
    /// - date or x-date: Current UTC time
    /// - host: Target host
    /// - (request-target): HTTP method and path
    /// - authorization: Signature with version="1"
    ///
    /// For POST/PUT/PATCH requests, also adds:
    /// - content-type: application/json (if not set)
    /// - content-length: Body length
    /// - x-content-sha256: SHA-256 hash of body
    pub fn sign_request(
        &self,
        method: &str,
        url: &url::Url,
        headers: &mut reqwest::header::HeaderMap,
        body: Option<&[u8]>,
    ) -> crate::core::Result<()> {
        let method_upper = method.to_uppercase();

        // 1. Add date header if not present
        if !headers.contains_key("date") && !headers.contains_key("x-date") {
            let now = chrono::Utc::now();
            headers.insert(
                "x-date",
                now.format("%a, %d %b %Y %H:%M:%S GMT")
                    .to_string()
                    .parse()
                    .map_err(|e| {
                        crate::core::OciError::SigningError(format!("Invalid date header: {}", e))
                    })?,
            );
        }

        // 2. Add host header if not present
        if !headers.contains_key("host")
            && let Some(host) = url.host_str()
        {
            let host_value = if let Some(port) = url.port() {
                format!("{}:{}", host, port)
            } else {
                host.to_string()
            };
            headers.insert(
                "host",
                host_value.parse().map_err(|e| {
                    crate::core::OciError::SigningError(format!("Invalid host header: {}", e))
                })?,
            );
        }

        // 3. Collect headers to sign
        let mut headers_to_sign = vec!["(request-target)", "host"];
        if headers.contains_key("x-date") {
            headers_to_sign.push("x-date");
        } else {
            headers_to_sign.push("date");
        }

        // 4. For POST/PUT/PATCH, add body-related headers
        if matches!(method_upper.as_str(), "POST" | "PUT" | "PATCH") {
            // Content-Type
            if !headers.contains_key("content-type") {
                headers.insert(
                    "content-type",
                    "application/json".parse().map_err(|e| {
                        crate::core::OciError::SigningError(format!("Invalid content-type: {}", e))
                    })?,
                );
            }

            // Content-SHA256 and Content-Length
            let (body_sha256, body_len) = if let Some(body_bytes) = body {
                let mut hasher = Sha256::new();
                hasher.update(body_bytes);
                let hash = hasher.finalize();
                let b64_hash = general_purpose::STANDARD.encode(hash);
                (b64_hash, body_bytes.len())
            } else {
                // Empty body
                let empty_sha = "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";
                (empty_sha.to_string(), 0)
            };

            headers.insert(
                "x-content-sha256",
                body_sha256.parse().map_err(|e| {
                    crate::core::OciError::SigningError(format!("Invalid sha256 header: {}", e))
                })?,
            );

            headers.insert(
                "content-length",
                body_len.to_string().parse().map_err(|e| {
                    crate::core::OciError::SigningError(format!("Invalid content-length: {}", e))
                })?,
            );

            headers_to_sign.extend_from_slice(&[
                "content-type",
                "content-length",
                "x-content-sha256",
            ]);
        }

        // 5. Build signing string
        let signing_string = self.build_signing_string(method, url, headers, &headers_to_sign)?;

        // 6. Sign the string
        let signature = self.sign_string(&signing_string)?;

        // 7. Build authorization header
        let key_id = self.auth_provider.get_key_id();
        let auth_header = format!(
            r#"Signature version="1",headers="{}",keyId="{}",algorithm="rsa-sha256",signature="{}""#,
            headers_to_sign.join(" "),
            key_id,
            signature
        );

        headers.insert(
            "authorization",
            auth_header.parse().map_err(|e| {
                crate::core::OciError::SigningError(format!("Invalid authorization header: {}", e))
            })?,
        );

        Ok(())
    }

    /// Build the signing string from request components
    fn build_signing_string(
        &self,
        method: &str,
        url: &url::Url,
        headers: &reqwest::header::HeaderMap,
        headers_to_sign: &[&str],
    ) -> crate::core::Result<String> {
        let mut parts = Vec::new();

        for header_name in headers_to_sign {
            let value = if *header_name == "(request-target)" {
                // Format: "method /path?query"
                let path = url.path();
                let query = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
                format!("{} {}{}", method.to_lowercase(), path, query)
            } else {
                // Get header value
                headers
                    .get(*header_name)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| {
                        crate::core::OciError::SigningError(format!(
                            "Missing header: {}",
                            header_name
                        ))
                    })?
                    .to_string()
            };

            parts.push(format!("{}: {}", header_name, value));
        }

        Ok(parts.join("\n"))
    }

    /// Sign a string using RSA-SHA256
    fn sign_string(&self, data: &str) -> crate::core::Result<String> {
        // Hash the data
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let hashed = hasher.finalize();

        // Sign with RSA PKCS#1 v1.5
        // Use unpadded signing since we already hashed the data
        let padding = Pkcs1v15Sign::new_unprefixed();
        let signature = self
            .private_key
            .sign(padding, &hashed)
            .map_err(|e| crate::core::OciError::SigningError(format!("Failed to sign: {}", e)))?;

        Ok(general_purpose::STANDARD.encode(&signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::provider::AuthProvider;
    use rsa::pkcs8::EncodePrivateKey;

    struct TestAuthProvider {
        key_pem: String,
    }

    impl AuthProvider for TestAuthProvider {
        fn get_key_id(&self) -> String {
            "ocid1.tenancy.oc1..test/ocid1.user.oc1..test/00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00".to_string()
        }

        fn get_private_key(&self) -> &str {
            &self.key_pem
        }

        fn get_passphrase(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn test_build_signing_string() {
        // Generate a test RSA key
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate key");
        let pem = private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .expect("failed to encode key");

        let auth_provider = Arc::new(TestAuthProvider {
            key_pem: pem.to_string(),
        });

        let signer = RequestSigner::new(auth_provider).unwrap();

        let url = url::Url::parse("https://osmh.ap-seoul-1.oci.oraclecloud.com/managedInstances")
            .unwrap();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-date", "Mon, 01 Jan 2024 00:00:00 GMT".parse().unwrap());
        headers.insert(
            "host",
            "osmh.ap-seoul-1.oci.oraclecloud.com".parse().unwrap(),
        );

        let signing_string = signer
            .build_signing_string(
                "GET",
                &url,
                &headers,
                &["(request-target)", "host", "x-date"],
            )
            .unwrap();

        assert!(signing_string.contains("(request-target): get /managedInstances"));
        assert!(signing_string.contains("host: osmh.ap-seoul-1.oci.oraclecloud.com"));
        assert!(signing_string.contains("x-date: Mon, 01 Jan 2024 00:00:00 GMT"));
    }
}
