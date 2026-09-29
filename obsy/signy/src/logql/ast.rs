#[derive(Debug, Clone, PartialEq)]
pub enum MatcherOp {
    Eq,
    Neq,
    Re,
    NRe,
}

#[derive(Debug, Clone)]
pub struct LabelMatcher {
    pub name: String,
    pub op: MatcherOp,
    pub value: String,
    regex: Option<Regex>,
}

impl LabelMatcher {
    pub fn new(name: String, op: MatcherOp, value: String) -> Result<Self, String> {
        let regex = match op {
            MatcherOp::Re | MatcherOp::NRe => Some(
                Regex::new(&format!("^(?:{})$", value))
                    .map_err(|e| format!("invalid regular expression '{value}': {e}"))?,
            ),
            MatcherOp::Eq | MatcherOp::Neq => None,
        };
        Ok(Self {
            name,
            op,
            value,
            regex,
        })
    }

    pub fn matches(&self, labels: &Labels) -> bool {
        let actual = labels.get(&self.name).map(String::as_str).unwrap_or("");
        match self.op {
            MatcherOp::Eq => actual == self.value,
            MatcherOp::Neq => actual != self.value,
            MatcherOp::Re => self.regex.as_ref().is_some_and(|re| re.is_match(actual)),
            MatcherOp::NRe => self.regex.as_ref().is_none_or(|re| !re.is_match(actual)),
        }
    }
}

#[derive(Debug, Clone)]

pub enum PipelineStage {
    Line(LineFilter),
    Json,
    Logfmt,
    Field(FieldFilter),
}

#[derive(Debug, Clone)]
pub struct LogQuery {
    pub matchers: Vec<LabelMatcher>,
    pub line_filters: Vec<LineFilter>,
    pub stages: Vec<PipelineStage>,
}

impl LogQuery {
    pub fn exact_field_predicates(&self) -> Vec<ExactFieldPredicate> {
        let mut parser_seen = false;
        // Whether `| json` over the *stored* line is the only way this
        // predicate's field can have been extracted: no logfmt stage, and no
        // `line_format` having rewritten the line a later parser would read.
        let mut json_only = true;
        let mut predicates = Vec::new();
        // A selector equality is a statement about the row's own pushed
        // attributes — the strongest predicate a scan can prune on, evaluated
        // before any parser can synthesize a field.
        for matcher in &self.matchers {
            if matcher.op == MatcherOp::Eq && !matcher.value.is_empty() {
                predicates.push(ExactFieldPredicate::new(
                    matcher.name.clone(),
                    matcher.value.clone(),
                ));
            }
        }
        for stage in &self.stages {
            match stage {
                PipelineStage::Json => parser_seen = true,
                PipelineStage::Logfmt => {
                    parser_seen = true;
                    json_only = false;
                }
                PipelineStage::Field(FieldFilter {
                    name,
                    op: FieldOp::Eq,
                    value,
                }) if name != PARSER_ERROR_FIELD
                    && name != PARSER_ERROR_DETAILS_FIELD
                    && (!parser_seen || !may_be_synthesized_extracted_name(name)) =>
                {
                    let (value, canonical) = match value {
                        FieldValue::String(value) => (value.clone(), false),
                        FieldValue::Number(value) => (value.canonical_string(), true),
                        FieldValue::Duration(value) => (value.to_string(), true),
                        FieldValue::Regex(_) => continue,
                    };
                    let mut predicate = if canonical {
                        ExactFieldPredicate::new_canonical_with_extraction(
                            name.clone(),
                            value,
                            parser_seen,
                        )
                    } else {
                        ExactFieldPredicate::new_with_extraction(name.clone(), value, parser_seen)
                    };
                    predicate.json_only_extraction = parser_seen && json_only;
                    predicates.push(predicate);
                }
                PipelineStage::Line(_) | PipelineStage::Field(_) => {}
            }
        }
        predicates
    }

    #[cfg(test)]
    pub fn matches_entry(&self, entry: &LogEntry) -> bool {
        self.process_entry(&mut entry.clone())
    }

    /// Evaluates the ordered pipeline and preserves the fields visible to the
    /// pipeline on the returned query-local entry. This includes fields
    /// extracted by json/logfmt and the synthetic `__error__` field.
    #[cfg(test)]
    pub fn process_entry(&self, entry: &mut LogEntry) -> bool {
        self.process_entry_with_labels(&BTreeMap::new(), entry)
    }

    /// Evaluates the pipeline with stream labels available as its initial
    /// field set. Labels are canonical for colliding names, so a parser
    /// extraction with the same name is retained as `<name>_extracted`.
    /// An unchanged label is not retained as query-local structured metadata;
    /// one a `label_format` rewrote is.
    #[cfg(test)]
    pub fn process_entry_with_labels(&self, labels: &Labels, entry: &mut LogEntry) -> bool {
        self.process_entry_with_labels_cancellable(labels, entry, None)
            .unwrap_or(false)
    }

    pub fn process_entry_with_labels_cancellable(
        &self,
        labels: &Labels,
        entry: &mut LogEntry,
        cancellation: Option<&AtomicBool>,
    ) -> Result<bool, String> {
        self.process_entry_with_precomputed_json(labels, entry, cancellation, None)
    }

