//! What the rewrite has to guarantee for a bundle built before the wasmtime 47
//! bump: the ratified names replace the prerelease ones, the component stays
//! valid, and anything already on `@0.3.0` is left byte-for-byte alone.

const RC_NAMED_SERVICE: &[u8] = include_bytes!("fixtures/rc-named-service.wasm");

fn component_names(component: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(component) {
        match payload {
            Ok(wasmparser::Payload::ComponentImportSection(reader)) => {
                names.extend(reader.into_iter().flatten().map(|i| i.name.name.to_string()));
            }
            Ok(wasmparser::Payload::ComponentExportSection(reader)) => {
                names.extend(reader.into_iter().flatten().map(|e| e.name.name.to_string()));
            }
            _ => {}
        }
    }
    names
}

fn assert_valid(component: &[u8]) {
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(component)
        .expect("rewritten component must stay valid");
}

#[test]
fn fixture_is_the_prerelease_shape_the_rewrite_targets() {
    assert_valid(RC_NAMED_SERVICE);
    let names = component_names(RC_NAMED_SERVICE);
    assert!(
        names
            .iter()
            .any(|name| name == "wasi:http/types@0.3.0-rc-2026-03-15"),
        "fixture lost its prerelease imports: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|name| name == "wasi:http/handler@0.3.0-rc-2026-03-15"),
        "fixture lost its prerelease export: {names:?}"
    );
}

#[test]
fn rewrite_moves_every_wasi_name_to_the_ratified_version() {
    let rewritten =
        fn0_wasmtime::rewrite_wasi_030_rc_names(RC_NAMED_SERVICE).expect("fixture must be rewritten");
    assert_valid(&rewritten);

    let names = component_names(&rewritten);
    assert!(
        !names.iter().any(|name| name.contains("-rc-")),
        "prerelease names survived: {names:?}"
    );
    assert!(names.iter().any(|name| name == "wasi:http/types@0.3.0"));
    assert!(names.iter().any(|name| name == "wasi:http/handler@0.3.0"));

    let before = component_names(RC_NAMED_SERVICE).len();
    assert_eq!(before, names.len(), "rewrite changed the name count");
}

#[test]
fn rewrite_leaves_a_ratified_component_untouched() {
    let once = fn0_wasmtime::rewrite_wasi_030_rc_names(RC_NAMED_SERVICE).unwrap();
    assert!(
        fn0_wasmtime::rewrite_wasi_030_rc_names(&once).is_none(),
        "a component already on @0.3.0 must not be rewritten again"
    );
}

#[test]
fn rewrite_skips_core_modules() {
    let core_module = b"\0asm\x01\0\0\0";
    assert!(!fn0_wasmtime::is_component(core_module));
    assert!(fn0_wasmtime::rewrite_wasi_030_rc_names(core_module).is_none());
}

/// The whole point of the rewrite: the fixture is what a pre-bump bundle looks
/// like, and it has to survive the compile step the cwasm compiler runs.
#[test]
fn compile_accepts_a_prerelease_bundle() {
    fn0_wasmtime::compile(RC_NAMED_SERVICE).expect("prerelease bundle must compile");
}
