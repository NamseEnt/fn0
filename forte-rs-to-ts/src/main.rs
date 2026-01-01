#![feature(rustc_private)]

extern crate rustc_driver;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod analyzer;
mod name_resolution;
mod rust_to_ts;
mod ts_codegen;

use analyzer::Analyzer;
use rustc_driver::run_compiler;
use std::env;
use std::process::Command;

fn main() {
    if env::var("MY_ANALYZER_WRAPPER_MODE").is_ok() {
        let mut args: Vec<String> = env::args().collect();

        let is_build_script = args.iter().any(|arg| arg == "build_script_build");

        if is_build_script {
            let rustc_path = &args[1];
            let rustc_args = &args[2..];

            let status = Command::new(rustc_path)
                .args(rustc_args)
                .status()
                .expect("Failed to execute original rustc");

            std::process::exit(status.code().unwrap_or(1));
        }

        if args.len() > 1 {
            args.remove(1);
        }
        let mut callbacks = Analyzer {
            ts_output_dir: "../fe/src/pages".to_string(),
        };
        run_compiler(&args, &mut callbacks);
        return;
    }
    let target_dir = env::args()
        .nth(1)
        .unwrap_or_else(|| "../forte-manual/rs".to_string());

    let current_exe = env::current_exe().expect("Failed to find current exe");
    println!("Running cargo check on: {target_dir}");

    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&target_dir)
        .env("RUSTC_WORKSPACE_WRAPPER", current_exe)
        .env("MY_ANALYZER_WRAPPER_MODE", "true")
        .status()
        .expect("Failed to run cargo");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}
