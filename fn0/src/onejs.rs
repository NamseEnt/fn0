// onejs.rs - JavaScript runtime for fn0 using deno_core (WinterCG only)
use anyhow::{Context, Result, anyhow};
use bytes::Bytes;
use deno_core::{JsRuntime, RuntimeOptions, url::Url, v8};
use http::{HeaderMap, HeaderName, HeaderValue};
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body::Body};
use std::borrow::Cow;
use std::error::Error as StdError;
use std::path::Path;
use std::sync::Arc;

pub struct Config<'a> {
    pub js_code: &'a str,
}

pub struct Fn0OneJs {
    module_code: String,
}

// Simple allow-all permission for WinterCG APIs
struct AllowAllPermissions;

impl deno_fetch::FetchPermissions for AllowAllPermissions {
    fn check_net_url(
        &mut self,
        _url: &Url,
        _api_name: &str,
    ) -> Result<(), deno_permissions::PermissionCheckError> {
        Ok(())
    }

    fn check_read<'a>(
        &mut self,
        path: &'a Path,
        _api_name: &str,
    ) -> Result<Cow<'a, Path>, deno_permissions::PermissionCheckError> {
        Ok(Cow::Borrowed(path))
    }
}

impl deno_web::TimersPermission for AllowAllPermissions {
    fn allow_hrtime(&mut self) -> bool {
        true
    }
}

impl deno_websocket::WebSocketPermissions for AllowAllPermissions {
    fn check_net_url(
        &mut self,
        _url: &Url,
        _api_name: &str,
    ) -> Result<(), deno_permissions::PermissionCheckError> {
        Ok(())
    }
}

impl deno_net::NetPermissions for AllowAllPermissions {
    fn check_net<T: AsRef<str>>(
        &mut self,
        _host: &(T, Option<u16>),
        _api_name: &str,
    ) -> Result<(), deno_permissions::PermissionCheckError> {
        Ok(())
    }

    fn check_read(
        &mut self,
        _path: &str,
        _api_name: &str,
    ) -> Result<std::path::PathBuf, deno_permissions::PermissionCheckError> {
        Ok(std::path::PathBuf::new())
    }

    fn check_write(
        &mut self,
        _path: &str,
        _api_name: &str,
    ) -> Result<std::path::PathBuf, deno_permissions::PermissionCheckError> {
        Ok(std::path::PathBuf::new())
    }

    fn check_write_path<'a>(
        &mut self,
        _path: &'a std::path::Path,
        _api_name: &str,
    ) -> Result<Cow<'a, std::path::Path>, deno_permissions::PermissionCheckError> {
        Ok(Cow::Borrowed(std::path::Path::new("")))
    }
}

impl Fn0OneJs {
    pub async fn new(config: Config<'_>) -> Result<Self> {
        let module_code = config.js_code.to_string();

        // Validate that code contains "handle" function
        if !module_code.contains("handle") {
            return Err(anyhow!("Module must export a 'handle' function"));
        }

        Ok(Self { module_code })
    }

