use crate::{Lsn, Revision};

/// The committed state of one logical document key.
///
/// `Missing` is intentionally not represented as `Option`: a deleted key
/// retains the LSN of its deletion so optimistic validation can detect ABA.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevisionState {
    Present { value: Vec<u8>, revision: Revision },
    Missing { revision: Revision },
}

/// The committed presence and identity of one logical document key without
/// materializing its value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservedState {
    Present { revision: Revision },
    Missing { revision: Revision },
}

impl ObservedState {
    pub const fn present(revision: Revision) -> Self {
        Self::Present { revision }
    }

    pub const fn missing(revision: Revision) -> Self {
        Self::Missing { revision }
    }

    pub const fn revision(self) -> Revision {
        match self {
            Self::Present { revision } | Self::Missing { revision } => revision,
        }
    }

    pub const fn is_missing(self) -> bool {
        matches!(self, Self::Missing { .. })
    }
}

impl RevisionState {
    pub fn present(value: impl Into<Vec<u8>>, revision: Revision) -> Self {
        Self::Present {
            value: value.into(),
            revision,
        }
    }

    pub fn missing(revision: Revision) -> Self {
        Self::Missing { revision }
    }

    pub const fn revision(&self) -> Revision {
        match self {
            Self::Present { revision, .. } | Self::Missing { revision } => *revision,
        }
    }

    pub fn value(&self) -> Option<&[u8]> {
        match self {
            Self::Present { value, .. } => Some(value),
            Self::Missing { .. } => None,
        }
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing { .. })
    }

    pub const fn observed(&self) -> ObservedState {
        match self {
            Self::Present { revision, .. } => ObservedState::Present {
                revision: *revision,
            },
            Self::Missing { revision } => ObservedState::Missing {
                revision: *revision,
            },
        }
    }
}

impl From<Lsn> for Revision {
    fn from(lsn: Lsn) -> Self {
        Self::new(lsn.get())
    }
}
