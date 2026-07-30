mod wasi_version_rewrite;

use wasmtime::*;

pub use wasi_version_rewrite::{is_component, rewrite_wasi_030_rc_names};
pub use wasmtime;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn engine_config() -> Config {
    const MB: usize = 1024 * 1024;

    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

    let total_memory_bytes = sys.total_memory() as usize;
    let total_memory_mb = total_memory_bytes / (1024 * 1024);
    const MAX_MEMORY_MB: usize = 128;
    let max_instance_count = total_memory_mb / MAX_MEMORY_MB;

    let mut pooling_allocation_config = PoolingAllocationConfig::new();
    pooling_allocation_config
        .max_memory_size(MB * MAX_MEMORY_MB)
        .linear_memory_keep_resident(MB * 16)
        .table_keep_resident(MB)
        .total_core_instances(max_instance_count as _)
        .total_memories(max_instance_count as _)
        .total_tables(max_instance_count as _)
        .pagemap_scan(wasmtime::Enabled::Auto)
        .memory_protection_keys(wasmtime::Enabled::Auto);

    let mut config = Config::new();

    config
        .allocation_strategy(InstanceAllocationStrategy::Pooling(
            pooling_allocation_config,
        ))
        .epoch_interruption(true)
        .wasm_component_model(true)
        .wasm_component_model_async(true)
        .cranelift_opt_level(wasmtime::OptLevel::None)
        .cache(Some(
            wasmtime::Cache::new(wasmtime::CacheConfig::new()).unwrap(),
        ))
        .parallel_compilation(true);

    config
}

pub fn compile(wasm_bytes: &[u8]) -> Result<Vec<u8>, Error> {
    let engine = Engine::new(&engine_config())?;

    if !is_component(wasm_bytes) {
        return engine.precompile_module(wasm_bytes);
    }

    let rewritten = rewrite_wasi_030_rc_names(wasm_bytes);
    let component = rewritten.as_deref().unwrap_or(wasm_bytes);

    // Precompiling resolves nothing by name, so a component whose imports no
    // host offers still compiles and only fails when a bundle is served. These
    // two checks move that to compile time, which is where the deploy pipeline
    // can act on it.
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(component)
        .map_err(|e| Error::msg(format!("component failed validation: {e}")))?;
    if let Some(name) = prerelease_wasi_name(component) {
        return Err(Error::msg(format!(
            "component imports or exports `{name}`, which no released WASI \
             version provides; rebuild it with a current forte-sdk"
        )));
    }

    engine.precompile_component(component)
}

fn prerelease_wasi_name(component: &[u8]) -> Option<String> {
    let mut found = None;
    for payload in wasmparser::Parser::new(0).parse_all(component) {
        let names: Vec<&str> = match payload {
            Ok(wasmparser::Payload::ComponentImportSection(reader)) => reader
                .into_iter()
                .filter_map(|import| import.ok().map(|import| import.name.name))
                .collect(),
            Ok(wasmparser::Payload::ComponentExportSection(reader)) => reader
                .into_iter()
                .filter_map(|export| export.ok().map(|export| export.name.name))
                .collect(),
            _ => continue,
        };
        if let Some(name) = names
            .into_iter()
            .find(|name| name.starts_with("wasi:") && name.contains("-rc-"))
        {
            found = Some(name.to_string());
            break;
        }
    }
    found
}
