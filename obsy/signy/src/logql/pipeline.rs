/// Why a string is not a duration, without allocating to say so.
///
/// It used to be `String`, and the failure path is the common path: every
/// indexed field value of every written row is speculatively parsed as a
/// duration by [`canonical_index_values`], and almost none of them are one.
/// A heap profile of the 2 GiB soak measured the `format!` behind those
/// refusals at **6.45 GiB allocated in 80 seconds** — 4.8% of the process's
/// whole allocation turnover, spent building messages nobody reads. The
/// message belongs to the four call sites that answer a user, and they build
/// it from this.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationError {
    Empty,
    Invalid,
    OutOfRange,
}

impl std::fmt::Display for DurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty duration",
            Self::Invalid => "invalid duration",
            Self::OutOfRange => "duration is out of range",
        })
    }
}

pub fn parse_duration_ns(input: &str) -> Result<i64, DurationError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(DurationError::Empty);
    }
    let mut total = 0i64;
    let mut pos = 0usize;
    while pos < input.len() {
        let number_start = pos;
        while pos < input.len()
            && (input.as_bytes()[pos].is_ascii_digit() || input.as_bytes()[pos] == b'.')
        {
            pos += 1;
        }
        if number_start == pos {
            return Err(DurationError::Invalid);
        }
        let number = &input[number_start..pos];
        let (unit, multiplier) = [
            ("ns", 1u128),
            ("us", 1_000u128),
            ("µs", 1_000u128),
            ("ms", 1_000_000u128),
            ("s", 1_000_000_000u128),
            ("m", 60_000_000_000u128),
            ("h", 3_600_000_000_000u128),
            ("d", 86_400_000_000_000u128),
            ("w", 604_800_000_000_000u128),
        ]
        .into_iter()
        .find(|(unit, _)| input[pos..].starts_with(unit))
        .ok_or(DurationError::Invalid)?;
        pos += unit.len();
        let component = decimal_duration_component_ns(number, multiplier)?;
        total = total
            .checked_add(component)
            .ok_or(DurationError::OutOfRange)?;
    }
    Ok(total)
}

fn decimal_duration_component_ns(number: &str, multiplier: u128) -> Result<i64, DurationError> {
    let (integer, fraction) = match number.split_once('.') {
        Some((integer, fraction)) if !fraction.contains('.') => (integer, fraction),
        Some(_) => return Err(DurationError::Invalid),
        None => (number, ""),
    };
    if integer.is_empty() && fraction.is_empty() {
        return Err(DurationError::Invalid);
    }
    if !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(DurationError::Invalid);
    }

    let significant_fraction = fraction.trim_end_matches('0');
    let scale = u32::try_from(significant_fraction.len())
        .map_err(|_| DurationError::OutOfRange)?;
    let divisor = 10u128
        .checked_pow(scale)
        .ok_or(DurationError::OutOfRange)?;
    let integer_value = if integer.is_empty() {
        0
    } else {
        integer
            .parse::<u128>()
            .map_err(|_| DurationError::OutOfRange)?
    };
    let fraction_value = if significant_fraction.is_empty() {
        0
    } else {
        significant_fraction
            .parse::<u128>()
            .map_err(|_| DurationError::OutOfRange)?
    };
    let decimal_numerator = integer_value
        .checked_mul(divisor)
        .and_then(|value| value.checked_add(fraction_value))
        .ok_or(DurationError::OutOfRange)?;
    let scaled = decimal_numerator
        .checked_mul(multiplier)
        .ok_or(DurationError::OutOfRange)?;
    let truncated = scaled / divisor;
    i64::try_from(truncated).map_err(|_| DurationError::OutOfRange)
}

/// Merges one parser stage's output into the live field set.
///
/// `shadowed_by_metadata` holds the names the push sent as structured metadata
/// and that are not also stream labels. An extraction under one of those names
/// is **discarded**, because that is what Loki does: measured against
/// `grafana/loki:3.3.2` with a stream pushed with `trace_id` metadata and a
/// line whose JSON also carries `trace_id`, `| json | trace_id="<the JSON
/// value>"` matches nothing, `| json | trace_id="<the metadata value>"`
/// matches, `trace_id_extracted` does not exist, and `line_format
/// "{{.trace_id}}"` renders the metadata value. Loki's `_extracted` suffix
/// applies to a collision with a **stream label** only — there both names
/// survive and both are filterable — which is the case the code below still
/// renames.
fn merge_extracted(
    fields: &mut BTreeMap<String, String>,
    next_suffix: &mut BTreeMap<String, usize>,
    shadowed_by_metadata: &BTreeSet<String>,
    extracted: BTreeMap<String, String>,
) {
    for (name, value) in extracted {
        if shadowed_by_metadata.contains(&name) {
            continue;
        }
        insert_extracted_with_counter(fields, next_suffix, name, value);
    }
}

