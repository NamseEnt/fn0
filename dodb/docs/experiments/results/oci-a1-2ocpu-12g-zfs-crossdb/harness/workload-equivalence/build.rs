use std::path::PathBuf;

fn segment<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = source
        .find(start)
        .unwrap_or_else(|| panic!("start marker not found: {start}"));
    let end_index = source[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("end marker not found: {end}"));
    &source[start_index..end_index]
}

fn main() {
    println!("cargo:rerun-if-env-changed=DODB_PHASE0_SOURCE");
    let source_path = PathBuf::from(
        std::env::var("DODB_PHASE0_SOURCE").expect("DODB_PHASE0_SOURCE must point at phase0-bench.rs"),
    );
    println!("cargo:rerun-if-changed={}", source_path.display());
    let source = std::fs::read_to_string(&source_path).expect("phase0-bench.rs should be readable");
    let enums = segment(
        &source,
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\nenum Distribution {",
        "#[derive(Clone, Copy, Debug, Eq, PartialEq)]\nenum SyncMode {",
    );
    let generator = segment(
        &source,
        "#[derive(Clone, Debug)]\nstruct WorkloadConfig {",
        "struct BenchFile {",
    );
    let seed_formula = segment(&source, "let seed = args\n", ";\n");
    let seed_value = segment(&source, "value: value_bytes(args.value_size, index as u64, 0)", "\n");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    std::fs::write(
        out_dir.join("original_generator.rs"),
        format!("{enums}\n{generator}\n"),
    )
    .unwrap();
    std::fs::write(out_dir.join("seed_formula.txt"), seed_formula).unwrap();
    std::fs::write(out_dir.join("seed_value.txt"), seed_value).unwrap();
    let writer_masks = ["seed ^ 0xaaaa_0000", "seed ^ 0xbbbb_0000", "seed ^ 0x1000_0000"]
        .iter()
        .map(|mask| format!("{mask}: {}", source.contains(mask)))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(out_dir.join("writer_masks.txt"), writer_masks).unwrap();
}
