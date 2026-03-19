use bytes::Bytes;
use fn0::{FetchHandler, FetchHandlerFuture, Fn0};
use http::HeaderMap;
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use std::sync::{Arc, Mutex};

use super::SimpleCache;

pub struct ForteFetchHandler {
    fn0: Arc<Fn0<SimpleCache>>,
    original_headers: HeaderMap,
    collected_cookies: Arc<Mutex<Vec<String>>>,
}

impl ForteFetchHandler {
    pub fn new(fn0: Arc<Fn0<SimpleCache>>, original_headers: HeaderMap) -> Self {
        Self {
            fn0,
            original_headers,
            collected_cookies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_collected_cookies(&self) -> Vec<String> {
        self.collected_cookies.lock().unwrap().clone()
    }
}

impl FetchHandler for ForteFetchHandler {
    fn handle(&self, req: fn0::Request) -> FetchHandlerFuture {
        let path = req.uri().path().to_string();

        if !path.starts_with("/__forte_hook/") {
            return Box::pin(async { None });
        }

        let fn0 = self.fn0.clone();
        let original_headers = self.original_headers.clone();
        let collected_cookies = self.collected_cookies.clone();

        Box::pin(async move {
            let (mut parts, body) = req.into_parts();

            for (key, value) in &original_headers {
                if key == http::header::HOST {
                    continue;
                }
                if !parts.headers.contains_key(key) {
                    parts.headers.insert(key.clone(), value.clone());
                }
            }

            let path_and_query = parts
                .uri
                .path_and_query()
                .map(|pq| pq.as_str())
                .unwrap_or("/");

            let Some(host) = original_headers
                .get(http::header::HOST)
                .and_then(|v| v.to_str().ok())
            else {
                let body: UnsyncBoxBody<Bytes, anyhow::Error> =
                    Full::new(Bytes::from("Missing Host header in original request"))
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .boxed_unsync();
                return Some(
                    hyper::Response::builder()
                        .status(400)
                        .body(body)
                        .expect("failed to build response"),
                );
            };

            let new_uri = format!("http://{}{}", host, path_and_query);
            let Ok(uri) = new_uri.parse() else {
                let body: UnsyncBoxBody<Bytes, anyhow::Error> =
                    Full::new(Bytes::from("Invalid URI"))
                        .map_err(|e| anyhow::anyhow!("{e}"))
                        .boxed_unsync();
                return Some(
                    hyper::Response::builder()
                        .status(400)
                        .body(body)
                        .expect("failed to build response"),
                );
            };
            parts.uri = uri;

            let req = hyper::Request::from_parts(parts, body);
            let response = fn0.run("backend", "", req, None).await;

            match response {
                Ok(resp) => {
                    for value in resp.headers().get_all(http::header::SET_COOKIE) {
                        if let Ok(s) = value.to_str() {
                            collected_cookies.lock().unwrap().push(s.to_string());
                        }
                    }
                    Some(resp)
                }
                Err(e) => {
                    eprintln!("[ForteFetchHandler] Error calling hook: {:?}", e);
                    let body: UnsyncBoxBody<Bytes, anyhow::Error> =
                        Full::new(Bytes::from(format!("Hook error: {}", e)))
                            .map_err(|e| anyhow::anyhow!("{e}"))
                            .boxed_unsync();
                    Some(
                        hyper::Response::builder()
                            .status(500)
                            .header("content-type", "text/plain")
                            .body(body)
                            .unwrap(),
                    )
                }
            }
        })
    }
}
