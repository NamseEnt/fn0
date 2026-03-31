use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: fn0-compiler <input.wasm> <output.cwasm>");
        process::exit(1);
    }

    let input = &args[1];
    let output = &args[2];

    let wasm_bytes = fs::read(input).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", input, e);
        process::exit(1);
    });

    let cwasm = fn0::compile(&wasm_bytes).unwrap_or_else(|e| {
        eprintln!("Failed to compile: {}", e);
        process::exit(1);
    });

    fs::write(output, &cwasm).unwrap_or_else(|e| {
        eprintln!("Failed to write {}: {}", output, e);
        process::exit(1);
    });

    eprintln!("Compiled {} ({} bytes) -> {} ({} bytes)", input, wasm_bytes.len(), output, cwasm.len());
}
