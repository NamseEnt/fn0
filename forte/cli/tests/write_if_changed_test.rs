use std::fs;
use std::process::Command;

#[test]
fn test_write_if_changed_prevents_rebuild_loop() {
    let project_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("forte-manual/rs");

    let route_generated_path = project_dir.join("src/route_generated.rs");

    Command::new("cargo")
        .arg("build")
        .current_dir(&project_dir)
        .status()
        .expect("First cargo build failed");

    let mtime_after_first_build = fs::metadata(&route_generated_path)
        .expect("route_generated.rs should exist")
        .modified()
        .expect("Should be able to get mtime");

    std::thread::sleep(std::time::Duration::from_millis(100));

    Command::new("cargo")
        .arg("build")
        .current_dir(&project_dir)
        .status()
        .expect("Second cargo build failed");

    let mtime_after_second_build = fs::metadata(&route_generated_path)
        .expect("route_generated.rs should exist")
        .modified()
        .expect("Should be able to get mtime");

    assert_eq!(
        mtime_after_first_build, mtime_after_second_build,
        "route_generated.rs should NOT be modified on second build (write-if-changed)"
    );
}