/// Insert one parser-produced field without losing a value when sanitization
/// or a previous parser maps two names to the same LogQL identifier.
fn insert_extracted_with_counter(
    fields: &mut BTreeMap<String, String>,
    next_suffix: &mut BTreeMap<String, usize>,
    name: String,
    value: String,
) {
    if let std::collections::btree_map::Entry::Vacant(entry) = fields.entry(name.clone()) {
        entry.insert(value);
        observe_extracted_name(next_suffix, &name);
        return;
    }

    let base = format!("{name}_extracted");
    let suffix = next_suffix.entry(base.clone()).or_insert(2);
    let mut candidate = base.clone();
    while fields.contains_key(&candidate) {
        candidate = format!("{base}_{}", *suffix);
        *suffix = (*suffix).saturating_add(1);
    }
    fields.insert(candidate, value);
}

fn observe_extracted_name(next_suffix: &mut BTreeMap<String, usize>, name: &str) {
    if let Some((base, suffix)) = name.rsplit_once("_extracted_")
        && let Ok(suffix) = suffix.parse::<usize>()
    {
        let base = format!("{base}_extracted");
        let next = suffix.saturating_add(1).max(2);
        next_suffix
            .entry(base)
            .and_modify(|current| *current = (*current).max(next))
            .or_insert(next);
    } else if name.ends_with("_extracted") {
        next_suffix.entry(name.to_string()).or_insert(2);
    }
}

fn extract_json(line: &str) -> Result<BTreeMap<String, String>, ()> {
    extract_json_cancellable(line, None).map_err(|_| ())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtractError {
    Parse,
    Cancelled,
}

fn extract_json_cancellable(
    line: &str,
    cancellation: Option<&AtomicBool>,
) -> Result<BTreeMap<String, String>, ExtractError> {
    if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
        return Err(ExtractError::Cancelled);
    }
    // serde_json's parser is not cancellation-aware. Check between chunks so
    // a cancelled query does not spend time entering a large parse, and check
    // again before returning the materialized object.
    for chunk in line.as_bytes().chunks(4096) {
        let _ = chunk;
        if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
            return Err(ExtractError::Cancelled);
        }
    }
    let value: serde_json::Value = serde_json::from_str(line).map_err(|_| ExtractError::Parse)?;
    if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
        return Err(ExtractError::Cancelled);
    }
    // A top-level array is valid JSON that Loki flattens by index, the same as
    // a nested one. Rejecting it here would make `["a","b"]` a parser error
    // while `{"x":["a","b"]}` extracted fine, which is a distinction the line
    // format does not actually make.
    let mut fields = BTreeMap::new();
    let mut next_suffix: BTreeMap<String, usize> = BTreeMap::new();
    match value {
        serde_json::Value::Object(object) => {
            flatten_json("", &object, &mut fields, &mut next_suffix, cancellation)?;
        }
        serde_json::Value::Array(items) => {
            flatten_array("", &items, &mut fields, &mut next_suffix, cancellation)?;
        }
        // A bare scalar line is not an object of fields. `json` on it is a
        // parser error, which is what sets `__error__` and keeps the entry
        // filterable rather than silently field-less.
        _ => return Err(ExtractError::Parse),
    }
    Ok(fields)
}

/// Arrays flatten by index: `{"a":["x","y"]}` yields `a_0` and `a_1`.
///
/// Dropping them silently was the previous behaviour, and it made a field
/// present in the line unqueryable with no indication that it had been skipped.
fn flatten_array(
    prefix: &str,
    items: &[serde_json::Value],
    fields: &mut BTreeMap<String, String>,
    next_suffix: &mut BTreeMap<String, usize>,
    cancellation: Option<&AtomicBool>,
) -> Result<(), ExtractError> {
    for (index, value) in items.iter().enumerate() {
        if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
            return Err(ExtractError::Cancelled);
        }
        let path = if prefix.is_empty() {
            index.to_string()
        } else {
            format!("{prefix}_{index}")
        };
        flatten_value(&path, value, fields, next_suffix, cancellation)?;
    }
    Ok(())
}

