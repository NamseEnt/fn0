use super::*;
use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
use opentelemetry_proto::tonic::logs::v1::ResourceLogs;

fn attribute(key: &str, value: any_value::Value) -> KeyValue {
    KeyValue {
        key: key.to_string(),
        value: Some(AnyValue { value: Some(value) }),
        ..Default::default()
    }
}

fn resource_logs(attributes: Vec<KeyValue>) -> ResourceLogs {
    ResourceLogs {
        resource: Some(Resource {
            attributes,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn named(tenant: &str) -> ResourceLogs {
    resource_logs(vec![attribute(
        TENANT_ATTRIBUTE,
        any_value::Value::StringValue(tenant.to_string()),
    )])
}

fn allow_all() -> TenantPolicy {
    TenantPolicy::disabled()
}

/// A registry serving exactly `tenants`, built the way the control plane
/// builds one: by pushing a policy per tenant.
async fn serving(tenants: &[&str]) -> TenantPolicy {
    let policy = TenantPolicy::enabled_with_clock(crate::clock::Clock::system());
    for tenant in tenants {
        policy
            .push(&TenantId::parse(tenant).unwrap(), 1, "30d", None)
            .await
            .expect("a policy push onboards the tenant");
    }
    policy
}

#[test]
fn a_resource_is_filed_under_the_tenant_its_attribute_names() {
    let request = ExportLogsServiceRequest {
        resource_logs: vec![named("acme")],
    };
    let split = split_logs(request, &allow_all());
    assert_eq!(split.groups.len(), 1);
    assert_eq!(split.groups[0].0.as_str(), "acme");
    assert!(split.dropped.is_empty());
    assert!(
        split.is_intact(),
        "one tenant and nothing dropped still describes the bytes that arrived"
    );
}

/// The three ways a resource fails to name a tenant are counted apart,
/// because they send an operator to three different places.
#[tokio::test]
async fn every_way_of_naming_no_tenant_is_dropped_under_its_own_reason() {
    let request = ExportLogsServiceRequest {
        resource_logs: vec![
            ResourceLogs::default(),
            resource_logs(vec![attribute(
                "service.name",
                any_value::Value::StringValue("checkout".to_string()),
            )]),
            resource_logs(vec![attribute(
                TENANT_ATTRIBUTE,
                any_value::Value::StringValue("not a tenant".to_string()),
            )]),
            resource_logs(vec![attribute(
                TENANT_ATTRIBUTE,
                any_value::Value::IntValue(7),
            )]),
            named("stranger"),
        ],
    };
    let split = split_logs(request, &serving(&["acme"]).await);

    assert!(split.groups.is_empty(), "nothing named a served tenant");
    assert_eq!(split.dropped.no_tenant, 2, "no resource, and no attribute");
    assert_eq!(
        split.dropped.invalid_tenant, 2,
        "unparseable, and not a string"
    );
    assert_eq!(split.dropped.tenant_not_served, 1);
}

/// Grouping is by first appearance, so a request rebuilt from the groups holds
/// its resources in the order they arrived and two runs agree.
#[test]
fn resources_group_by_tenant_in_the_order_the_tenants_first_appear() {
    let request = ExportLogsServiceRequest {
        resource_logs: vec![named("beta"), named("acme"), named("beta")],
    };
    let split = split_logs(request, &allow_all());

    let tenants: Vec<_> = split
        .groups
        .iter()
        .map(|(tenant, _)| tenant.as_str().to_string())
        .collect();
    assert_eq!(tenants, vec!["beta", "acme"]);
    assert_eq!(split.groups[0].1.resource_logs.len(), 2);
    assert_eq!(split.groups[1].1.resource_logs.len(), 1);
    assert!(
        !split.is_intact(),
        "a split request is no longer the bytes that arrived"
    );
}

/// A drop is enough on its own to invalidate the passthrough: the bytes still
/// carry the resource that was thrown away.
#[test]
fn a_drop_alone_ends_the_passthrough() {
    let request = ExportLogsServiceRequest {
        resource_logs: vec![named("acme"), ResourceLogs::default()],
    };
    let split = split_logs(request, &allow_all());
    assert_eq!(split.groups.len(), 1);
    assert_eq!(split.dropped.no_tenant, 1);
    assert!(!split.is_intact());
}

#[test]
fn tenant_not_served_drops_are_aggregated_by_signal_without_tenant_labels() {
    let metrics = crate::metrics::RuntimeMetrics::new();
    Dropped {
        tenant_not_served: 2,
        ..Default::default()
    }
    .record(&metrics, "logs");
    Dropped {
        tenant_not_served: 3,
        ..Default::default()
    }
    .record(&metrics, "traces");
    Dropped {
        tenant_not_served: 5,
        ..Default::default()
    }
    .record(&metrics, "metrics");

    use std::sync::atomic::Ordering::Relaxed;
    assert_eq!(
        metrics
            .ingest_dropped_tenant_not_served
            .load(Relaxed),
        10
    );
    assert_eq!(
        metrics
            .ingest_dropped_tenant_not_served_logs
            .load(Relaxed),
        2
    );
    assert_eq!(
        metrics
            .ingest_dropped_tenant_not_served_traces
            .load(Relaxed),
        3
    );
    assert_eq!(
        metrics
            .ingest_dropped_tenant_not_served_metrics
            .load(Relaxed),
        5
    );
}
