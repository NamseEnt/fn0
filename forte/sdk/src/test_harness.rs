//! Test discovery and execution for `#[forte_sdk::test]`, driven by
//! `forte-test-runner` over the `fn0:test-harness/harness` export.
//!
//! libtest cannot host these tests: its `main` reaches wasm through the
//! `wasm32-wasip2` target's synchronous `wasi:cli/run`, and a synchronous task
//! traps rather than blocking on a host future, so any test doing real I/O dies
//! on its first await. Tests register themselves here instead, and the runner
//! calls the async `run-test` export once per test with a fresh instance.
//!
//! A test crate opts in with `harness = false` on its target plus a single
//! [`crate::test_main!`] invocation.

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

pub mod bindings {
    wit_bindgen::generate!({
        path: "wit",
        world: "fn0:test-harness/tests",
        default_bindings_module: "forte_sdk::test_harness::bindings",
        pub_export_macro: true,
        // The harness world imports nothing, but `path` is the shared WASI wit
        // directory and it does not parse with this feature left gated off.
        features: ["clocks-timezone"],
    });
}

pub type TestFuture = Pin<Box<dyn Future<Output = Result<(), String>>>>;

pub struct RegisteredTest {
    pub module_path: &'static str,
    pub name: &'static str,
    pub run: fn() -> TestFuture,
}

impl RegisteredTest {
    /// The name the runner reports, matching libtest's `module::path::test`
    /// spelling: the crate segment `module_path!()` starts with is dropped.
    pub fn full_name(&self) -> String {
        match self.module_path.split_once("::") {
            Some((_crate_name, module_path_within_crate)) => {
                format!("{module_path_within_crate}::{}", self.name)
            }
            None => self.name.to_string(),
        }
    }
}

inventory::collect!(RegisteredTest);

pub fn registered_tests() -> Vec<&'static RegisteredTest> {
    let mut tests: Vec<&'static RegisteredTest> = inventory::iter::<RegisteredTest>
        .into_iter()
        .collect();
    tests.sort_by_key(|test| test.full_name());
    tests
}

/// Normalizes what a test function returns into a pass/fail outcome, the way
/// libtest's `Termination` bound does for `#[test]`.
pub trait TestOutcome {
    fn into_outcome(self) -> Result<(), String>;
}

impl TestOutcome for () {
    fn into_outcome(self) -> Result<(), String> {
        Ok(())
    }
}

impl<T, E: Debug> TestOutcome for Result<T, E> {
    fn into_outcome(self) -> Result<(), String> {
        self.map(|_| ()).map_err(|error| format!("{error:?}"))
    }
}

pub struct Harness;

impl bindings::exports::fn0::test_harness::harness::Guest for Harness {
    async fn list_tests() -> Vec<String> {
        registered_tests()
            .into_iter()
            .map(RegisteredTest::full_name)
            .collect()
    }

    async fn run_test(name: String) -> Result<(), String> {
        let test = registered_tests()
            .into_iter()
            .find(|test| test.full_name() == name)
            .ok_or_else(|| format!("no test named `{name}` is registered"))?;
        (test.run)().await
    }
}

pub fn print_direct_run_hint() {
    eprintln!(
        "this is a Forte test component; run it with `cargo test`, which drives \
         it through forte-test-runner"
    );
}

/// Wires a test target into the harness: exports `fn0:test-harness/harness` and
/// supplies the `main` that `harness = false` requires.
#[macro_export]
macro_rules! test_main {
    () => {
        use $crate::test_harness::Harness as ForteTestHarness;

        $crate::test_harness::bindings::export!(ForteTestHarness with_types_in $crate::test_harness::bindings);

        fn main() {
            $crate::test_harness::print_direct_run_hint();
        }
    };
}