fn flatten_value(
    path: &str,
    value: &serde_json::Value,
    fields: &mut BTreeMap<String, String>,
    next_suffix: &mut BTreeMap<String, usize>,
    cancellation: Option<&AtomicBool>,
) -> Result<(), ExtractError> {
    match value {
        serde_json::Value::String(value) => {
            insert_extracted_with_counter(fields, next_suffix, path.to_string(), value.clone());
        }
        serde_json::Value::Bool(value) => {
            insert_extracted_with_counter(fields, next_suffix, path.to_string(), value.to_string());
        }
        serde_json::Value::Number(value) => {
            insert_extracted_with_counter(fields, next_suffix, path.to_string(), value.to_string());
        }
        serde_json::Value::Object(nested) => {
            flatten_json(path, nested, fields, next_suffix, cancellation)?;
        }
        serde_json::Value::Array(items) => {
            flatten_array(path, items, fields, next_suffix, cancellation)?;
        }
        // Extracted as empty rather than dropped, which is what Loki does. The
        // difference is observable: `| field=""` matches a JSON null, and an
        // absent field is a different question from a null one.
        serde_json::Value::Null => {
            insert_extracted_with_counter(fields, next_suffix, path.to_string(), String::new());
        }
    }
    Ok(())
}

fn flatten_json(
    prefix: &str,
    object: &serde_json::Map<String, serde_json::Value>,
    fields: &mut BTreeMap<String, String>,
    next_suffix: &mut BTreeMap<String, usize>,
    cancellation: Option<&AtomicBool>,
) -> Result<(), ExtractError> {
    let mut entries: Vec<_> = object.iter().collect();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    for (name, value) in entries {
        if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
            return Err(ExtractError::Cancelled);
        }
        let name = sanitize_field_name(name);
        let path = if prefix.is_empty() {
            name
        } else {
            format!("{prefix}_{name}")
        };
        flatten_value(&path, value, fields, next_suffix, cancellation)?;
    }
    Ok(())
}

fn sanitize_field_name(name: &str) -> String {
    let mut output = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            if index == 0 && ch.is_ascii_digit() {
                output.push('_');
            }
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    if output.is_empty() {
        "_".to_string()
    } else {
        output
    }
}

fn extract_logfmt(line: &str) -> Result<BTreeMap<String, String>, ()> {
    extract_logfmt_cancellable(line, None).map_err(|_| ())
}

fn extract_logfmt_cancellable(
    line: &str,
    cancellation: Option<&AtomicBool>,
) -> Result<BTreeMap<String, String>, ExtractError> {
    let mut fields = BTreeMap::new();
    let mut source_keys: BTreeMap<String, String> = BTreeMap::new();
    let mut next_suffix: BTreeMap<String, usize> = BTreeMap::new();
    let mut pos = 0usize;
    while pos < line.len() {
        if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
            return Err(ExtractError::Cancelled);
        }
        skip_space(line, &mut pos);
        if pos == line.len() {
            break;
        }
        let key_start = pos;
        while pos < line.len()
            && !line.as_bytes()[pos].is_ascii_whitespace()
            && line.as_bytes()[pos] != b'='
        {
            if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
                return Err(ExtractError::Cancelled);
            }
            pos += 1;
        }
        if key_start == pos || line.as_bytes().get(pos) != Some(&b'=') {
            return Err(ExtractError::Parse);
        }
        let raw_key = line[key_start..pos].to_string();
        let key = sanitize_field_name(&raw_key);
        pos += 1;
        let value = if line.as_bytes().get(pos) == Some(&b'"') {
            if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
                return Err(ExtractError::Cancelled);
            }
            let value = match parse_logfmt_quoted(line, &mut pos, cancellation) {
                Ok(value) => value,
                Err(()) if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) => {
                    return Err(ExtractError::Cancelled);
                }
                Err(()) => return Err(ExtractError::Parse),
            };
            if pos < line.len() && !line.as_bytes()[pos].is_ascii_whitespace() {
                return Err(ExtractError::Parse);
            }
            value
        } else {
            let start = pos;
            while pos < line.len() && !line.as_bytes()[pos].is_ascii_whitespace() {
                if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
                    return Err(ExtractError::Cancelled);
                }
                pos += 1;
            }
            line[start..pos].to_string()
        };
        // Repeated occurrences of the same logfmt key retain the usual
        // last-value-wins behavior. Distinct raw keys that sanitize to the
        // same identifier are preserved under a deterministic suffix.
        if let Some(storage_key) = source_keys.get(&raw_key) {
            fields.insert(storage_key.clone(), value);
        } else {
            let mut storage_key = key;
            if fields.contains_key(&storage_key) {
                let base = format!("{storage_key}_extracted");
                let suffix = next_suffix.entry(base.clone()).or_insert(2);
                if fields.contains_key(&base) {
                    storage_key = format!("{base}_{}", *suffix);
                    *suffix = (*suffix).saturating_add(1);
                } else {
                    storage_key = base;
                }
            }
            let inserted_key = storage_key.clone();
            source_keys.insert(raw_key, inserted_key.clone());
            fields.insert(inserted_key.clone(), value);
            observe_extracted_name(&mut next_suffix, &inserted_key);
        }
    }
    Ok(fields)
}

