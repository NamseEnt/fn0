use std::process::ExitCode;

use anyhow::{Result, anyhow};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::p2::bindings::CommandPre;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::WasiHttpCtx;
use wasmtime_wasi_http::p3::{WasiHttpCtxView, WasiHttpView, default_hooks};

struct State {
    table: ResourceTable,
    wasi: WasiCtx,
    http: WasiHttpCtx,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl WasiHttpView for State {
    fn http(&mut self) -> WasiHttpCtxView<'_> {
        WasiHttpCtxView {
            ctx: &mut self.http,
            table: &mut self.table,
            hooks: default_hooks(),
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("forte-test-runner: {err:?}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode> {
    let mut args = std::env::args();
    let _exe = args.next();
    let component_path = args
        .next()
        .ok_or_else(|| anyhow!("usage: forte-test-runner <component.wasm> [args...]"))?;
    let forwarded: Vec<String> = args.collect();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move { run_component(component_path, forwarded).await })
}

async fn run_component(component_path: String, forwarded: Vec<String>) -> Result<ExitCode> {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    let engine = Engine::new(&config)?;

    let component = Component::from_file(&engine, &component_path)?;

    let mut linker: Linker<State> = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_async(&mut linker)?;
    wasmtime_wasi::p3::add_to_linker(&mut linker)?;
    wasmtime_wasi_http::p3::add_to_linker(&mut linker)?;

    let pre = CommandPre::new(linker.instantiate_pre(&component)?)?;

    let mut args = Vec::with_capacity(forwarded.len() + 1);
    args.push(component_path.clone());
    args.extend(forwarded);

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .args(&args)
        .build();

    let mut store = Store::new(
        &engine,
        State {
            table: ResourceTable::new(),
            wasi,
            http: WasiHttpCtx::new(),
        },
    );

    let command = pre.instantiate_async(&mut store).await?;
    match command.wasi_cli_run().call_run(&mut store).await? {
        Ok(()) => Ok(ExitCode::from(0)),
        Err(()) => Ok(ExitCode::from(1)),
    }
}
