use crate::Fn0;
use adapt_cache::AdaptCache;
use bytes::Bytes;
use hyper::HeaderMap;
use hyper::http;
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use ski::{FetchHandler, FetchHandlerFuture};
use std::string::FromUtf8Error;
use std::sync::{Arc, Mutex};

pub(crate) struct ForteFetchHandler<J>
where
    J: AdaptCache<String, FromUtf8Error> + 'static,
{
    fn0: Arc<Fn0<J>>,
    code_id: String,
    original_headers: HeaderMap,
    collected_cookies: Arc<Mutex<Vec<String>>>,
}

impl<J> ForteFetchHandler<J>
where
    J: AdaptCache<String, FromUtf8Error> + 'static,
{
    pub(crate) fn new(fn0: Arc<Fn0<J>>, code_id: String, original_headers: HeaderMap) -> Self {
        Self {
            fn0,
            code_id,
            original_headers,
            collected_cookies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn get_collected_cookies(&self) -> Vec<String> {
        self.collected_cookies.lock().unwrap().clone()
    }
}

impl<J> FetchHandler for ForteFetchHandler<J>
where
    J: AdaptCache<String, FromUtf8Error> + Send + Sync + 'static,
{
    fn handle(&self, req: crate::Request) -> FetchHandlerFuture {
        let path = req.uri().path().to_string();

        if !path.starts_with("/__forte_hook/") {
            return Box::pin(async { None });
        }

        let fn0 = self.fn0.clone();
        let code_id = self.code_id.clone();
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
            let response = fn0.run_forte_backend(&code_id, "", req, None).await;

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
