#[path = "pages/api/auth/callback/github.rs"]
mod pages_api_auth_callback_github;
#[path = "pages/index/mod.rs"]
mod pages_index;
#[allow(non_snake_case)]
#[path = "pages/post/[id]/mod.rs"]
mod pages_post__id_;
#[path = "pages/write.rs"]
mod pages_write;
#[path = "hooks/me.rs"]
mod hooks_me;
use forte_sdk::anyhow::Result;
use forte_sdk::http::{Error, Request, Response, StatusCode, body::Body, HeaderMap};
use forte_sdk::http_header::{COOKIE, LOCATION, SET_COOKIE};
use forte_sdk::*;
use std::collections::HashMap;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[allow(non_camel_case_types)]
pub enum Redirect {
    External { url: String },
    ApiAuthCallbackGithub,
    Index,
    Post_id_ { id: String },
    Write,
}
impl Redirect {
    pub fn to_path(&self) -> String {
        match self {
            Redirect::External { url } => url.clone(),
            Redirect::ApiAuthCallbackGithub => "/api/auth/callback/github".to_string(),
            Redirect::Index => "/".to_string(),
            Redirect::Post_id_ { id } => {
                format!("/{}", ["post".to_string(), id.to_string()].join("/"))
            }
            Redirect::Write => "/write".to_string(),
        }
    }
}
impl std::fmt::Display for Redirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redirect to {}", self.to_path())
    }
}
impl std::error::Error for Redirect {}
#[forte_sdk::wstd::http_server]
pub async fn main(request: Request<Body>) -> Result<Response<Body>, Error> {
    let (parts, body) = request.into_parts();
    let headers = parts.headers;
    let path = parts.uri.path();
    let query = parts.uri.query().unwrap_or("");
    let mut cookie_jar = make_cookie_jar(&headers);
    if let Some(hook_name) = path.strip_prefix("/__forte_hook/") {
        return handle_hook(hook_name, &headers, &mut cookie_jar, body).await;
    }
    let query_params: HashMap<String, String> = query
        .split('&')
        .filter(|s| !s.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((key.to_string(), value.to_string()))
        })
        .collect();
    let path_segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    if path == "/api/auth/callback/github" {
        let Some(code) = query_params.get("code").cloned() else {
            return Ok(
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(
                        Body::from(
                            format!("Missing required query parameter: {}", "code"),
                        ),
                    )
                    .unwrap(),
            );
        };
        let Some(state) = query_params.get("state").cloned() else {
            return Ok(
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(
                        Body::from(
                            format!("Missing required query parameter: {}", "state"),
                        ),
                    )
                    .unwrap(),
            );
        };
        let search_params = pages_api_auth_callback_github::SearchParams {
            code,
            state,
        };
        match pages_api_auth_callback_github::handler(
                headers,
                &mut cookie_jar,
                search_params,
            )
            .await
        {
            Ok(props) => {
                let stream = forte_json::to_stream(&props);
                Ok(
                    build_response_with_cookies(
                        Response::new(Body::from_stream(stream)),
                        &cookie_jar,
                    ),
                )
            }
            Err(e) => {
                if let Some(redirect) = e.downcast_ref::<Redirect>() {
                    Ok(
                        build_response_with_cookies(
                            Response::builder()
                                .status(StatusCode::FOUND)
                                .header(LOCATION, redirect.to_path())
                                .body(Body::empty())
                                .unwrap(),
                            &cookie_jar,
                        ),
                    )
                } else {
                    Ok(
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(format!("Error: {:?}", e)))
                            .unwrap(),
                    )
                }
            }
        }
    } else if path == "/" {
        let after: Option<String> = query_params.get("after").cloned();
        let search_params = pages_index::SearchParams { after };
        match pages_index::handler(headers, &mut cookie_jar, search_params).await {
            Ok(props) => {
                let stream = forte_json::to_stream(&props);
                Ok(
                    build_response_with_cookies(
                        Response::new(Body::from_stream(stream)),
                        &cookie_jar,
                    ),
                )
            }
            Err(e) => {
                if let Some(redirect) = e.downcast_ref::<Redirect>() {
                    Ok(
                        build_response_with_cookies(
                            Response::builder()
                                .status(StatusCode::FOUND)
                                .header(LOCATION, redirect.to_path())
                                .body(Body::empty())
                                .unwrap(),
                            &cookie_jar,
                        ),
                    )
                } else {
                    Ok(
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(format!("Error: {:?}", e)))
                            .unwrap(),
                    )
                }
            }
        }
    } else if path_segments.len() == 2usize && path_segments.first() == Some(&"post") {
        let id: String = path_segments[1usize].to_string();
        let path_params = pages_post__id_::PathParams { id };
        match pages_post__id_::handler(headers, &mut cookie_jar, path_params).await {
            Ok(props) => {
                let stream = forte_json::to_stream(&props);
                Ok(
                    build_response_with_cookies(
                        Response::new(Body::from_stream(stream)),
                        &cookie_jar,
                    ),
                )
            }
            Err(e) => {
                if let Some(redirect) = e.downcast_ref::<Redirect>() {
                    Ok(
                        build_response_with_cookies(
                            Response::builder()
                                .status(StatusCode::FOUND)
                                .header(LOCATION, redirect.to_path())
                                .body(Body::empty())
                                .unwrap(),
                            &cookie_jar,
                        ),
                    )
                } else {
                    Ok(
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(format!("Error: {:?}", e)))
                            .unwrap(),
                    )
                }
            }
        }
    } else if path == "/write" {
        match pages_write::handler(headers, &mut cookie_jar).await {
            Ok(props) => {
                let stream = forte_json::to_stream(&props);
                Ok(
                    build_response_with_cookies(
                        Response::new(Body::from_stream(stream)),
                        &cookie_jar,
                    ),
                )
            }
            Err(e) => {
                if let Some(redirect) = e.downcast_ref::<Redirect>() {
                    Ok(
                        build_response_with_cookies(
                            Response::builder()
                                .status(StatusCode::FOUND)
                                .header(LOCATION, redirect.to_path())
                                .body(Body::empty())
                                .unwrap(),
                            &cookie_jar,
                        ),
                    )
                } else {
                    Ok(
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(Body::from(format!("Error: {:?}", e)))
                            .unwrap(),
                    )
                }
            }
        }
    } else {
        Ok(
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        )
    }
}
async fn handle_hook(
    hook_name: &str,
    headers: &HeaderMap,
    cookie_jar: &mut cookie::CookieJar,
    mut body: Body,
) -> Result<Response<Body>, Error> {
    let Some(host) = headers.get(http_header::HOST).and_then(|v| v.to_str().ok()) else {
        return Ok(
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Missing Host header"))
                .unwrap(),
        );
    };
    let body_bytes = body.contents().await?;
    match hook_name {
        "me" => {
            let input: hooks_me::Input = serde_json::from_slice(body_bytes)
                .map_err(|e| Error::msg(e.to_string()))?;
            let req = HookRequest {
                host,
                headers,
                jar: cookie_jar,
                body: input,
            };
            let output = hooks_me::handler(req);
            let json = forte_json::to_vec(&output);
            Ok(
                build_response_with_cookies(
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "application/json")
                        .body(Body::from(json))
                        .unwrap(),
                    cookie_jar,
                ),
            )
        }
        _ => {
            Ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from(format!("Hook '{}' not found", hook_name)))
                    .unwrap(),
            )
        }
    }
}
fn make_cookie_jar(headers: &HeaderMap) -> cookie::CookieJar {
    let mut jar = cookie::CookieJar::new();
    let Some(cookie) = headers.get(COOKIE) else {
        return jar;
    };
    let Ok(cookie_str) = cookie.to_str() else {
        return jar;
    };
    for cookie in cookie::Cookie::split_parse_encoded(cookie_str) {
        let Ok(cookie) = cookie else { continue };
        jar.add_original(cookie.into_owned());
    }
    jar
}
fn build_response_with_cookies(
    mut response: Response<Body>,
    cookie_jar: &cookie::CookieJar,
) -> Response<Body> {
    for cookie in cookie_jar.delta() {
        if let Ok(value) = cookie.encoded().to_string().parse() {
            response.headers_mut().append(SET_COOKIE, value);
        }
    }
    response
}
