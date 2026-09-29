//! What may be called a label, enforced wherever label names enter.
//!
//! These lived in the Loki push decoder while that ingest existed; they
//! survived it because they were never about the wire. LogQL matches label
//! names with a Prometheus-shaped grammar, and stream labels are persisted as
//! Parquet columns, so both the grammar and the reserved column names are
//! properties of the storage and the query language. The OTLP ingest path
//! needs no call site: its stream labels come only from the fixed promotion
//! list in `otlp_log.rs`, every sanitized member of which is valid here — a
//! test on that module holds the invariant.

pub fn validate_label_name(name: &str) -> Result<(), String> {
    validate_field_name(name)?;
    // Stream labels are persisted as Parquet columns, so these storage column
    // names remain reserved even though LogQL pipeline fields may use them.
    match name {
        "_msg" | "_tenant" | "timestamp_ns" | "structured_metadata" => Err(format!(
            "invalid label name '{}': reserved column name",
            name
        )),
        _ => Ok(()),
    }
}

pub fn validate_field_name(name: &str) -> Result<(), String> {
    let bytes = name.as_bytes();
    if bytes.is_empty() {
        return Err("empty label name".to_string());
    }
    let first = bytes[0];
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return Err(format!(
            "invalid label name '{}': must match [a-zA-Z_][a-zA-Z0-9_]*",
            name
        ));
    }
    for &b in &bytes[1..] {
        if !(b.is_ascii_alphanumeric() || b == b'_') {
            return Err(format!(
                "invalid label name '{}': must match [a-zA-Z_][a-zA-Z0-9_]*",
                name
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_reserved_column_names() {
        assert!(validate_label_name("_msg").is_err());
        assert!(validate_label_name("_tenant").is_err());
        assert!(validate_label_name("timestamp_ns").is_err());
        assert!(validate_label_name("structured_metadata").is_err());
    }

    #[test]
    fn enforces_the_prometheus_shaped_grammar() {
        assert!(validate_label_name("_app").is_ok());
        assert!(validate_label_name("app_2").is_ok());
        assert!(validate_label_name("1app").is_err());
        assert!(validate_label_name("app-name").is_err());
        assert!(validate_label_name("").is_err());
    }
}
