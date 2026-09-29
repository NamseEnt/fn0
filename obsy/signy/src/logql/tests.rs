
    use super::*;

    fn entry(line: &str, metadata: &[(&str, &str)]) -> LogEntry {
        LogEntry {
            timestamp_ns: 1,
            line: line.to_string(),
            structured_metadata: metadata
                .iter()
                .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
                .collect(),
        }
    }

    /// Selector boilerplate for pipeline tests: seeds the field map with the
    /// pairs a row's own attributes would supply.
    fn seeded(pairs: &[(&str, &str)]) -> Labels {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Queries are built through the shared flat grammar — the one parser the
    /// engine has (`docs/QUERY_API.md`).
    fn flat(raw: &str) -> LogQuery {
        crate::query::parse_filter_params(raw, 0, crate::query::LOGS_PARAMS)
            .expect("test query parses")
            .query
    }

    /// Numeric and duration field filters have no flat spelling yet
    /// (`FieldOp::Lt..Gte` are engine capability reserved for a later grammar
    /// extension), so the tests that pin their evaluation construct them
    /// directly.
    fn numeric_filter(name: &str, op: FieldOp, text: &str) -> PipelineStage {
        PipelineStage::Field(FieldFilter {
            name: name.to_string(),
            op,
            value: FieldValue::Number(Decimal::parse(text).expect("test number parses")),
        })
    }

    #[test]
    fn parser_keywords_can_also_be_field_names() {
        let json = flat("parse=json&attr=json=ok");
        assert!(json.matches_entry(&entry(r#"{"json":"ok"}"#, &[])));

        let logfmt = flat("attr=logfmt=value");
        assert!(logfmt.matches_entry(&entry("line", &[("logfmt", "value")])));
    }

    #[test]
    fn json_scalar_extraction_is_ordered_and_metadata_wins_collisions() {
        let query = flat("parse=json&attr=trace_id=metadata&attr=nested_count=2");
        assert!(query.matches_entry(&entry(
            r#"{"nested":{"count":2},"trace_id":"line"}"#,
            &[("trace_id", "metadata")],
        )));

        // An extraction whose name is a pushed structured-metadata key is
        // dropped rather than renamed, which is Loki's rule as measured against
        // 3.3.2: with `foo` and `foo_extracted` both pushed as metadata and a
        // line of `{"foo":"z"}`, `| json` returns the metadata values and none
        // of `foo="z"`, `foo_extracted="z"`, `foo_extracted_2="z"` matches.
        let shadowed = flat("parse=json&attr=foo_extracted_2=z");
        assert!(shadowed.exact_field_predicates().is_empty());
        assert!(!shadowed.matches_entry(&entry(
            r#"{"foo":"z"}"#,
            &[("foo", "x"), ("foo_extracted", "y")],
        )));
        assert!(
            !flat("parse=json&attr=foo=z")
                .matches_entry(&entry(r#"{"foo":"z"}"#, &[("foo", "x")]))
        );

        // A collision with a *stream label* still renames, because there Loki
        // kept both names and both were filterable.
        let mut renamed = entry(r#"{"foo":"z"}"#, &[]);
        let labels: Labels = [
            ("foo".to_string(), "x".to_string()),
            ("foo_extracted".to_string(), "y".to_string()),
        ]
        .into_iter()
        .collect();
        assert!(flat("parse=json").process_entry_with_labels(&labels, &mut renamed));
        assert_eq!(
            renamed.structured_metadata,
            vec![("foo_extracted_2".to_string(), "z".to_string())]
        );

        let sanitized = flat("parse=json&attr=namespace_key=value&attr=_9code=200");
        assert!(sanitized.matches_entry(&entry(r#"{"namespace:key":"value","9code":200}"#, &[],)));

        let storage_reserved = flat("parse=json&attr=_msg=parsed");
        assert!(storage_reserved.matches_entry(&entry(r#"{"_msg":"parsed"}"#, &[],)));

        let structured = flat("attr=structured_metadata=queryable");
        assert!(structured.matches_entry(&entry("line", &[("structured_metadata", "queryable")],)));
    }

    #[test]
    fn parser_sanitization_collisions_do_not_drop_scalar_values() {
        let json =
            extract_json(r#"{"a-b":"dash","a_b":"underscore","a_b_extracted":"native"}"#).unwrap();
        assert_eq!(json.len(), 3);
        assert!(json.values().any(|value| value == "dash"));
        assert!(json.values().any(|value| value == "underscore"));
        assert!(json.values().any(|value| value == "native"));

        let logfmt = extract_logfmt("a-b=dash a_b=underscore a_b=different").unwrap();
        assert_eq!(logfmt.get("a_b"), Some(&"dash".to_string()));
        assert_eq!(logfmt.get("a_b_extracted"), Some(&"different".to_string()));
        assert_eq!(logfmt.len(), 2);
    }

    #[test]
    fn stream_labels_are_pipeline_fields_and_parser_collisions_are_extracted() {
        let query = flat("parse=json&attr=app=api");
        let labels: Labels = [("app".to_string(), "api".to_string())]
            .into_iter()
            .collect();
        let mut parsed = entry(r#"{"app":"line"}"#, &[]);

        assert!(query.process_entry_with_labels(&labels, &mut parsed));
        assert_eq!(
            parsed.structured_metadata,
            vec![("app_extracted".to_string(), "line".to_string())]
        );

        let logfmt = flat("parse=logfmt&attr=app=api");
        let mut parsed = entry("app=line", &[]);
        assert!(logfmt.process_entry_with_labels(&labels, &mut parsed));
        assert_eq!(
            parsed.structured_metadata,
            vec![("app_extracted".to_string(), "line".to_string())]
        );

        let field_only = flat("attr=app=api");
        assert!(field_only.process_entry_with_labels(&labels, &mut entry("line", &[])));
    }

    /// `__error__` is paired with `__error_details__`, so a malformed line is
    /// filterable and its failure is explained rather than silent.
    #[test]
    fn malformed_parsers_set_a_filterable_error_field() {
        let json = flat("parse=json&attr=__error__=JSONParserErr");
        let logfmt = flat("parse=logfmt&attr=__error__=LogfmtParserErr");
        let mut malformed = entry("not json", &[]);
        assert!(json.process_entry(&mut malformed));
        assert_eq!(
            malformed.structured_metadata,
            vec![
                ("__error__".to_string(), "JSONParserErr".to_string()),
                (
                    "__error_details__".to_string(),
                    "line is not valid JSON".to_string()
                ),
            ]
        );
        assert!(logfmt.matches_entry(&entry("missing-equals", &[])));

        let logfmt_error = vec![
            ("__error__".to_string(), "LogfmtParserErr".to_string()),
            (
                "__error_details__".to_string(),
                "line is not valid logfmt".to_string(),
            ),
        ];
        let mut adjacent = entry(r#"level="error"status=500"#, &[]);
        assert!(logfmt.process_entry(&mut adjacent));
        assert_eq!(adjacent.structured_metadata, logfmt_error);
        assert!(
            !flat("parse=logfmt&attr=level=error").matches_entry(&adjacent),
            "fields parsed before a malformed boundary must not leak"
        );

        let mut bad_escape = entry(r#"value="bad\q""#, &[]);
        assert!(logfmt.process_entry(&mut bad_escape));
        assert_eq!(bad_escape.structured_metadata, logfmt_error);
    }

    #[test]
    fn logfmt_extracts_quoted_string_and_duration_scalars() {
        let mut query = flat("parse=logfmt&attr=level=error&attr=message=~hello.*");
        query.stages.push(PipelineStage::Field(FieldFilter {
            name: "elapsed".to_string(),
            op: FieldOp::Lt,
            value: FieldValue::Duration(1_000_000_000),
        }));
        assert!(query.matches_entry(&entry(
            r#"level=error message="hello world" elapsed=250ms"#,
            &[],
        )));
        assert!(query.matches_entry(&entry(
            "level=error\tmessage=\"hello tab\"\telapsed=250ms",
            &[],
        )));
        assert_eq!(
            extract_logfmt(r#"value="quote: \" slash: \\ tab:\t""#)
                .unwrap()
                .get("value")
                .unwrap(),
            "quote: \" slash: \\ tab:\t"
        );
    }

    #[test]
    fn field_regexes_match_whole_values() {
        let exact = flat("attr=level%3D~err");
        let negative = flat("attr=level!~err");
        assert!(exact.matches_entry(&entry("line", &[("level", "err")])));
        assert!(!exact.matches_entry(&entry("line", &[("level", "error")])));
        assert!(!negative.matches_entry(&entry("line", &[("level", "err")])));
        assert!(negative.matches_entry(&entry("line", &[("level", "error")])));
    }

    #[test]
    fn pipeline_fields_are_retained_for_query_evaluation() {
        let query = flat("parse=json&attr=level=error");
        let mut parsed = entry(r#"{"level":"error"}"#, &[]);
        assert!(query.process_entry(&mut parsed));
        assert_eq!(
            parsed.structured_metadata,
            vec![("level".to_string(), "error".to_string())]
        );
    }

    #[test]
    fn duration_parsing_is_exact_and_checked() {
        assert_eq!(
            parse_duration_ns("9007199254740993ns").unwrap(),
            9_007_199_254_740_993
        );
        assert!(parse_duration_ns("9223372036854775808ns").is_err());
        assert_eq!(parse_duration_ns("1.5s250ms").unwrap(), 1_750_000_000);
        assert_eq!(parse_duration_ns("1.9ns").unwrap(), 1);
        assert_eq!(parse_duration_ns("0.5ns").unwrap(), 0);
        assert_eq!(parse_duration_ns("0.9ns0.9ns").unwrap(), 0);

        let mut filter = flat("");
        filter.stages.push(PipelineStage::Field(FieldFilter {
            name: "elapsed".to_string(),
            op: FieldOp::Eq,
            value: FieldValue::Duration(parse_duration_ns("9007199254740993ns").unwrap()),
        }));
        assert!(filter.matches_entry(&entry("line", &[("elapsed", "9007199254740993ns")],)));

        let mut scientific = flat("");
        scientific
            .stages
            .push(numeric_filter("count", FieldOp::Gte, "1e3"));
        assert!(scientific.matches_entry(&entry("line", &[("count", "1000")])));
        assert!(Decimal::parse("NaN").is_err());
        assert!(Decimal::parse("inf").is_err());
    }

    #[test]
    fn numeric_field_comparisons_do_not_round_at_two_to_the_fifty_third() {
        let mut query = flat("");
        query
            .stages
            .push(numeric_filter("value", FieldOp::Neq, "9007199254740993"));
        assert!(query.matches_entry(&entry("line", &[("value", "9007199254740992")],)));
        assert!(!query.matches_entry(&entry("line", &[("value", "9007199254740993")],)));
        assert!(!query.matches_entry(&entry("line", &[("value", "NaN")])));
        assert!(!query.matches_entry(&entry("line", &[("value", "Inf")])));

        let mut decimal = flat("");
        decimal
            .stages
            .push(numeric_filter("value", FieldOp::Eq, "500.0"));
        assert!(decimal.matches_entry(&entry("line", &[("value", "500")])));
        assert!(Decimal::parse("1e-9223372036854775808").is_err());
    }

    /// Arrays flatten by index. Dropping them made a field that is present in
    /// the line unqueryable, with nothing to say it had been skipped.
    #[test]
    fn json_arrays_flatten_by_index() {
        let query = flat("parse=json");
        let mut entry = LogEntry {
            timestamp_ns: 1,
            line: r#"{"tags":["red","green"],"nested":[{"id":7}]}"#.to_string(),
            structured_metadata: vec![],
        };
        assert!(query.process_entry_with_labels(&seeded(&[("a", "b")]), &mut entry));
        let fields: std::collections::BTreeMap<_, _> =
            entry.structured_metadata.iter().cloned().collect();
        assert_eq!(fields["tags_0"], "red");
        assert_eq!(fields["tags_1"], "green");
        assert_eq!(fields["nested_0_id"], "7");
    }

    /// A JSON null extracts as empty rather than vanishing. The difference is
    /// observable: `attr=field=` matches a null, and "absent" is a different
    /// question from "null".
    #[test]
    fn json_null_extracts_as_an_empty_value() {
        let query = flat("parse=json");
        let mut entry = LogEntry {
            timestamp_ns: 1,
            line: r#"{"user":null,"id":1}"#.to_string(),
            structured_metadata: vec![],
        };
        assert!(query.process_entry_with_labels(&seeded(&[("a", "b")]), &mut entry));
        let fields: std::collections::BTreeMap<_, _> =
            entry.structured_metadata.iter().cloned().collect();
        assert_eq!(fields.get("user"), Some(&String::new()));
    }

    /// A top-level array is valid JSON and flattens the same as a nested one.
    #[test]
    fn a_top_level_json_array_extracts() {
        let query = flat("parse=json");
        let mut entry = LogEntry {
            timestamp_ns: 1,
            line: r#"[{"id":1},{"id":2}]"#.to_string(),
            structured_metadata: vec![],
        };
        assert!(query.process_entry_with_labels(&seeded(&[("a", "b")]), &mut entry));
        let fields: std::collections::BTreeMap<_, _> =
            entry.structured_metadata.iter().cloned().collect();
        assert_eq!(fields["0_id"], "1");
        assert_eq!(fields["1_id"], "2");
    }

    /// A bare scalar is not a set of fields. It stays a parser error, which is
    /// what sets `__error__` and keeps the entry filterable rather than
    /// silently field-less.
    #[test]
    fn a_bare_scalar_line_is_still_a_json_parser_error() {
        let query = flat("parse=json");
        let mut entry = LogEntry {
            timestamp_ns: 1,
            line: "42".to_string(),
            structured_metadata: vec![],
        };
        assert!(query.process_entry_with_labels(&seeded(&[("a", "b")]), &mut entry));
        let fields: std::collections::BTreeMap<_, _> =
            entry.structured_metadata.iter().cloned().collect();
        assert_eq!(fields[crate::logql::PARSER_ERROR_FIELD], "JSONParserErr");
    }

/// The contract is one-sided: a returned literal must appear in every match.
/// Each case pairs a pattern with lines that match it, and every extracted
/// literal must be a substring of every one of them; the negative cases pin
/// the conservative bail-outs.
#[test]
fn required_regex_literals_never_name_what_a_match_could_lack() {
    use crate::logql::required_regex_literals;
    let cases: &[(&str, &[&str])] = &[
        ("error.*timeout", &["error timeout", "error, then a timeout"]),
        ("connection (reset|refused)", &["connection reset", "connection refused"]),
        ("(abc)+def", &["abcdef", "abcabcdef"]),
        ("^level=error", &["level=error at start"]),
        ("failed\\d+times", &["failed12times"]),
    ];
    for (pattern, lines) in cases {
        let literals = required_regex_literals(pattern);
        let regex = regex::Regex::new(pattern).unwrap();
        for line in *lines {
            assert!(regex.is_match(line), "case must actually match: {pattern}");
            for literal in &literals {
                assert!(
                    line.contains(literal.as_str()),
                    "literal {literal:?} from {pattern} missing in matching line {line:?}"
                );
            }
        }
    }
    assert_eq!(
        required_regex_literals("error.*timeout"),
        vec!["error".to_string(), "timeout".to_string()]
    );
    assert_eq!(
        required_regex_literals("connection (reset|refused)"),
        vec!["connection ".to_string()],
        "an alternation is mandatory as a whole but no branch's literal is"
    );
    assert!(
        required_regex_literals("(?i)error").is_empty(),
        "case folding removes the literal, so nothing may prune"
    );
    assert!(
        required_regex_literals("(zebra)?stripe").contains(&"stripe".to_string()),
        "an optional group contributes nothing; the mandatory tail still does"
    );
    assert!(
        !required_regex_literals("(zebra)?stripe").contains(&"zebra".to_string()),
        "a min-zero repetition must not be required"
    );
}
