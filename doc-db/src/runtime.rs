use anyhow::Result;
use dibi_protocol::MAX_FRAME_SIZE;
use std::time::Duration;

const DATABASE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(target_arch = "wasm32")]
pub(crate) async fn dibi_request(endpoint: &str, frame: &[u8]) -> anyhow::Result<Vec<u8>> {
    use anyhow::bail;
    use forte_sdk::http::{Client, HeaderValue, Request, RequestTimeouts, Uri};
    use std::str::FromStr;

    let request_endpoint = if endpoint.ends_with('/') {
        endpoint.to_owned()
    } else {
        format!("{endpoint}/")
    };
    let uri = Uri::from_str(&request_endpoint)
        .map_err(|error| anyhow::anyhow!("Invalid Dibi URI: {error}"))?;
    let request = Request::post(&uri)
        .header(
            "Content-Type",
            HeaderValue::from_static("application/octet-stream"),
        )
        .body(frame.to_vec())
        .map_err(|error| anyhow::anyhow!("Failed to build Dibi request: {error}"))?;
    let client = Client::new().with_timeouts(RequestTimeouts::all(DATABASE_RESPONSE_TIMEOUT));
    let response = client
        .send(request)
        .await
        .map_err(|error| anyhow::anyhow!("Dibi host transport error for {request_endpoint}: {error:?}"))?;
    if !response.status().is_success() {
        bail!("Dibi host transport returned HTTP status {}", response.status());
    }
    let bytes = response
        .into_body()
        .bytes_limited(MAX_FRAME_SIZE)
        .await
        .map_err(|error| anyhow::anyhow!("Dibi response body error: {error}"))?;
    Ok(bytes.to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn dibi_request(_endpoint: &str, _frame: &[u8]) -> anyhow::Result<Vec<u8>> {
    anyhow::bail!("Dibi host transport is unavailable outside the fn0 WASM runtime")
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn http_post_json(
    url: &str,
    body: Vec<u8>,
    auth_token: Option<&str>,
) -> Result<Vec<u8>> {
    use anyhow::bail;
    use forte_sdk::http::{Client, HeaderValue, Request, RequestTimeouts, Uri};
    use std::str::FromStr;

    let uri = Uri::from_str(url).map_err(|e| anyhow::anyhow!("Invalid URI: {e}"))?;
    let mut builder = Request::post(&uri).header(
        "Content-Type",
        HeaderValue::from_str("application/json")
            .map_err(|e| anyhow::anyhow!("Invalid Content-Type: {e}"))?,
    );
    if let Some(token) = auth_token {
        builder = builder.header(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|e| anyhow::anyhow!("Invalid Authorization: {e}"))?,
        );
    }
    let request = builder
        .body(body)
        .map_err(|e| anyhow::anyhow!("Failed to build request: {e}"))?;
    let client = Client::new().with_timeouts(RequestTimeouts::all(DATABASE_RESPONSE_TIMEOUT));
    let response = client.send(request).await?;
    if !response.status().is_success() {
        bail!("HTTP request failed with status: {}", response.status());
    }
    let bytes = response.into_body().bytes_limited(usize::MAX).await?;
    Ok(bytes.to_vec())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn http_post_json(
    url: &str,
    body: Vec<u8>,
    auth_token: Option<&str>,
) -> Result<Vec<u8>> {
    use anyhow::bail;
    use std::sync::OnceLock;

    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    let client = CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(DATABASE_RESPONSE_TIMEOUT)
            .read_timeout(DATABASE_RESPONSE_TIMEOUT)
            .build()
            .expect("database http client must build")
    });

    let mut req = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body);
    if let Some(token) = auth_token {
        req = req.header("Authorization", format!("Bearer {token}"));
    }
    let resp = req.send().await?;
    if !resp.status().is_success() {
        bail!("HTTP request failed with status: {}", resp.status());
    }
    Ok(resp.bytes().await?.to_vec())
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn sleep(duration: Duration) {
    forte_sdk::time_wasi::sleep(duration).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn random_bytes(buf: &mut [u8]) {
    forte_sdk::rand::get_insecure_random_bytes(buf);
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn random_bytes(buf: &mut [u8]) {
    use rand::RngCore;
    rand::thread_rng().fill_bytes(buf);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn unresponsive_database_fails_within_the_response_timeout() {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let silent_server = tokio::spawn(async move {
            let (_connection, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(60)).await;
        });

        let started = std::time::Instant::now();
        let result = http_post_json(
            &format!("http://127.0.0.1:{port}/v2/pipeline"),
            vec![],
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(started.elapsed() >= DATABASE_RESPONSE_TIMEOUT);
        assert!(started.elapsed() < DATABASE_RESPONSE_TIMEOUT + Duration::from_secs(2));
        silent_server.abort();
    }
}
