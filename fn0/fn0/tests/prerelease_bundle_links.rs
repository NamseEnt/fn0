//! A bundle built against the pre-ratification WASI 0.3 WIT has to keep serving
//! traffic after the wasmtime bump, without being rebuilt. Linking is where that
//! is decided: wasmtime resolves imports by name, and its semver-compatible
//! matching skips prereleases, so `@0.3.0-rc-2026-03-15` never resolves against
//! the `@0.3.0` this host offers.

use fn0::wasmtime::{Engine, component::Component};

const RC_NAMED_SERVICE: &[u8] =
    include_bytes!("../../wasmtime/tests/fixtures/rc-named-service.wasm");

fn engine() -> Engine {
    Engine::new(&fn0::execute::engine_config()).expect("engine")
}

#[test]
fn prerelease_names_do_not_link_against_this_host() {
    let engine = engine();
    let linker = fn0::build_linker(&engine);

    let component = Component::new(&engine, RC_NAMED_SERVICE)
        .expect("a prerelease component still compiles; only linking rejects it");
    let error = match linker.instantiate_pre(&component) {
        Ok(_) => panic!("prerelease imports must not resolve"),
        Err(error) => error,
    };

    let rendered = format!("{error:?}");
    assert!(
        rendered.contains("0.3.0-rc-2026-03-15"),
        "expected the unresolved prerelease import to be named: {rendered}"
    );
}

#[test]
fn rewritten_bundle_links_and_exposes_the_http_service() {
    let engine = engine();
    let linker = fn0::build_linker(&engine);

    let cwasm = fn0::compile(RC_NAMED_SERVICE).expect("compile rewrites and precompiles");
    fn0::build_service_pre(&engine, &linker, &cwasm)
        .expect("rewritten bundle must link and export wasi:http/handler");
}
