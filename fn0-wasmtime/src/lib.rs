use anyhow::{anyhow, Result};
use wasmtime::*;

pub use wasmtime;

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
        .async_support(true)
        .allocation_strategy(InstanceAllocationStrategy::Pooling(
            pooling_allocation_config,
        ))
        .epoch_interruption(true)
        .wasm_component_model(true)
        .cranelift_opt_level(wasmtime::OptLevel::None)
        .cache(Some(
            wasmtime::Cache::new(wasmtime::CacheConfig::new()).unwrap(),
        ))
        .parallel_compilation(true);

    config
}

pub fn compile(wasm_bytes: &[u8]) -> Result<Vec<u8>> {
    compile_for_target(wasm_bytes, None)
}

pub fn compile_for_target(wasm_bytes: &[u8], target: Option<&str>) -> Result<Vec<u8>> {
    let mut config = engine_config();
    if let Some(target) = target {
        config.target(target).map_err(|e| anyhow!("{e}"))?;
    }
    let engine = Engine::new(&config).map_err(|e| anyhow!("{e}"))?;

    let result = if wasm_bytes.len() > 8 && wasm_bytes[4..8] == [0x0d, 0x00, 0x01, 0x00] {
        engine.precompile_component(wasm_bytes)
    } else {
        engine.precompile_module(wasm_bytes)
    };
    result.map_err(|e| anyhow!("{e}"))
}
