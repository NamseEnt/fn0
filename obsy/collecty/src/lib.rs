/// The resource attribute signy reads a tenant from.
///
/// collecty does not use it on anything it forwards — a payload it never
/// decodes is a payload it cannot read this out of, which is the whole point
/// of the tenant living in there. It is here for exports collecty builds
/// itself, including its own metrics and enabled host sources.
pub const TENANT_ATTRIBUTE: &str = "tenant.id";

pub mod config;
pub mod host_metrics;
pub mod journal;
pub mod memprof;
pub mod observe;
pub mod queue;
pub mod receive;
pub mod send;
pub mod signal;
#[cfg(test)]
pub mod test_support;
pub mod wire;
