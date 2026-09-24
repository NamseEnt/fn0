use std::fmt;

macro_rules! integer_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(u64);

        impl $name {
            pub const ZERO: Self = Self(0);

            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}

integer_id!(
    /// Tenant identity. It is deliberately distinct from [`ShardId`].
    TenantId
);
integer_id!(
    /// Physical shard identity. It is deliberately distinct from [`TenantId`].
    ShardId
);
integer_id!(
    /// Fencing/ownership epoch for a shard.
    ShardEpoch
);
integer_id!(
    /// Fixed-size storage page identity.
    PageId
);
integer_id!(
    /// Commit/log sequence number. A committed document revision is based on this value.
    Lsn
);
integer_id!(
    /// Transaction identity.
    TxnId
);
integer_id!(
    /// Last committed state-transition identifier for one document key.
    Revision
);
