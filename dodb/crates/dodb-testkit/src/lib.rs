//! Deterministic correctness and crash-testing foundations.

pub mod crash;
pub mod crashable_file;
pub mod reference;

pub use crash::{CrashInjected, CrashInjector};
pub use crashable_file::{CrashableFile, FaultAction, FaultPlan, FileOperation};
pub use reference::{Document, ReferenceDb, ReferenceTransaction};
