use bytes::Bytes;

use crate::{DibiError, Result, StoredDocument};

const ESCAPED_ZERO: u8 = 0xff;
const TERMINATOR: u8 = 0x00;

pub fn encode_document_key(pk: &str, sk: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(pk.len() + sk.len() + 4);
    encode_component(pk, &mut encoded);
    encode_component(sk, &mut encoded);
    encoded
}

pub fn decode_document_key(encoded: &[u8]) -> Result<(String, String)> {
    let (pk, remainder) = decode_component(encoded)?;
    let (sk, remainder) = decode_component(remainder)?;
    if !remainder.is_empty() {
        return Err(DibiError::CorruptKey(
            "trailing bytes after the second component".to_owned(),
        ));
    }
    Ok((pk, sk))
}

pub fn encode_document_value(document: &StoredDocument) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(8 + document.data.len());
    encoded.extend_from_slice(&document.version.to_be_bytes());
    encoded.extend_from_slice(&document.data);
    encoded
}

pub fn decode_document_value(encoded: &[u8]) -> Result<StoredDocument> {
    let version_bytes: [u8; 8] = encoded
        .get(..8)
        .ok_or_else(|| DibiError::CorruptValue("value is shorter than 8 bytes".to_owned()))?
        .try_into()
        .map_err(|_| DibiError::CorruptValue("invalid version prefix".to_owned()))?;
    Ok(StoredDocument {
        version: i64::from_be_bytes(version_bytes),
        data: Bytes::copy_from_slice(&encoded[8..]),
    })
}

fn encode_component(component: &str, output: &mut Vec<u8>) {
    for byte in component.as_bytes() {
        if *byte == TERMINATOR {
            output.push(TERMINATOR);
            output.push(ESCAPED_ZERO);
        } else {
            output.push(*byte);
        }
    }
    output.push(TERMINATOR);
    output.push(TERMINATOR);
}

fn decode_component(encoded: &[u8]) -> Result<(String, &[u8])> {
    let mut decoded = Vec::new();
    let mut offset = 0;
    while offset < encoded.len() {
        let byte = encoded[offset];
        if byte != TERMINATOR {
            decoded.push(byte);
            offset += 1;
            continue;
        }
        let marker = *encoded
            .get(offset + 1)
            .ok_or_else(|| DibiError::CorruptKey("truncated zero escape".to_owned()))?;
        match marker {
            TERMINATOR => {
                let component = String::from_utf8(decoded).map_err(|error| {
                    DibiError::CorruptKey(format!("component is not UTF-8: {error}"))
                })?;
                return Ok((component, &encoded[offset + 2..]));
            }
            ESCAPED_ZERO => {
                decoded.push(TERMINATOR);
                offset += 2;
            }
            _ => {
                return Err(DibiError::CorruptKey(format!(
                    "invalid zero escape marker {marker:#04x}"
                )));
            }
        }
    }
    Err(DibiError::CorruptKey(
        "component terminator is missing".to_owned(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_key_round_trip_and_ordering() {
        let mut keys = [
            ("한글", "日本語"),
            ("a\0b", ""),
            ("", "😀"),
            ("a", "z"),
            ("a", ""),
            ("a/b", "a&b"),
        ];
        let mut encoded = keys
            .iter()
            .map(|(pk, sk)| encode_document_key(pk, sk))
            .collect::<Vec<_>>();
        keys.sort();
        let expected = keys
            .iter()
            .map(|(pk, sk)| ((*pk).to_owned(), (*sk).to_owned()))
            .collect::<Vec<_>>();
        encoded.sort();
        let decoded = encoded
            .iter()
            .map(|key| decode_document_key(key).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn malformed_key_is_an_error() {
        for malformed in [vec![], vec![0], vec![0, 1], vec![0, 0], vec![b'a', 0, 0, 0]] {
            assert!(decode_document_key(&malformed).is_err());
        }
    }

    #[test]
    fn short_value_is_an_error() {
        assert!(decode_document_value(&[0; 7]).is_err());
    }
}
