//! Rewrites pre-ratification WASI 0.3 interface names in an already-built
//! component to the ratified `@0.3.0` spelling.
//!
//! Interface versions are part of the import and export names baked into a
//! component at build time, and wasmtime's semver-compatible name matching
//! deliberately skips prereleases, so a component built against
//! `0.3.0-rc-2026-03-15` cannot link against a host that offers `0.3.0`. Because
//! the two WIT packages are identical apart from one `@since` annotation and a
//! doc-comment URL, the rename is a pure spelling change and lets bundles that
//! were deployed before the bump keep running without being rebuilt.
//!
//! Only sections whose payloads consist of names and indices are touched, plus
//! core module import and export names, which must keep matching the arguments
//! the component instantiates them with. Custom sections carry build metadata
//! that no longer describes the rewritten component, but nothing links against
//! them, and one of them embeds a whole encoded WIT package whose internal
//! framing a byte-level rewrite would break.

const RC_VERSION: &str = "@0.3.0-rc-2026-03-15";
const RATIFIED_VERSION: &str = "@0.3.0";

const COMPONENT_PREAMBLE_LEN: usize = 8;
const COMPONENT_LAYER: [u8; 2] = [0x01, 0x00];

/// Component sections that hold nothing but names and indices.
const COMPONENT_NAME_SECTIONS: &[u8] = &[
    2,  // core instance
    3,  // core type
    5,  // instance
    6,  // alias
    7,  // type
    10, // import
    11, // export
];
const COMPONENT_CORE_MODULE_SECTION: u8 = 1;
const COMPONENT_NESTED_COMPONENT_SECTION: u8 = 4;

/// Core module sections that hold names.
const CORE_NAME_SECTIONS: &[u8] = &[
    2, // import
    7, // export
];

/// Returns the rewritten component, or `None` if `wasm` is not a component or
/// carries no prerelease WASI names.
pub fn rewrite_wasi_030_rc_names(wasm: &[u8]) -> Option<Vec<u8>> {
    if !is_component(wasm) {
        return None;
    }
    if !contains(wasm, RC_VERSION.as_bytes()) {
        return None;
    }
    rewrite_container(wasm, true)
}

pub fn is_component(wasm: &[u8]) -> bool {
    wasm.len() > COMPONENT_PREAMBLE_LEN
        && wasm[0..4] == *b"\0asm"
        && wasm[6..8] == COMPONENT_LAYER
}

/// `None` when nothing inside changed, which keeps the rewrite idempotent: the
/// prerelease version also appears in custom sections this rewrite leaves alone,
/// so "the bytes still mention the prerelease" cannot stand in for "there is
/// still something to rewrite".
fn rewrite_container(bytes: &[u8], is_component_layer: bool) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(&bytes[..COMPONENT_PREAMBLE_LEN]);
    let mut rewrote_any = false;

    let mut cursor = COMPONENT_PREAMBLE_LEN;
    while cursor < bytes.len() {
        let section_id = bytes[cursor];
        let (declared_len, payload_start) = read_leb128(bytes, cursor + 1)?;
        let payload_end = payload_start.checked_add(declared_len)?;
        let payload = bytes.get(payload_start..payload_end)?;

        let rewritten = if is_component_layer && section_id == COMPONENT_CORE_MODULE_SECTION {
            rewrite_container(payload, false)
        } else if is_component_layer && section_id == COMPONENT_NESTED_COMPONENT_SECTION {
            rewrite_container(payload, true)
        } else {
            let name_sections = if is_component_layer {
                COMPONENT_NAME_SECTIONS
            } else {
                CORE_NAME_SECTIONS
            };
            if name_sections.contains(&section_id) {
                rewrite_names(payload)
            } else {
                None
            }
        };

        rewrote_any |= rewritten.is_some();
        let payload = rewritten.as_deref().unwrap_or(payload);
        out.push(section_id);
        write_leb128(&mut out, payload.len());
        out.extend_from_slice(payload);

        cursor = payload_end;
    }

    rewrote_any.then_some(out)
}

/// Rewrites every length-prefixed name in `payload` that carries the prerelease
/// version. A name is recognized by its own length byte, so a match cannot span
/// a name boundary or hit an index that happens to look like text.
fn rewrite_names(payload: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(payload.len());
    let mut cursor = 0;
    let mut rewrote_any = false;

    while cursor < payload.len() {
        if let Some(name) = read_name(payload, cursor)
            && let Some(rewritten) = rewrite_name(name)
        {
            write_leb128(&mut out, rewritten.len());
            out.extend_from_slice(rewritten.as_bytes());
            cursor += 1 + name.len();
            rewrote_any = true;
            continue;
        }
        out.push(payload[cursor]);
        cursor += 1;
    }

    rewrote_any.then_some(out)
}

/// A name whose length fits one LEB128 byte, is entirely printable ASCII, and
/// stays inside `payload`.
fn read_name(payload: &[u8], at: usize) -> Option<&str> {
    let declared_len = *payload.get(at)? as usize;
    if declared_len == 0 || declared_len > 0x7f {
        return None;
    }
    let bytes = payload.get(at + 1..at + 1 + declared_len)?;
    if !bytes.iter().all(|b| b.is_ascii_graphic() || *b == b' ') {
        return None;
    }
    std::str::from_utf8(bytes).ok()
}

fn rewrite_name(name: &str) -> Option<String> {
    if !name.starts_with("wasi:") || !name.contains(RC_VERSION) {
        return None;
    }
    Some(name.replace(RC_VERSION, RATIFIED_VERSION))
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

fn read_leb128(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let mut value = 0usize;
    let mut shift = 0;
    let mut cursor = at;
    loop {
        let byte = *bytes.get(cursor)?;
        cursor += 1;
        value |= ((byte & 0x7f) as usize) << shift;
        if byte & 0x80 == 0 {
            return Some((value, cursor));
        }
        shift += 7;
        if shift > 28 {
            return None;
        }
    }
}

fn write_leb128(out: &mut Vec<u8>, mut value: usize) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return;
        }
        out.push(byte | 0x80);
    }
}
