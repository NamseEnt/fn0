mod codegen;
mod models;
mod type_mapper;

use anyhow::{Context, Result};
use clap::Parser;
use models::ParsedModel;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "oci-gen")]
#[command(about = "OCI Rust SDK code generator from TypeScript SDK", long_about = None)]
struct Args {
    /// Path to parsed models JSON file
    #[arg(short, long)]
    input: PathBuf,

    /// Output directory for generated Rust files
    #[arg(short, long)]
    output: PathBuf,

    /// Limit number of models to generate (for testing)
    #[arg(short, long)]
    limit: Option<usize>,

    /// Dry run - don't write files
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("OCI Rust SDK Code Generator");
    println!("===========================\n");

    // Read parsed models
    println!("Reading models from: {}", args.input.display());
    let json_content = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read input file: {}", args.input.display()))?;

    let models: Vec<ParsedModel> = serde_json::from_str(&json_content)
        .context("Failed to parse JSON")?;

    println!("Loaded {} models", models.len());

    // Apply limit if specified
    let models_to_generate: Vec<_> = if let Some(limit) = args.limit {
        models.into_iter().take(limit).collect()
    } else {
        models
    };

    println!("Generating {} models\n", models_to_generate.len());

    // Initialize code generator
    let generator = codegen::CodeGenerator::new()?;

    // Generate code for each model
    let mut interface_count = 0;
    let mut enum_count = 0;

    for model in &models_to_generate {
        match generator.generate_model(model) {
            Ok(code) => {
                if model.is_interface() {
                    interface_count += 1;
                } else if model.is_enum() {
                    enum_count += 1;
                }

                if !args.dry_run {
                    let file_name = format!("{}.rs", model.file_name);
                    let output_path = args.output.join(&file_name);

                    fs::write(&output_path, code)
                        .with_context(|| format!("Failed to write {}", output_path.display()))?;

                    println!("  ✓ Generated {}", file_name);
                } else {
                    println!("  [DRY RUN] Would generate {}.rs ({} lines)",
                        model.file_name, code.lines().count());
                }
            }
            Err(e) => {
                eprintln!("  ✗ Failed to generate {}: {:?}", model.name, e);
            }
        }
    }

    println!("\nGeneration complete!");
    println!("  Interfaces: {}", interface_count);
    println!("  Enums: {}", enum_count);

    if args.dry_run {
        println!("\n(Dry run - no files were written)");
    }

    Ok(())
}
