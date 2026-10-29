use serde_json::{Value, json};

pub(super) fn failure_metadata(retained_workload: Option<Value>) -> Value {
    match retained_workload {
        Some(workload) => json!({
            "status": "available_unvalidated_after_later_failure",
            "workload": workload,
        }),
        None => json!({"status": "unavailable_after_early_abort"}),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::failure_metadata;

    #[test]
    fn successful_workload_is_reported_as_available_but_unvalidated() {
        let workload = json!({"requests": 17, "valid_for_oracle": true});
        let metadata = failure_metadata(Some(workload.clone()));

        assert_eq!(
            metadata["status"],
            "available_unvalidated_after_later_failure"
        );
        assert_eq!(metadata["workload"], workload);
    }

    #[test]
    fn failed_or_unreached_workload_is_reported_as_unavailable() {
        assert_eq!(
            failure_metadata(None),
            json!({"status": "unavailable_after_early_abort"})
        );
    }
}
