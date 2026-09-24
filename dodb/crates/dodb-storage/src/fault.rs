use dodb_core::Result;

/// Optional deterministic hook used by crash tests and fault-injection
/// harnesses. Production stores leave it unset.
pub trait FaultInjector {
    fn hit(&mut self, point: &str) -> Result<()>;
}
