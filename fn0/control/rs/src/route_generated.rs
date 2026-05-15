#[path = "pages/index/mod.rs"]
mod pages_index;
#[path = "pages/login/mod.rs"]
mod pages_login;
#[path = "pages/oauth/github/callback/mod.rs"]
mod pages_oauth_github_callback;
#[path = "pages/tokens/mod.rs"]
mod pages_tokens;
use forte_sdk::anyhow::Result;
use forte_sdk::http::{HeaderMap, Request, Response, StatusCode, body::Body};
use forte_sdk::http_header::{COOKIE, LOCATION, SET_COOKIE};
use forte_sdk::*;
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[allow(non_camel_case_types)]
pub enum Redirect {
    External { url: String },
    OauthGithubCallback,
    Index,
    Tokens,
    Login,
}
impl Redirect {
    pub fn to_path(&self) -> String {
        match self {
            Redirect::External { url } => url.clone(),
            Redirect::OauthGithubCallback => "/oauth/github/callback".to_string(),
            Redirect::Index => "/".to_string(),
            Redirect::Tokens => "/tokens".to_string(),
            Redirect::Login => "/login".to_string(),
        }
    }
}
impl std::fmt::Display for Redirect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Redirect to {}", self.to_path())
    }
}
impl std::error::Error for Redirect {}
pub mod enqueue {
    pub async fn cloudflare_register(
        input: crate::queue_task::cloudflare_register::Input,
    ) -> forte_sdk::anyhow::Result<()> {
        let payload = forte_sdk::serde_json::to_value(&input)?;
        let body = forte_sdk::serde_json::to_vec(&forte_sdk::serde_json::json!(
            { "task_name" : "cloudflare_register", "payload" : payload, }
        ))?;
        let url = std::env::var("FN0_QUEUE_URL")
            .map_err(|_| forte_sdk::anyhow::anyhow!("FN0_QUEUE_URL is not set"))?;
        let request = forte_sdk::http::Request::post(&url)
            .header("Content-Type", "application/json")
            .body(body)?;
        let response = forte_sdk::http::Client::new().send(request).await?;
        if !response.status().is_success() {
            return Err(forte_sdk::anyhow::anyhow!(
                "enqueue failed with status {}",
                response.status()
            ));
        }
        Ok(())
    }
    pub async fn cloudflare_unregister(
        input: crate::queue_task::cloudflare_unregister::Input,
    ) -> forte_sdk::anyhow::Result<()> {
        let payload = forte_sdk::serde_json::to_value(&input)?;
        let body = forte_sdk::serde_json::to_vec(&forte_sdk::serde_json::json!(
            { "task_name" : "cloudflare_unregister", "payload" : payload, }
        ))?;
        let url = std::env::var("FN0_QUEUE_URL")
            .map_err(|_| forte_sdk::anyhow::anyhow!("FN0_QUEUE_URL is not set"))?;
        let request = forte_sdk::http::Request::post(&url)
            .header("Content-Type", "application/json")
            .body(body)?;
        let response = forte_sdk::http::Client::new().send(request).await?;
        if !response.status().is_success() {
            return Err(forte_sdk::anyhow::anyhow!(
                "enqueue failed with status {}",
                response.status()
            ));
        }
        Ok(())
    }
}
#[allow(clippy::crate_in_macro_def)]
mod proxy {
    forte_sdk::wit_bindgen::generate!(
        { inline :
        "package forte:user; world service-export { import wasi:http/types@0.3.0-rc-2026-03-15; export wasi:http/handler@0.3.0-rc-2026-03-15; }",
        path :
        "/Users/namse/fn0/fn0/control/rs/target/wasm32-wasip2/release/build/fn0-control-f6e9b948ff726a5a/out/forte-wit",
        world : "service-export", default_bindings_module :
        "crate::route_generated::proxy", pub_export_macro : true, async : true, features
        : ["clocks-timezone"], with : { "wasi:http/handler@0.3.0-rc-2026-03-15" :
        generate, "wasi:http/types@0.3.0-rc-2026-03-15" :
        forte_sdk::bindings::wasi::http::types, "wasi:clocks/types@0.3.0-rc-2026-03-15" :
        forte_sdk::bindings::wasi::clocks::types, }, runtime_path :
        "forte_sdk::wit_bindgen::rt", }
    );
}
struct Server;
impl proxy::exports::wasi::http::handler::Guest for Server {
    async fn handle(
        req: forte_sdk::bindings::wasi::http::types::Request,
    ) -> core::result::Result<
        forte_sdk::bindings::wasi::http::types::Response,
        forte_sdk::bindings::wasi::http::types::ErrorCode,
    > {
        forte_sdk::serve::serve(req, |request| async move { dispatch(request).await }).await
    }
}
proxy::export!(Server);
async fn dispatch(request: Request<Vec<u8>>) -> Result<Response<Body>> {
    let path_for_route = request.uri().path().to_string();
    let key = classify_route(&path_for_route);
    let result = dispatch_inner(request).await;
    result.map(|mut resp| {
        if let Ok(v) = forte_sdk::http::HeaderValue::from_str(&key) {
            resp.headers_mut()
                .insert("x-fn0-execution-time-metric-key", v);
        }
        resp
    })
}
fn classify_route(path: &str) -> String {
    if path.strip_prefix("/__forte_action/").is_some() {
        return "/__forte_action/[name]".to_string();
    }
    if path.strip_prefix("/__forte_admin/").is_some() {
        return "/__forte_admin/[name]".to_string();
    }
    if path == "/__fn0_queue_task/execute" {
        return "/__fn0_queue_task/execute".to_string();
    }
    if path.strip_prefix("/__self_invoke/").is_some() {
        return "/__self_invoke/[name]".to_string();
    }
    if path == "/oauth/github/callback" {
        return "/oauth/github/callback".to_string();
    }
    if path == "/" {
        return "/".to_string();
    }
    if path == "/tokens" {
        return "/tokens".to_string();
    }
    if path == "/login" {
        return "/login".to_string();
    }
    "unknown".to_string()
}
async fn dispatch_inner(request: Request<Vec<u8>>) -> Result<Response<Body>> {
    let (parts, body_bytes) = request.into_parts();
    let headers = parts.headers;
    let path = parts.uri.path().to_string();
    let method = parts.method;
    let mut cookie_jar = make_cookie_jar(&headers);
    let Some(uri_authority) = parts.uri.authority() else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Missing authority in request URI"))
            .unwrap());
    };
    let uri_authority = uri_authority.as_str();
    let path = path.as_str();
    if let Some(hook_name) = path.strip_prefix("/__self_invoke/") {
        return handle_hook(
            hook_name,
            uri_authority,
            &method,
            &headers,
            &mut cookie_jar,
            &body_bytes,
        )
        .await;
    }
    if let Some(action_name) = path.strip_prefix("/__forte_action/") {
        return handle_action(
            action_name,
            uri_authority,
            &method,
            &headers,
            &mut cookie_jar,
            &body_bytes,
        )
        .await;
    }
    if path == "/__fn0_queue_task/execute" {
        return handle_queue_task_execute(&body_bytes).await;
    }
    if let Some(task_name) = path.strip_prefix("/__forte_admin/") {
        return handle_admin_task(task_name, &headers, &body_bytes).await;
    }
    if path == "/oauth/github/callback" {
        use std::collections::HashMap;
        let query = parts.uri.query().unwrap_or("");
        let query_params: HashMap<String, String> =
            forte_sdk::form_urlencoded::parse(query.as_bytes())
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();
        let Some(code) = query_params.get("code").cloned() else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!(
                    "Missing required query parameter: {}",
                    "code"
                )))
                .unwrap());
        };
        let Some(state) = query_params.get("state").cloned() else {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!(
                    "Missing required query parameter: {}",
                    "state"
                )))
                .unwrap());
        };
        let search_params = pages_oauth_github_callback::SearchParams { code, state };
        let req = ForteRequest {
            uri_authority,
            method: &method,
            headers: &headers,
            jar: &mut cookie_jar,
            raw_body: &body_bytes,
            body: (),
        };
        match pages_oauth_github_callback::handler(req, search_params).await {
            Ok(redirect) => Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::FOUND)
                    .header(LOCATION, redirect.to_path())
                    .body(Body::empty())
                    .unwrap(),
                &cookie_jar,
            )),
            Err(e) => {
                eprintln!("Error at {}: {:?}", path, e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Internal Server Error"))
                    .unwrap())
            }
        }
    } else if path == "/" {
        let req = ForteRequest {
            uri_authority,
            method: &method,
            headers: &headers,
            jar: &mut cookie_jar,
            raw_body: &body_bytes,
            body: (),
        };
        match pages_index::handler(req).await {
            Ok(props) => {
                let body_bytes = forte_json::to_vec(&props);
                Ok(build_response_with_cookies(
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("x-fn0-next", "js")
                        .body(Body::from(body_bytes))
                        .unwrap(),
                    &cookie_jar,
                ))
            }
            Err(e) => {
                if let Some(redirect) = e.downcast_ref::<Redirect>() {
                    Ok(build_response_with_cookies(
                        Response::builder()
                            .status(StatusCode::FOUND)
                            .header(LOCATION, redirect.to_path())
                            .body(Body::empty())
                            .unwrap(),
                        &cookie_jar,
                    ))
                } else {
                    eprintln!("Error at {}: {:?}", path, e);
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap())
                }
            }
        }
    } else if path == "/tokens" {
        let req = ForteRequest {
            uri_authority,
            method: &method,
            headers: &headers,
            jar: &mut cookie_jar,
            raw_body: &body_bytes,
            body: (),
        };
        match pages_tokens::handler(req).await {
            Ok(props) => {
                let body_bytes = forte_json::to_vec(&props);
                Ok(build_response_with_cookies(
                    Response::builder()
                        .status(StatusCode::OK)
                        .header("x-fn0-next", "js")
                        .body(Body::from(body_bytes))
                        .unwrap(),
                    &cookie_jar,
                ))
            }
            Err(e) => {
                if let Some(redirect) = e.downcast_ref::<Redirect>() {
                    Ok(build_response_with_cookies(
                        Response::builder()
                            .status(StatusCode::FOUND)
                            .header(LOCATION, redirect.to_path())
                            .body(Body::empty())
                            .unwrap(),
                        &cookie_jar,
                    ))
                } else {
                    eprintln!("Error at {}: {:?}", path, e);
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal Server Error"))
                        .unwrap())
                }
            }
        }
    } else if path == "/login" {
        let req = ForteRequest {
            uri_authority,
            method: &method,
            headers: &headers,
            jar: &mut cookie_jar,
            raw_body: &body_bytes,
            body: (),
        };
        match pages_login::handler(req).await {
            Ok(redirect) => Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::FOUND)
                    .header(LOCATION, redirect.to_path())
                    .body(Body::empty())
                    .unwrap(),
                &cookie_jar,
            )),
            Err(e) => {
                eprintln!("Error at {}: {:?}", path, e);
                Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Internal Server Error"))
                    .unwrap())
            }
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap())
    }
}
async fn handle_hook(
    hook_name: &str,
    _uri_authority: &str,
    _method: &http::Method,
    _headers: &HeaderMap,
    _cookie_jar: &mut cookie::CookieJar,
    _body_bytes: &[u8],
) -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from(format!("Hook '{}' not found", hook_name)))
        .unwrap())
}
#[forte_sdk::tracing::instrument(
    name = "handle_action",
    skip_all,
    fields(action = action_name),
)]
async fn handle_action(
    action_name: &str,
    uri_authority: &str,
    method: &http::Method,
    headers: &HeaderMap,
    cookie_jar: &mut cookie::CookieJar,
    body_bytes: &[u8],
) -> Result<Response<Body>> {
    match action_name {
        "set_pending_fn0_wasmtime" => {
            let input: crate::actions::set_pending_fn0_wasmtime::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::set_pending_fn0_wasmtime::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "revoke_token" => {
            let input: crate::actions::revoke_token::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::revoke_token::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "domain_remove" => {
            let input: crate::actions::domain_remove::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::domain_remove::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "promote_pending_fn0_wasmtime" => {
            let input: crate::actions::promote_pending_fn0_wasmtime::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::promote_pending_fn0_wasmtime::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "deploy" => {
            let input: crate::actions::deploy::Input = match forte_json::from_slice(body_bytes) {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(format!("invalid request body: {}", e)))
                        .unwrap());
                }
            };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::deploy::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "zombie_sweep" => {
            let input: crate::actions::zombie_sweep::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::zombie_sweep::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "secrets_init" => {
            let input: crate::actions::secrets_init::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::secrets_init::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "new_project" => {
            let input: crate::actions::new_project::Input = match forte_json::from_slice(body_bytes)
            {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(format!("invalid request body: {}", e)))
                        .unwrap());
                }
            };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::new_project::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "rename_project" => {
            let input: crate::actions::rename_project::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::rename_project::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "bundle_compiled" => {
            let input: crate::actions::bundle_compiled::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::bundle_compiled::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "secrets_encrypt" => {
            let input: crate::actions::secrets_encrypt::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::secrets_encrypt::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "list_tokens" => {
            let input: crate::actions::list_tokens::Input = match forte_json::from_slice(body_bytes)
            {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(format!("invalid request body: {}", e)))
                        .unwrap());
                }
            };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::list_tokens::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "issue_token" => {
            let input: crate::actions::issue_token::Input = match forte_json::from_slice(body_bytes)
            {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(format!("invalid request body: {}", e)))
                        .unwrap());
                }
            };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::issue_token::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "cron_on_tick" => {
            let input: crate::actions::cron_on_tick::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::cron_on_tick::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "domain_status" => {
            let input: crate::actions::domain_status::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::domain_status::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "bundle_uploaded" => {
            let input: crate::actions::bundle_uploaded::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::bundle_uploaded::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "domain_add" => {
            let input: crate::actions::domain_add::Input = match forte_json::from_slice(body_bytes)
            {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(format!("invalid request body: {}", e)))
                        .unwrap());
                }
            };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::domain_add::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        "deploy_status" => {
            let input: crate::actions::deploy_status::Input =
                match forte_json::from_slice(body_bytes) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!("invalid request body: {}", e)))
                            .unwrap());
                    }
                };
            let req = ForteRequest {
                uri_authority,
                method,
                headers,
                jar: cookie_jar,
                raw_body: body_bytes,
                body: input,
            };
            let output = crate::actions::deploy_status::handler(req).await;
            let json = forte_json::to_vec(&output);
            Ok(build_response_with_cookies(
                Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap(),
                cookie_jar,
            ))
        }
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from(format!("Action '{}' not found", action_name)))
            .unwrap()),
    }
}
async fn handle_queue_task_execute(body_bytes: &[u8]) -> Result<Response<Body>> {
    let request: forte_sdk::serde_json::Value = match forte_sdk::serde_json::from_slice(body_bytes)
    {
        Ok(v) => v,
        Err(e) => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("invalid request body: {}", e)))
                .unwrap());
        }
    };
    let task_name = match request["task_name"].as_str() {
        Some(s) => s,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("missing or non-string task_name"))
                .unwrap());
        }
    };
    let payload = match request["payload"].as_str() {
        Some(s) => s,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("missing or non-string payload"))
                .unwrap());
        }
    };
    let result = match task_name {
        "cloudflare_register" => {
            let input: crate::queue_task::cloudflare_register::Input =
                match forte_sdk::serde_json::from_str(payload) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!(
                                "invalid payload for task '{}': {}",
                                task_name, e
                            )))
                            .unwrap());
                    }
                };
            crate::queue_task::cloudflare_register::handle(input).await
        }
        "cloudflare_unregister" => {
            let input: crate::queue_task::cloudflare_unregister::Input =
                match forte_sdk::serde_json::from_str(payload) {
                    Ok(v) => v,
                    Err(e) => {
                        return Ok(Response::builder()
                            .status(StatusCode::BAD_REQUEST)
                            .body(Body::from(format!(
                                "invalid payload for task '{}': {}",
                                task_name, e
                            )))
                            .unwrap());
                    }
                };
            crate::queue_task::cloudflare_unregister::handle(input).await
        }
        _ => {
            return Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("Queue task '{}' not found", task_name)))
                .unwrap());
        }
    };
    match result {
        Ok(()) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(Body::empty())
            .unwrap()),
        Err(e) => Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("{:?}", e)))
            .unwrap()),
    }
}
async fn handle_admin_task(
    _task_name: &str,
    _headers: &HeaderMap,
    _body_bytes: &[u8],
) -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("No admin tasks defined"))
        .unwrap())
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