    /// The pipeline with the `| json` extraction optionally supplied by the
    /// caller — the storage's `_pf:` columns hold exactly what
    /// `extract_json` would produce, so a scan that already decoded them can
    /// spare the per-row parse. `Some` is only sound when the map is known
    /// complete (the part's parsed-key list under its cap) and non-empty
    /// (an empty reconstruction cannot distinguish a parse failure, which
    /// must set `__error__`, from a line with no scalar fields).
    pub fn process_entry_with_precomputed_json(
        &self,
        labels: &Labels,
        entry: &mut LogEntry,
        cancellation: Option<&AtomicBool>,
        precomputed_json: Option<&BTreeMap<String, String>>,
    ) -> Result<bool, String> {
        let mut fields: BTreeMap<String, String> = labels.clone();
        let mut next_extracted_suffix = BTreeMap::new();
        for name in fields.keys() {
            observe_extracted_name(&mut next_extracted_suffix, name);
        }
        // The names a parser stage may not extract under, because structured
        // metadata outranks an extraction of the same name on Loki. See
        // `merge_extracted`. A name that is *also* a stream label is not in
        // here: there the extraction survives as `<name>_extracted`.
        let mut shadowed_by_metadata: BTreeSet<String> = BTreeSet::new();
        for (name, value) in &entry.structured_metadata {
            // A stream label is the stable value visible to the pipeline.
            // Structured metadata with another name remains queryable.
            if !labels.contains_key(name) {
                shadowed_by_metadata.insert(name.clone());
            }
            fields.entry(name.clone()).or_insert_with(|| value.clone());
            observe_extracted_name(&mut next_extracted_suffix, name);
        }
        // The selector is the pipeline's leading filter now that no stream
        // exists to match it against: it reads the same field map the stages
        // do — the row's pushed attributes, before any parser runs.
        for matcher in &self.matchers {
            if !matcher.matches(&fields) {
                return Ok(false);
            }
        }
        for stage in &self.stages {
            if cancellation.is_some_and(|flag| flag.load(AtomicOrdering::Acquire)) {
                return Err("query timed out".to_string());
            }
            match stage {
                PipelineStage::Line(filter) => {
                    if !filter.matches(&entry.line) {
                        return Ok(false);
                    }
                }
                PipelineStage::Json if precomputed_json.is_some() => {
                    merge_extracted(
                        &mut fields,
                        &mut next_extracted_suffix,
                        &shadowed_by_metadata,
                        precomputed_json.cloned().unwrap_or_default(),
                    )
                }
                PipelineStage::Json => match extract_json_cancellable(&entry.line, cancellation) {
                    Ok(extracted) => {
                        merge_extracted(
                            &mut fields,
                            &mut next_extracted_suffix,
                            &shadowed_by_metadata,
                            extracted,
                        )
                    }
                    Err(ExtractError::Parse) => {
                        set_parser_error(entry, &mut fields, "JSONParserErr", "line is not valid JSON");
                    }
                    Err(ExtractError::Cancelled) => return Err("query timed out".to_string()),
                },
                PipelineStage::Logfmt => {
                    match extract_logfmt_cancellable(&entry.line, cancellation) {
                        Ok(extracted) => {
                            merge_extracted(
                            &mut fields,
                            &mut next_extracted_suffix,
                            &shadowed_by_metadata,
                            extracted,
                        )
                        }
                        Err(ExtractError::Parse) => {
                            set_parser_error(
                                entry,
                                &mut fields,
                                "LogfmtParserErr",
                                "line is not valid logfmt",
                            );
                        }
                        Err(ExtractError::Cancelled) => return Err("query timed out".to_string()),
                    }
                }
                PipelineStage::Field(filter) => {
                    if !filter.matches(&fields) {
                        return Ok(false);
                    }
                }
            }
        }
        // `entry` is a query-local clone, so retaining the evaluated fields
        // here lets metric evaluation use the same label set that filtering
        // used without changing the persisted log entry.
        //
        // A stream label is dropped only while it still *equals* the stream
        // label — carrying an identical copy would say nothing. A
        // `label_format` that rewrote one is kept, because the row's label set
        // genuinely changed and the response has to show the new value: Loki
        // 3.3.2 answers `{app="probe"} | label_format level="rewritten"` with
        // `level="rewritten"`, not with the stored value.
        entry.structured_metadata = fields
            .into_iter()
            .filter(|(name, value)| labels.get(name) != Some(value))
            .collect();
        Ok(true)
    }
}

fn may_be_synthesized_extracted_name(name: &str) -> bool {
    if name.ends_with("_extracted") {
        return true;
    }
    name.rsplit_once("_extracted_").is_some_and(|(_, suffix)| {
        !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn set_parser_error(
    entry: &mut LogEntry,
    fields: &mut BTreeMap<String, String>,
    error: &str,
    details: &str,
) {
    fields.insert(PARSER_ERROR_FIELD.to_string(), error.to_string());
    fields.insert(PARSER_ERROR_DETAILS_FIELD.to_string(), details.to_string());
    entry
        .structured_metadata
        .retain(|(name, _)| name != PARSER_ERROR_FIELD && name != PARSER_ERROR_DETAILS_FIELD);
    entry
        .structured_metadata
        .push((PARSER_ERROR_FIELD.to_string(), error.to_string()));
    entry
        .structured_metadata
        .push((PARSER_ERROR_DETAILS_FIELD.to_string(), details.to_string()));
}

