//! Cargo test runner for `wasm32-wasip2` test components.
//!
//! Components that export `fn0:test-harness/harness` are driven test by test:
//! the runner lists the tests, then instantiates a fresh component per test so
//! one failure cannot take the rest of the run down with it (a guest panic is
//! an abort, which traps and poisons its instance). Anything else falls back to
//! calling `wasi:cli/run`, which is how plain command components still work.

use std::process::ExitCode;

use anyhow::{Result, anyhow};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::p2::bindings::CommandPre;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::WasiHttpCtx;
use wasmtime_wasi_http::p3::{WasiHttpCtxView, WasiHttpView, default_hooks};

mod harness {
    wasmtime::component::bindgen!({
        path: "../wit/wit",
        world: "fn0:test-harness/tests",
        exports: { default: async | store },
        require_store_data_send: true,
    });
}

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

/// A component is instantiated once per test, so everything that survives
/// across tests is built once and shared.
struct Harness {
    engine: Engine,
    pre: harness::TestsPre<State>,
    wasi_args: Vec<String>,
}

impl Harness {
    fn store(&self) -> Store<State> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_env()
            .args(&self.wasi_args)
            .build();
        Store::new(
            &self.engine,
            State {
                table: ResourceTable::new(),
                wasi,
                http: WasiHttpCtx::new(),
            },
        )
    }

    async fn list_tests(&self) -> Result<Vec<String>> {
        let mut store = self.store();
        let instance = self.pre.instantiate_async(&mut store).await?;
        let names = store
            .run_concurrent(async move |store| {
                instance
                    .fn0_test_harness_harness()
                    .call_list_tests(store)
                    .await
            })
            .await??;
        Ok(names)
    }

    async fn run_test(&self, name: &str) -> Result<(), String> {
        let mut store = self.store();
        let instance = self
            .pre
            .instantiate_async(&mut store)
            .await
            .map_err(|err| failure_message(&err))?;
        let name = name.to_string();
        let outcome = store
            .run_concurrent(async move |store| {
                instance
                    .fn0_test_harness_harness()
                    .call_run_test(store, name)
                    .await
            })
            .await;
        match outcome {
            Ok(Ok(Ok(()))) => Ok(()),
            Ok(Ok(Err(message))) => Err(message),
            Ok(Err(err)) | Err(err) => Err(failure_message(&err)),
        }
    }
}

/// The outermost error of a trap is a whole wasm backtrace; the reason sits at
/// the end of the chain (`wasm trap: ...`). The guest's own panic message has
/// already gone to the inherited stderr, so the summary wants only that reason.
fn failure_message(err: &wasmtime::Error) -> String {
    let rendered = format!("{}", err.root_cause());
    rendered
        .lines()
        .next()
        .unwrap_or("test failed")
        .trim_end_matches(':')
        .to_string()
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

    let instance_pre = linker.instantiate_pre(&component)?;

    let mut wasi_args = Vec::with_capacity(forwarded.len() + 1);
    wasi_args.push(component_path.clone());
    wasi_args.extend(forwarded.iter().cloned());

    match harness::TestsPre::new(instance_pre.clone()) {
        Ok(pre) => {
            let harness = Harness {
                engine,
                pre,
                wasi_args,
            };
            run_harness(harness, &forwarded).await
        }
        Err(_) => run_command(engine, CommandPre::new(instance_pre)?, wasi_args).await,
    }
}

async fn run_command(
    engine: Engine,
    pre: CommandPre<State>,
    wasi_args: Vec<String>,
) -> Result<ExitCode> {
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_env()
        .args(&wasi_args)
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

/// The subset of libtest's command line that means something here. Unknown
/// flags are ignored rather than rejected: cargo and IDEs pass options this
/// harness has no analogue for, and failing the run over them helps nobody.
struct Options {
    filters: Vec<String>,
    skips: Vec<String>,
    exact: bool,
    list: bool,
    only_ignored: bool,
}

fn parse_options(forwarded: &[String]) -> Options {
    let mut options = Options {
        filters: Vec::new(),
        skips: Vec::new(),
        exact: false,
        list: false,
        only_ignored: false,
    };
    let mut arguments = forwarded.iter();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--exact" => options.exact = true,
            "--list" => options.list = true,
            "--ignored" => options.only_ignored = true,
            "--skip" => {
                if let Some(pattern) = arguments.next() {
                    options.skips.push(pattern.clone());
                }
            }
            "--test-threads" | "--format" | "--logfile" | "--color" | "-Z" => {
                arguments.next();
            }
            _ if argument.starts_with('-') => {}
            _ => options.filters.push(argument.clone()),
        }
    }
    options
}

fn selected(name: &str, options: &Options) -> bool {
    // Nothing carries `#[ignore]`, so asking for only the ignored tests asks
    // for none of them.
    if options.only_ignored {
        return false;
    }
    if options.skips.iter().any(|skip| matches(name, skip, options)) {
        return false;
    }
    options.filters.is_empty()
        || options
            .filters
            .iter()
            .any(|filter| matches(name, filter, options))
}

fn matches(name: &str, pattern: &str, options: &Options) -> bool {
    if options.exact {
        name == pattern
    } else {
        name.contains(pattern)
    }
}

async fn run_harness(harness: Harness, forwarded: &[String]) -> Result<ExitCode> {
    let options = parse_options(forwarded);
    let all_tests = harness.list_tests().await?;
    let tests: Vec<String> = all_tests
        .iter()
        .filter(|name| selected(name, &options))
        .cloned()
        .collect();
    let filtered_out = all_tests.len() - tests.len();

    if options.list {
        for name in &tests {
            println!("{name}: test");
        }
        println!();
        println!("{} tests, 0 benchmarks", tests.len());
        return Ok(ExitCode::from(0));
    }

    println!();
    println!("running {} test{}", tests.len(), plural(tests.len()));

    let mut failures: Vec<(String, String)> = Vec::new();
    for name in &tests {
        print!("test {name} ... ");
        match harness.run_test(name).await {
            Ok(()) => println!("ok"),
            Err(message) => {
                println!("FAILED");
                failures.push((name.clone(), message));
            }
        }
    }

    if !failures.is_empty() {
        println!();
        println!("failures:");
        for (name, message) in &failures {
            println!("    {name}: {message}");
        }
    }

    let passed = tests.len() - failures.len();
    println!();
    println!(
        "test result: {}. {passed} passed; {} failed; {filtered_out} filtered out",
        if failures.is_empty() { "ok" } else { "FAILED" },
        failures.len()
    );
    println!();

    if failures.is_empty() {
        Ok(ExitCode::from(0))
    } else {
        Ok(ExitCode::from(1))
    }
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}