fn parse_logfmt_quoted(
    input: &str,
    pos: &mut usize,
    cancellation: Option<&AtomicBool>,
) -> Result<String, ()> {
    if input.as_bytes().get(*pos) != Some(&b'"') {
        return Err(());
    }
    *pos += 1;
    let mut value = String::new();
    while *pos < input.len() {
        if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
            return Err(());
        }
        match input.as_bytes()[*pos] {
            b'"' => {
                *pos += 1;
                return Ok(value);
            }
            b'\\' => {
                *pos += 1;
                let escape = *input.as_bytes().get(*pos).ok_or(())?;
                match escape {
                    b'a' => {
                        value.push('\u{7}');
                        *pos += 1;
                    }
                    b'b' => {
                        value.push('\u{8}');
                        *pos += 1;
                    }
                    b'f' => {
                        value.push('\u{c}');
                        *pos += 1;
                    }
                    b'n' => {
                        value.push('\n');
                        *pos += 1;
                    }
                    b'r' => {
                        value.push('\r');
                        *pos += 1;
                    }
                    b't' => {
                        value.push('\t');
                        *pos += 1;
                    }
                    b'v' => {
                        value.push('\u{b}');
                        *pos += 1;
                    }
                    b'"' => {
                        value.push('"');
                        *pos += 1;
                    }
                    b'\\' => {
                        value.push('\\');
                        *pos += 1;
                    }
                    b'x' => {
                        *pos += 1;
                        let codepoint = parse_logfmt_escape_digits(input, pos, 2, 16)?;
                        if codepoint > u8::MAX as u32 {
                            return Err(());
                        }
                        value.push(char::from_u32(codepoint).ok_or(())?);
                    }
                    b'u' => {
                        *pos += 1;
                        let codepoint = parse_logfmt_escape_digits(input, pos, 4, 16)?;
                        value.push(char::from_u32(codepoint).ok_or(())?);
                    }
                    b'U' => {
                        *pos += 1;
                        let codepoint = parse_logfmt_escape_digits(input, pos, 8, 16)?;
                        value.push(char::from_u32(codepoint).ok_or(())?);
                    }
                    b'0'..=b'7' => {
                        let codepoint = parse_logfmt_escape_digits(input, pos, 3, 8)?;
                        if codepoint > u8::MAX as u32 {
                            return Err(());
                        }
                        value.push(char::from_u32(codepoint).ok_or(())?);
                    }
                    _ => return Err(()),
                }
            }
            _ => {
                let ch = input[*pos..].chars().next().ok_or(())?;
                value.push(ch);
                *pos += ch.len_utf8();
            }
        }
    }
    Err(())
}

fn parse_logfmt_escape_digits(
    input: &str,
    pos: &mut usize,
    digits: usize,
    radix: u32,
) -> Result<u32, ()> {
    let end = pos.checked_add(digits).ok_or(())?;
    let encoded = input.get(*pos..end).ok_or(())?;
    if !encoded.is_ascii() {
        return Err(());
    }
    let value = u32::from_str_radix(encoded, radix).map_err(|_| ())?;
    *pos = end;
    Ok(value)
}

