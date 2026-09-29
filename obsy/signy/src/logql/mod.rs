use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};

use regex::Regex;

use crate::memtable::{Labels, LogEntry};
use crate::part::ExactFieldPredicate;

pub const PARSER_ERROR_FIELD: &str = "__error__";
/// Loki pairs `__error__` with a second label describing the failure, and
/// Grafana shows both. The *text* is each engine's own — Loki's comes from its
/// JSON library's internals — so the comparison digests this label by
/// presence and treats the wording as unmatchable.
pub const PARSER_ERROR_DETAILS_FIELD: &str = "__error_details__";

include!("field_filters.rs");
include!("ast.rs");
include!("pipeline.rs");

#[cfg(test)]
mod tests {
    include!("tests.rs");
}
