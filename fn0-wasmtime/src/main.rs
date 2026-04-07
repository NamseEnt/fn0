use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input = None;
    let mut output = None;
    let mut zstd_output = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--zstd" => {
                zstd_output = true;
            }
            _ => {
                if input.is_none() {
                    input = Some(args[i].clone());
                } else if output.is_none() {
                    output = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    let input = input.unwrap_or_else(|| {
        eprintln!("Usage: fn0-wasmtime [--zstd] <input.wasm> <output.cwasm>");
        process::exit(1);
    });
    let output = output.unwrap_or_else(|| {
        eprintln!("Usage: fn0-wasmtime [--zstd] <input.wasm> <output.cwasm>");
        process::exit(1);
    });

    let wasm_bytes = fs::read(&input).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", input, e);
        process::exit(1);
    });

    let cwasm = fn0_wasmtime::compile(&wasm_bytes).unwrap_or_else(|e| {
        eprintln!("Failed to compile: {}", e);
        process::exit(1);
    });

    if zstd_output {
        let compressed = zstd::encode_all(cwasm.as_slice(), 22).unwrap_or_else(|e| {
            eprintln!("Failed to compress: {}", e);
            process::exit(1);
        });
        fs::write(&output, &compressed).unwrap_or_else(|e| {
            eprintln!("Failed to write {}: {}", output, e);
            process::exit(1);
        });
        eprintln!(
            "Compiled {} ({} bytes) -> {} ({} bytes, zstd {:.1}%)",
            input,
            wasm_bytes.len(),
            output,
            compressed.len(),
            compressed.len() as f64 / cwasm.len() as f64 * 100.0
        );
    } else {
        fs::write(&output, &cwasm).unwrap_or_else(|e| {
            eprintln!("Failed to write {}: {}", output, e);
            process::exit(1);
        });
        eprintln!(
            "Compiled {} ({} bytes) -> {} ({} bytes)",
            input,
            wasm_bytes.len(),
            output,
            cwasm.len()
        );
    }
}
