use anyhow::Result;
use std::time::Duration;

const DATABASE_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) async fn http_post_doc_db(url: &str, body: Vec<u8>) -> Result<(u16, Vec<u8>)> {
    #[cfg(target_arch = "wasm32")]
    {
        use forte_sdk::http::{Client, HeaderValue, Request, RequestTimeouts, Uri};
        use std::str::FromStr;

        let uri = Uri::from_str(url).map_err(|e| anyhow::anyhow!("Invalid URI: {e}"))?;
        let request = Request::post(&uri)
            .header(
                "Content-Type",
                HeaderValue::from_static(doc_db_protocol::CONTENT_TYPE),
            )
            .body(body)
            .map_err(|e| anyhow::anyhow!("Failed to build request: {e}"))?;
        let client = Client::new().with_timeouts(RequestTimeouts::all(DATABASE_RESPONSE_TIMEOUT));
        let response = client.send(request).await?;
        let status = response.status().as_u16();
        let bytes = response
            .into_body()
            .bytes_limited(doc_db_protocol::MAX_FRAME_SIZE)
            .await?;
        Ok((status, bytes.to_vec()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::OnceLock;

        static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
        let client = CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .connect_timeout(DATABASE_RESPONSE_TIMEOUT)
                .read_timeout(DATABASE_RESPONSE_TIMEOUT)
                .build()
                .expect("database http client must build")
        });
        let response = client
            .post(url)
            .header("Content-Type", doc_db_protocol::CONTENT_TYPE)
            .body(body)
            .send()
            .await?;
        let status = response.status().as_u16();
        if response
            .content_length()
            .is_some_and(|length| length > doc_db_protocol::MAX_FRAME_SIZE as u64)
        {
            anyhow::bail!("doc-db response exceeds the semantic RPC frame limit");
        }
        let mut response_body = Vec::new();
        let mut response = response;
        while let Some(chunk) = response.chunk().await? {
            if chunk.len() > doc_db_protocol::MAX_FRAME_SIZE.saturating_sub(response_body.len()) {
                anyhow::bail!("doc-db response exceeds the semantic RPC frame limit");
            }
            response_body.extend_from_slice(&chunk);
        }
        Ok((status, response_body))
    }
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