/// Values represented in the BTF2 exact-field bloom. The union is deliberate:
/// it permits conservative push-down after either parser without making the
/// immutable part format depend on a query's particular pipeline ordering.
/// The literal substrings every match of `pattern` must contain, for trigram
/// pruning of `|~`.
///
/// Prune-only, so the extraction has one obligation: never return a literal
/// some matching line could lack. A concatenation's literal runs are all
/// mandatory; a group's contents are mandatory when the group is; a
/// repetition's are mandatory when it must occur at least once. An
/// alternation makes nothing mandatory (a literal shared by every branch
/// would be, but that is not computed), and `(?i)` case folding turns
/// literals into classes, so an insensitive pattern naturally extracts
/// nothing. Anything the parser cannot handle extracts nothing.
///
/// Runs under three bytes are dropped: a trigram filter cannot test them.
pub(crate) fn required_regex_literals(pattern: &str) -> Vec<String> {
    let Ok(hir) = regex_syntax::parse(pattern) else {
        return Vec::new();
    };
    let mut literals = Vec::new();
    collect_required_literals(&hir, &mut literals);
    literals.retain(|literal| literal.len() >= 3);
    literals
}

fn collect_required_literals(hir: &regex_syntax::hir::Hir, out: &mut Vec<String>) {
    use regex_syntax::hir::HirKind;
    match hir.kind() {
        HirKind::Literal(literal) => {
            if let Ok(text) = std::str::from_utf8(&literal.0) {
                out.push(text.to_string());
            }
        }
        HirKind::Concat(parts) => {
            // Adjacent literal children are one run in the matched text, and a
            // run long enough to hold a trigram prunes where its split pieces
            // could not.
            let mut run = String::new();
            for part in parts {
                if let HirKind::Literal(literal) = part.kind() {
                    if let Ok(text) = std::str::from_utf8(&literal.0) {
                        run.push_str(text);
                        continue;
                    }
                    // A literal that is not valid UTF-8 on its own ends the
                    // run without joining it.
                    if !run.is_empty() {
                        out.push(std::mem::take(&mut run));
                    }
                    continue;
                }
                if !run.is_empty() {
                    out.push(std::mem::take(&mut run));
                }
                collect_required_literals(part, out);
            }
            if !run.is_empty() {
                out.push(run);
            }
        }
        HirKind::Capture(capture) => collect_required_literals(&capture.sub, out),
        HirKind::Repetition(repetition) if repetition.min >= 1 => {
            collect_required_literals(&repetition.sub, out)
        }
        _ => {}
    }
}

/// What `| json` would extract from this line, or `None` when it would fail —
/// the write-side twin of the query-time stage, for the `_pf:` columns.
///
/// Deliberately `extract_json` itself and not `indexed_parser_fields`: the
/// bloom indexes every sanitization variant of a name, but a column holds one
/// value, and the one that is exact is the one the pipeline will produce.
/// Logfmt is deliberately absent for now; a logfmt extraction keeps its
/// query-time cost until this is extended.
///
/// No fast-path gate on the first byte, on purpose: the column is only exact
/// if this function accepts and rejects *precisely* the lines the query-time
/// stage does, and `| json` also extracts from top-level arrays and tolerates
/// leading whitespace. A cheap pre-check that disagreed on either would turn
/// the two-pass scan's exactness into a wrong answer.
pub(crate) fn parsed_json_fields(line: &str) -> Option<BTreeMap<String, String>> {
    extract_json(line).ok()
}

/// The logfmt half of the parser-visible fields; the json half now rides the
/// `_pf:` parse the writer already ran, so nothing indexes json twice.
pub(crate) fn indexed_logfmt_fields(line: &str) -> BTreeMap<String, Vec<String>> {
    let mut indexed: BTreeMap<String, Vec<String>> = BTreeMap::new();
    // Parser error fields are intentionally not indexed: exact_field_predicates
    // excludes __error__, so indexing those values only adds work and memory.
    if line.contains('=')
        && let Ok(fields) = extract_logfmt(line)
    {
        for (name, value) in fields {
            indexed.entry(name).or_default().push(value);
        }
    }
    indexed
}

/// Returns the representations that the exact-field bloom stores for one
/// scalar value. The raw representation preserves string equality, while the
/// canonical numeric and duration forms make typed equality prune safely.
pub(crate) fn canonical_index_values(value: &str) -> Vec<String> {
    let mut values = vec![value.to_string()];
    if let Ok(number) = Decimal::parse(value) {
        values.push(number.canonical_string());
    }
    if let Ok(duration) = parse_duration_ns(value) {
        values.push(duration.to_string());
    }
    values.sort();
    values.dedup();
    values
}



fn skip_space(line: &str, pos: &mut usize) {
    while *pos < line.len() && line.as_bytes()[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}
