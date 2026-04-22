use color_eyre::eyre::{Result, eyre};
use reqsign::{AwsCredential, AwsV4Signer};
use std::sync::Arc;

#[derive(Clone)]
pub struct LambdaClient {
    http: reqwest::Client,
    signer: Arc<AwsV4Signer>,
    credential: AwsCredential,
    endpoint: String,
}

pub struct InvokeResponse {
    pub function_error: Option<String>,
    pub payload: Vec<u8>,
}

impl LambdaClient {
    pub fn new(region: &str, access_key_id: &str, secret_access_key: &str) -> Self {
        let endpoint = format!("https://lambda.{region}.amazonaws.com");
        Self {
            http: reqwest::Client::new(),
            signer: Arc::new(AwsV4Signer::new("lambda", region)),
            credential: AwsCredential {
                access_key_id: access_key_id.to_string(),
                secret_access_key: secret_access_key.to_string(),
                session_token: None,
                expires_in: None,
            },
            endpoint,
        }
    }

    pub async fn invoke(&self, function_name: &str, payload: Vec<u8>) -> Result<InvokeResponse> {
        let url = format!(
            "{}/2015-03-31/functions/{function_name}/invocations",
            self.endpoint
        );
        let mut req = http::Request::builder()
            .method(http::Method::POST)
            .uri(&url)
            .header(http::header::CONTENT_TYPE, "application/json")
            .body(payload)
            .map_err(|e| eyre!("Failed to build Lambda request: {e}"))?;

        self.signer
            .sign(&mut req, &self.credential)
            .map_err(|e| eyre!("Failed to sign Lambda request: {e}"))?;

        let (parts, body) = req.into_parts();
        let mut reqwest_req = self
            .http
            .request(parts.method, parts.uri.to_string())
            .body(body);
        for (name, value) in parts.headers.iter() {
            reqwest_req = reqwest_req.header(name, value);
        }

        let resp = reqwest_req
            .send()
            .await
            .map_err(|e| eyre!("Lambda invoke request failed: {e}"))?;

        let function_error = resp
            .headers()
            .get("X-Amz-Function-Error")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let status = resp.status();
        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| eyre!("Failed to read Lambda response body: {e}"))?;

        if !status.is_success() && function_error.is_none() {
            return Err(eyre!(
                "Lambda invoke HTTP {status}: {}",
                String::from_utf8_lossy(&body_bytes)
            ));
        }

        Ok(InvokeResponse {
            function_error,
            payload: body_bytes.to_vec(),
        })
    }
}
