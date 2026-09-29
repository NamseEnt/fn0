fn encode_exact_field_token(name: &str, value: &str) -> io::Result<Vec<u8>> {
    let name_len = u32::try_from(name.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "field name is too large"))?;
    let value_len = u32::try_from(value.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "field value is too large"))?;
    let capacity = EXACT_FIELD_TOKEN_MAGIC
        .len()
        .checked_add(1 + 4 + name.len() + 4 + value.len())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "field token is too large"))?;
    let mut token = Vec::with_capacity(capacity);
    token.extend_from_slice(EXACT_FIELD_TOKEN_MAGIC);
    token.push(EXACT_FIELD_SCALAR_SCOPE);
    token.extend_from_slice(&name_len.to_le_bytes());
    token.extend_from_slice(name.as_bytes());
    token.extend_from_slice(&value_len.to_le_bytes());
    token.extend_from_slice(value.as_bytes());
    Ok(token)
}

/// Writes the blooms as one length-prefixed section behind the container
/// header. The container keeps its section framing so another section can
/// appear again without a new file.
fn write_index(
    path: &Path,
    rows: &[Row],
    parsed_rows: &[Option<BTreeMap<String, String>>],
    row_group_size: usize,
) -> io::Result<()> {
    let bloom = encode_blooms(rows, parsed_rows, row_group_size)?;
    write_index_sections(path, &bloom)
}

/// The container both writers emit: magic, then the bloom section
/// length-prefixed.
///
/// Shared so a streaming writer cannot lay the file out differently from the
/// batch one.
fn write_index_sections(path: &Path, bloom: &[u8]) -> io::Result<()> {
    let mut buf = Vec::with_capacity(INDEX_MAGIC.len() + 4 + bloom.len());
    buf.extend_from_slice(INDEX_MAGIC);
    let length = u32::try_from(bloom.len())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "index section is too large"))?;
    buf.extend_from_slice(&length.to_le_bytes());
    buf.extend_from_slice(bloom);
    fs::write(path, &buf)?;
    sync_file(path)?;
    crate::page_cache::drop_cache(path);
    Ok(())
}

/// Splits a container back into its bloom section.
fn split_index(buf: &[u8]) -> Result<&[u8], String> {
    if !buf.starts_with(INDEX_MAGIC) {
        return Err("part index file has an unrecognized header".to_string());
    }
    let rest = &buf[INDEX_MAGIC.len()..];
    if rest.len() < 4 {
        return Err("part index file is truncated".to_string());
    }
    let length = u32::from_le_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
    let rest = &rest[4..];
    if rest.len() < length {
        return Err("part index file is truncated".to_string());
    }
    Ok(&rest[..length])
}
