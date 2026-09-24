use std::fmt;

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PrimaryKey(Vec<u8>);

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SortKey(Vec<u8>);

#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DocumentKey {
    pub pk: PrimaryKey,
    pub sk: SortKey,
}

impl PrimaryKey {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl SortKey {
    pub fn new(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl DocumentKey {
    pub fn new(pk: impl Into<Vec<u8>>, sk: impl Into<Vec<u8>>) -> Self {
        Self {
            pk: PrimaryKey::new(pk),
            sk: SortKey::new(sk),
        }
    }

    pub fn from_parts(pk: PrimaryKey, sk: SortKey) -> Self {
        Self { pk, sk }
    }

    /// Encodes `(pk, sk)` as two escaped components.
    ///
    /// A zero byte in a component is encoded as `00 FF`; the component
    /// terminator is `00 00`. This makes both arbitrary binary data and empty
    /// components order-preserving under bytewise lexicographic comparison.
    pub fn encode(&self) -> Vec<u8> {
        let mut encoded = Vec::with_capacity(self.pk.0.len() + self.sk.0.len() + 4);
        encode_component(&self.pk.0, &mut encoded);
        encode_component(&self.sk.0, &mut encoded);
        encoded
    }

    /// Returns the length of the canonical encoding without allocating it.
    pub fn encoded_len(&self) -> usize {
        encoded_component_len(&self.pk.0) + encoded_component_len(&self.sk.0)
    }

    /// Decodes the exact canonical encoding produced by [`Self::encode`].
    pub fn decode(encoded: &[u8]) -> Result<Self, KeyCodecError> {
        let (pk, next) = decode_component(encoded, 0)?;
        let (sk, end) = decode_component(encoded, next)?;
        if end != encoded.len() {
            return Err(KeyCodecError::TrailingBytes { offset: end });
        }
        Ok(Self::from_parts(PrimaryKey::new(pk), SortKey::new(sk)))
    }
}

impl fmt::Debug for PrimaryKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("PrimaryKey").field(&self.0).finish()
    }
}

impl fmt::Debug for SortKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("SortKey").field(&self.0).finish()
    }
}

impl fmt::Debug for DocumentKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DocumentKey")
            .field("pk", &self.pk)
            .field("sk", &self.sk)
            .finish()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum KeyCodecError {
    TruncatedEscape { offset: usize },
    InvalidEscape { offset: usize, byte: u8 },
    MissingComponent { offset: usize },
    TrailingBytes { offset: usize },
}

impl fmt::Display for KeyCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedEscape { offset } => write!(formatter, "truncated escape at {offset}"),
            Self::InvalidEscape { offset, byte } => {
                write!(formatter, "invalid escape byte {byte:#04x} at {offset}")
            }
            Self::MissingComponent { offset } => write!(formatter, "missing component at {offset}"),
            Self::TrailingBytes { offset } => write!(formatter, "trailing bytes at {offset}"),
        }
    }
}

impl std::error::Error for KeyCodecError {}

fn encode_component(bytes: &[u8], output: &mut Vec<u8>) {
    for &byte in bytes {
        if byte == 0 {
            output.extend_from_slice(&[0, 0xff]);
        } else {
            output.push(byte);
        }
    }
    output.extend_from_slice(&[0, 0]);
}

fn encoded_component_len(bytes: &[u8]) -> usize {
    bytes.len() + bytes.iter().filter(|byte| **byte == 0).count() + 2
}

fn decode_component(encoded: &[u8], start: usize) -> Result<(Vec<u8>, usize), KeyCodecError> {
    if start >= encoded.len() {
        return Err(KeyCodecError::MissingComponent { offset: start });
    }

    let mut bytes = Vec::new();
    let mut offset = start;
    while offset < encoded.len() {
        let byte = encoded[offset];
        if byte != 0 {
            bytes.push(byte);
            offset += 1;
            continue;
        }

        let Some(&escape) = encoded.get(offset + 1) else {
            return Err(KeyCodecError::TruncatedEscape { offset });
        };
        match escape {
            0 => return Ok((bytes, offset + 2)),
            0xff => {
                bytes.push(0);
                offset += 2;
            }
            other => {
                return Err(KeyCodecError::InvalidEscape {
                    offset,
                    byte: other,
                });
            }
        }
    }

    Err(KeyCodecError::TruncatedEscape { offset })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn round_trip_arbitrary_binary(pk in proptest::collection::vec(any::<u8>(), 0..80), sk in proptest::collection::vec(any::<u8>(), 0..80)) {
            let key = DocumentKey::new(pk, sk);
            let encoded = key.encode();
            prop_assert_eq!(DocumentKey::decode(&encoded), Ok(key));
        }

        #[test]
        fn encoding_preserves_document_order(
            apk in proptest::collection::vec(any::<u8>(), 0..20),
            ask in proptest::collection::vec(any::<u8>(), 0..20),
            bpk in proptest::collection::vec(any::<u8>(), 0..20),
            bsk in proptest::collection::vec(any::<u8>(), 0..20),
        ) {
            let left = DocumentKey::new(apk, ask);
            let right = DocumentKey::new(bpk, bsk);
            prop_assert_eq!(left.cmp(&right), left.encode().cmp(&right.encode()));
        }
    }

    #[test]
    fn malformed_encodings_are_rejected() {
        for input in [
            vec![],
            vec![0],
            vec![0, 1],
            vec![0, 0],
            vec![0, 0, 0],
            vec![1, 0, 0, 0, 0, 9],
        ] {
            assert!(DocumentKey::decode(&input).is_err(), "accepted {input:?}");
        }
        assert!(DocumentKey::decode(&[0, 0, 0, 0]).is_ok());
    }

    #[test]
    fn encoded_len_matches_canonical_encoding_and_counts_zero_escapes() {
        let key = DocumentKey::new(vec![0; 1_994], Vec::new());
        assert_eq!(key.encoded_len(), 3_992);
        assert_eq!(key.encoded_len(), key.encode().len());
        let oversized = DocumentKey::new(vec![0; 1_995], Vec::new());
        assert_eq!(oversized.encoded_len(), 3_994);
    }
}
