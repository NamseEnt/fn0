//! The bench-side view of the corpus.
//!
//! The generator itself is `signy::corpus`, so the benches and
//! `bin/load` measure the same bytes. What stays here is what the library must
//! not carry: a `#[global_allocator]` and a scratch directory that deletes
//! itself.

pub mod alloc;
pub mod scratch;

pub use signy::corpus::*;
