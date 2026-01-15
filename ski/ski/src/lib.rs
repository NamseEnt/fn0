mod http_body_resource;
mod module_loader;
mod ops;
mod runner;
mod runtime_options;
pub mod source_map;

pub use module_loader::{FetchedModule, ModuleFetcher, SsrModuleLoader};
pub use runner::{run, run_esm};
use bytes::Bytes;
use deno_core::anyhow;
use http_body_util::combinators::UnsyncBoxBody;
use std::future::Future;
use std::pin::Pin;

pub type Body = UnsyncBoxBody<Bytes, anyhow::Error>;
pub type Request = hyper::Request<Body>;
pub type Response = hyper::Response<Body>;

pub type FetchHandlerFuture = Pin<Box<dyn Future<Output = Option<Response>> + Send>>;

pub trait FetchHandler: Send + Sync + 'static {
    fn handle(&self, request: Request) -> FetchHandlerFuture;
}

pub struct SourceMapInfo {
    pub script_url: String,
    pub source_map_json: Vec<u8>,
}