    pub async fn run<B>(&self, req: Request<B>) -> Result<Response<Full<Bytes>>>
    where
        B: Body<Data = Bytes> + Unpin + Send + 'static,
        B::Error: StdError + Send + Sync,
    {
        // Create runtime with WinterCG APIs only
        let mut runtime = create_runtime().context("Failed to create runtime")?;

        let bootstrap_url = Url::parse("ext:onejs/bootstrap.js")?;

        let mod_id = runtime
            .load_side_es_module(&bootstrap_url)
            .await
            .context("Failed to load bootstrap module")?;

        let evaluation = runtime.mod_evaluate(mod_id);

        // 모듈의 import들이 처리되도록 이벤트 루프를 실행합니다.
        runtime
            .run_event_loop(deno_core::PollEventLoopOptions::default())
            .await
            .context("Event loop error during bootstrap")?;

        evaluation.await.context("Bootstrap evaluation failed")?;

        todo!();

        // Execute bootstrap.js via import statement
        runtime
            .execute_script(
                "<setup>",
                deno_core::ModuleCodeString::from(
                    r#"import "ext:onejs/bootstrap.js";"#.to_string(),
                ),
            )
            .context("Failed to execute bootstrap")?;

        // Run event loop to complete ESM loading
        runtime
            .run_event_loop(deno_core::PollEventLoopOptions::default())
            .await
            .context("Event loop error during bootstrap")?;

        // Extract request components
        let (parts, body) = req.into_parts();
        let method = parts.method.as_str();
        let uri = parts.uri.to_string();

        // Collect body
        let body_bytes = body
            .collect()
            .await
            .map_err(|e| anyhow::anyhow!("Body collect error: {}", e))?
            .to_bytes();
        let body_str = String::from_utf8_lossy(&body_bytes);

        // Build headers as JS array
        let mut headers_vec = vec![];
        for (name, value) in parts.headers.iter() {
            if let Ok(value_str) = value.to_str() {
                headers_vec.push(format!(r#"["{}", "{}"]"#, name.as_str(), value_str));
            }
        }
        let headers_str = headers_vec.join(", ");

        // Execute the user code (assumes it's already bundled by rolldown)
        // Note: bootstrap.js has been imported above, so Request/Response are available
        // User code should have: async function handle(req) { ... }
        let user_script = format!(
            r#"
            {user_code}

            // Create Request from hyper request
            const headersArray = [{headers_str}];
            const req = new Request("{uri}", {{
                method: "{method}",
                headers: headersArray,
                body: "{body_str}"
            }});

            // Call the handle function and extract response data
            globalThis.__fn0_status = null;
            globalThis.__fn0_headers = null;
            globalThis.__fn0_body = null;
            globalThis.__fn0_error = null;

            handle(req)
                .then(async (resp) => {{
                    globalThis.__fn0_status = resp.status;
                    globalThis.__fn0_headers = Array.from(resp.headers.entries());
                    globalThis.__fn0_body = await resp.text();
                }})
                .catch((err) => {{
                    globalThis.__fn0_error = err.toString();
                }});
            "#,
            user_code = &self.module_code,
            headers_str = headers_str,
            uri = uri,
            method = method,
            body_str = body_str
        );

        runtime
            .execute_script("<user>", deno_core::ModuleCodeString::from(user_script))
            .context("Failed to execute user code")?;

        // Run event loop to completion
        runtime
            .run_event_loop(deno_core::PollEventLoopOptions::default())
            .await
            .context("Event loop error")?;

        // Get values first, then use scope
        let error_value = runtime.execute_script(
            "<anon>",
            deno_core::ModuleCodeString::from(String::from("globalThis.__fn0_error")),
        )?;

        let status_value = runtime.execute_script(
            "<anon>",
            deno_core::ModuleCodeString::from(String::from("globalThis.__fn0_status")),
        )?;

        let headers_value = runtime.execute_script(
            "<anon>",
            deno_core::ModuleCodeString::from(String::from("globalThis.__fn0_headers")),
        )?;

        let body_value = runtime.execute_script(
            "<anon>",
            deno_core::ModuleCodeString::from(String::from("globalThis.__fn0_body")),
        )?;

        // Now use scope to extract values
        let (status, headers, body) = {
            let scope = &mut runtime.handle_scope();
            let context = scope.get_current_context();
            let mut scope = v8::ContextScope::new(scope, context);

            // Check error
            let error_local = v8::Local::new(&mut scope, &error_value);
            if !error_local.is_undefined() && !error_local.is_null() {
                let error_str = error_local
                    .to_string(&mut scope)
                    .unwrap()
                    .to_rust_string_lossy(&mut scope);
                return Err(anyhow!("JavaScript error: {}", error_str));
            }

            // Extract status
            let status_local = v8::Local::new(&mut scope, &status_value);
            if status_local.is_undefined() || status_local.is_null() {
                return Err(anyhow!("No status returned from handle()"));
            }
            let status = status_local
                .to_uint32(&mut scope)
                .ok_or_else(|| anyhow!("Status is not a number"))?
                .value() as u16;

            // Extract headers - array of [name, value] pairs
            let mut headers_map = HeaderMap::new();
            let headers_local = v8::Local::new(&mut scope, &headers_value);
            if !headers_local.is_undefined()
                && !headers_local.is_null()
                && let Some(headers_array) = headers_local.to_object(&mut scope)
            {
                // Get array length
                let key = v8::String::new_external_onebyte_static(&mut scope, b"length").unwrap();
                if let Some(length_val) = headers_array.get(&mut scope, key.into())
                    && let Some(length) = length_val.to_uint32(&mut scope)
                {
                    let len = length.value() as usize;
                    for i in 0..len {
                        // Get each element
                        let idx = v8::Number::new(&mut scope, i as f64);
                        if let Some(elem_val) = headers_array.get(&mut scope, idx.into())
                            && let Some(elem) = elem_val.to_object(&mut scope)
                        {
                            // Get elem[0] (name) and elem[1] (value)
                            let zero = v8::Number::new(&mut scope, 0.0);
                            let one = v8::Number::new(&mut scope, 1.0);
                            if let Some(name_val) = elem.get(&mut scope, zero.into())
                                && let Some(name_str) = name_val.to_string(&mut scope)
                            {
                                let name = name_str.to_rust_string_lossy(&mut scope);
                                if let Some(val_val) = elem.get(&mut scope, one.into())
                                    && let Some(val_str) = val_val.to_string(&mut scope)
                                {
                                    let value = val_str.to_rust_string_lossy(&mut scope);
                                    // Parse header name and value
                                    if let Ok(name) = HeaderName::from_bytes(name.as_bytes())
                                        && let Ok(value) = HeaderValue::from_str(&value)
                                    {
                                        headers_map.append(name, value);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Extract body
            let body_local = v8::Local::new(&mut scope, &body_value);
            let body = if !body_local.is_undefined() && !body_local.is_null() {
                let body_str = body_local
                    .to_string(&mut scope)
                    .unwrap()
                    .to_rust_string_lossy(&mut scope);
                Bytes::from(body_str)
            } else {
                Bytes::new()
            };

            (status, headers_map, body)
        };

        // Build hyper response
        let mut builder = Response::builder().status(status);
        if let Some(headers_mut) = builder.headers_mut() {
            *headers_mut = headers;
        }
        Ok(builder.body(Full::new(body)).unwrap())
    }
}

fn create_runtime() -> Result<JsRuntime> {
    use deno_core::v8::CreateParams;
    use deno_web::BlobStore;

    let extensions = vec![
        // Console API
        deno_console::deno_console::init_ops(),
        // WebIDL
        deno_webidl::deno_webidl::init_ops(),
        // URL API
        deno_url::deno_url::init_ops(),
        // Web API (TextEncoder/Decoder, Streams, Blob, etc.)
        deno_web::deno_web::init_ops::<AllowAllPermissions>(Arc::new(BlobStore::default()), None),
        // Fetch API (Request, Response, fetch)
        deno_fetch::deno_fetch::init_ops::<AllowAllPermissions>(deno_fetch::Options {
            user_agent: "fn0/onejs".to_string(),
            ..Default::default()
        }),
        // Net (required for HTTP)
        deno_net::deno_net::init_ops::<AllowAllPermissions>(None, None),
        // TLS (required for HTTPS)
        deno_tls::deno_tls::init_ops(),
        // WebSocket (required by deno_http internally)
        deno_websocket::deno_websocket::init_ops::<AllowAllPermissions>(
            "fn0/onejs".to_string(),
            None,
            None,
        ),
        // HTTP
        deno_http::deno_http::init_ops::<deno_http::DefaultHttpPropertyExtractor>(
            deno_http::Options::default(),
        ),
        // Web Crypto API
        deno_crypto::deno_crypto::init_ops(None),
        // Our custom bootstrap
        crate::onejs_ops::onejs::init_ops(),
    ];

    // Create StaticModuleLoader with bootstrap.js
    let bootstrap_specifier =
        Url::parse("ext:onejs/bootstrap.js").context("Failed to parse bootstrap specifier")?;
    let bootstrap_code = include_str!("../js/bootstrap.js");

    let loader = std::rc::Rc::new(deno_core::StaticModuleLoader::new([(
        bootstrap_specifier.clone(),
        bootstrap_code.to_string(),
    )]));

    let create_params = CreateParams::default();

    let runtime_options = RuntimeOptions {
        extensions,
        module_loader: Some(loader),
        create_params: Some(create_params),
        ..Default::default()
    };

    Ok(JsRuntime::new(runtime_options))
}
