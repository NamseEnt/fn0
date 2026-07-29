//! HTTP transport abstraction: WASI HTTP inside WASM components, `reqwest`
//! everywhere else. The two `send` implementations share one request/response
//! shape so the rest of the crate is target-agnostic.

use crate::body::Body;
use crate::{Error, Result};

pub(crate) struct Request {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Body>,
}

pub(crate) struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Body,
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn send(req: Request) -> Result<Response> {
    use crate::body::Inner;
    use forte_sdk::http::{Body as SdkBody, Client, Method, Request as HttpRequest};

    let method =
        Method::from_bytes(req.method.as_bytes()).map_err(|e| Error::Transport(e.to_string()))?;
    let mut builder = HttpRequest::builder().method(method).uri(&req.url);
    for (name, value) in &req.headers {
        builder = builder.header(name, value);
    }
    let body = match req.body {
        None => SdkBody::Empty,
        Some(Body(Inner::Bytes(bytes))) => SdkBody::Bytes(bytes.to_vec()),
        Some(Body(Inner::Stream(stream))) => stream,
    };
    let request = builder
        .body(body)
        .map_err(|e| Error::Transport(e.to_string()))?;

    let response = Client::new()
        .send(request)
        .await
        .map_err(|e| Error::Transport(e.to_string()))?;

    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();
    Ok(Response {
        status,
        headers,
        body: Body::from(response.into_body()),
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn send(req: Request) -> Result<Response> {
    use std::sync::OnceLock;

    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    let client = CLIENT.get_or_init(reqwest::Client::new);

    let method = reqwest::Method::from_bytes(req.method.as_bytes())
        .map_err(|e| Error::Transport(e.to_string()))?;
    let mut builder = client.request(method, &req.url);
    for (name, value) in &req.headers {
        builder = builder.header(name, value);
    }
    if let Some(body) = req.body {
        builder = builder.body(reqwest::Body::from(body));
    }

    let response = builder
        .send()
        .await
        .map_err(|e| Error::Transport(e.to_string()))?;

    let status = response.status().as_u16();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_string(),
                String::from_utf8_lossy(value.as_bytes()).into_owned(),
            )
        })
        .collect();
    Ok(Response {
        status,
        headers,
        body: Body(crate::body::Inner::Stream(response)),
    })
}
