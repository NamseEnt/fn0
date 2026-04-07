use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut input = None;
    let mut output = None;
    let mut target = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                i += 1;
                target = Some(args[i].clone());
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
        eprintln!("Usage: fn0-compiler [--target <triple>] <input.wasm> <output.cwasm>");
        process::exit(1);
    });
    let output = output.unwrap_or_else(|| {
        eprintln!("Usage: fn0-compiler [--target <triple>] <input.wasm> <output.cwasm>");
        process::exit(1);
    });

    let wasm_bytes = fs::read(&input).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", input, e);
        process::exit(1);
    });

    let cwasm = fn0::compile_for_target(&wasm_bytes, target.as_deref()).unwrap_or_else(|e| {
        eprintln!("Failed to compile: {}", e);
        process::exit(1);
    });

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
