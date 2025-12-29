mod http_body_resource;
mod runtime_options;

use bytes::Bytes;
use deno_core::anyhow::{Result, anyhow};
use deno_core::*;
use http::*;
use http_body_resource::*;
use http_body_util::combinators::UnsyncBoxBody;
use http_body_util::{BodyExt, Empty};
use runtime_options::*;
use std::rc::Rc;

type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
type Request = hyper::Request<Body>;
type Response = hyper::Response<Body>;

static RUNTIME_SNAPSHOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/RUNJS_SNAPSHOT.bin"));
static RUN_JS: FastStaticString = ascii_str_include!("../run.js");

pub async fn run(code: &str, request: Request) -> Result<Response> {
    let mut runtime_options = runtime_options();
    runtime_options.startup_snapshot = Some(RUNTIME_SNAPSHOT);

    let mut runtime = JsRuntime::new(runtime_options);
    runtime.execute_script("[user code]", code.to_string())?;

    {
        let op_state = runtime.op_state();
        let mut state = op_state.borrow_mut();
        let (url, method, headers, rid) = register_hyper_request(&mut state, request);
        state.put(RequestParts {
            url,
            method,
            headers,
            rid,
        });
    }

    {
        let script_result = runtime.execute_script("[run]", RUN_JS)?;
        let run_future = runtime.resolve(script_result);
        runtime.run_event_loop(Default::default()).await?;
        run_future.await?;
    }

    let op_state = runtime.op_state();

    let response_parts = op_state
        .borrow_mut()
        .try_take::<ResponseParts>()
        .ok_or_else(|| anyhow!("Did not get a response from JavaScript"))?;

    let mut builder =
        hyper::Response::builder().status(StatusCode::from_u16(response_parts.status)?);

    for (key, value) in response_parts.headers {
        if let Ok(name) = HeaderName::from_bytes(key.as_bytes()) {
            builder = builder.header(name, value);
        }
    }

    let Some(rid) = response_parts.rid else {
        let body = BodyExt::boxed_unsync(Empty::<Bytes>::new().map_err(|never| match never {}));
        return Ok(builder.body(body)?);
    };

    let resource = op_state
        .borrow_mut()
        .resource_table
        .take::<HttpBodyResource>(rid)
        .map_err(|_| anyhow!("Resource not found"))?;
    let body = Rc::try_unwrap(resource)
        .map_err(|_| anyhow!("Failed to unwrap resource"))?
        .body;
    let body = Rc::try_unwrap(body)
        .map_err(|_| anyhow!("Failed to unwrap body"))?
        .into_inner();

    Ok(builder.body(body)?)
}

fn register_hyper_request(
    state: &mut OpState,
    req: Request,
) -> (String, String, Vec<(String, String)>, Option<ResourceId>) {
    let (parts, body) = req.into_parts();

    let url = parts.uri.to_string();
    let method = parts.method.to_string();
    let headers: Vec<(String, String)> = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    let rid = if method == "GET" || method == "HEAD" {
        None
    } else {
        let resource = HttpBodyResource::new(body);
        Some(state.resource_table.add(resource))
    };

    (url, method, headers, rid)
}
