//! Cloudflare GraphQL Analytics client for R2 operation counts.
//!
//! The `r2OperationsAdaptiveGroups` dataset is ABR-sampled: every count is an
//! unbiased estimate with a confidence interval, not an exact tally. Callers
//! persist estimate + bounds together (see `ProjectOperationsUsageDoc`); the
//! estimate is the authoritative usage number. Data is retained by Cloudflare
//! for only 31 days, which is why control polls and persists it.

use crate::docs::SampledOperationCount;
use forte_sdk::*;
use serde::Deserialize;

const GRAPHQL_URL: &str = "https://api.cloudflare.com/client/v4/graphql";

// GraphQL adaptive-groups queries cap `limit` at 10000.
const QUERY_LIMIT: usize = 10000;

pub struct CloudflareAnalyticsClient {
    api_token: String,
    account_id: String,
}

pub struct BucketOperationsRow {
    pub bucket_name: String,
    pub action_type: String,
    pub count: SampledOperationCount,
}

pub struct PrefixOperationsRow {
    pub action_type: String,
    pub count: SampledOperationCount,
}

impl CloudflareAnalyticsClient {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            api_token: std::env::var("FN0_CLOUDFLARE_API_TOKEN")
                .map_err(|_| anyhow::anyhow!("FN0_CLOUDFLARE_API_TOKEN not set"))?,
            account_id: std::env::var("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID")
                .map_err(|_| anyhow::anyhow!("FN0_STATIC_ASSET_STORAGE_ACCOUNT_ID not set"))?,
        })
    }

    /// Operation counts for every bucket in the account within the window,
    /// grouped by bucket and S3 action type.
    pub async fn r2_operations_by_bucket(
        &self,
        window_start: DateTime,
        window_end: DateTime,
    ) -> anyhow::Result<Vec<BucketOperationsRow>> {
        let query = r#"query($accountTag: string!, $start: Time!, $end: Time!, $limit: uint64!) {
  viewer {
    accounts(filter: {accountTag: $accountTag}) {
      r2OperationsAdaptiveGroups(
        filter: {datetime_geq: $start, datetime_lt: $end}
        limit: $limit
      ) {
        dimensions { bucketName actionType }
        sum { requests }
        confidence(level: 0.95) {
          sum { requests { estimate lower upper isValid } }
        }
      }
    }
  }
}"#;
        let variables = serde_json::json!({
            "accountTag": self.account_id,
            "start": format_time(window_start),
            "end": format_time(window_end),
            "limit": QUERY_LIMIT,
        });
        let groups = self.query_adaptive_groups(query, variables).await?;
        Ok(groups
            .into_iter()
            .filter_map(|g| {
                let count = g.sampled_count();
                Some(BucketOperationsRow {
                    bucket_name: g.dimensions.bucket_name?,
                    action_type: g.dimensions.action_type,
                    count,
                })
            })
            .collect())
    }

    /// Operation counts within one bucket restricted to keys under `prefix`,
    /// grouped by S3 action type.
    pub async fn r2_operations_for_key_prefix(
        &self,
        bucket_name: &str,
        prefix: &str,
        window_start: DateTime,
        window_end: DateTime,
    ) -> anyhow::Result<Vec<PrefixOperationsRow>> {
        let query = r#"query($accountTag: string!, $start: Time!, $end: Time!, $bucket: string!, $prefix: string!, $limit: uint64!) {
  viewer {
    accounts(filter: {accountTag: $accountTag}) {
      r2OperationsAdaptiveGroups(
        filter: {datetime_geq: $start, datetime_lt: $end, bucketName: $bucket, objectName_like: $prefix}
        limit: $limit
      ) {
        dimensions { actionType }
        sum { requests }
        confidence(level: 0.95) {
          sum { requests { estimate lower upper isValid } }
        }
      }
    }
  }
}"#;
        let variables = serde_json::json!({
            "accountTag": self.account_id,
            "start": format_time(window_start),
            "end": format_time(window_end),
            "bucket": bucket_name,
            "prefix": format!("{prefix}%"),
            "limit": QUERY_LIMIT,
        });
        let groups = self.query_adaptive_groups(query, variables).await?;
        Ok(groups
            .into_iter()
            .map(|g| PrefixOperationsRow {
                action_type: g.dimensions.action_type.clone(),
                count: g.sampled_count(),
            })
            .collect())
    }

    async fn query_adaptive_groups(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> anyhow::Result<Vec<AdaptiveGroup>> {
        let payload = serde_json::to_vec(&serde_json::json!({
            "query": query,
            "variables": variables,
        }))?;
        let req = http::Request::builder()
            .uri(GRAPHQL_URL)
            .method("POST")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .body(payload)?;
        let resp = http::Client::new().send(req).await?;
        let status = resp.status().as_u16();
        let body = resp.into_body().bytes().await.to_vec();
        if !(200..300).contains(&status) {
            anyhow::bail!(
                "graphql analytics query failed (status={status}): {}",
                String::from_utf8_lossy(&body)
            );
        }
        let envelope: GraphqlEnvelope = serde_json::from_slice(&body)?;
        if let Some(errors) = envelope.errors
            && !errors.is_empty()
        {
            anyhow::bail!(
                "graphql analytics query errors: {}",
                errors
                    .iter()
                    .map(|e| e.message.as_str())
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        let mut accounts = envelope
            .data
            .ok_or_else(|| anyhow::anyhow!("graphql analytics: missing data"))?
            .viewer
            .accounts;
        let groups = match accounts.len() {
            0 => anyhow::bail!("graphql analytics: account not found"),
            _ => std::mem::take(&mut accounts[0].r2_operations_adaptive_groups),
        };
        if groups.len() == QUERY_LIMIT {
            tracing::warn!(
                "graphql analytics returned exactly {QUERY_LIMIT} rows; results may be truncated"
            );
        }
        Ok(groups)
    }
}

fn format_time(t: DateTime) -> String {
    t.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[derive(Deserialize)]
struct GraphqlEnvelope {
    data: Option<GraphqlData>,
    errors: Option<Vec<GraphqlError>>,
}

#[derive(Deserialize)]
struct GraphqlError {
    message: String,
}

#[derive(Deserialize)]
struct GraphqlData {
    viewer: GraphqlViewer,
}

#[derive(Deserialize)]
struct GraphqlViewer {
    accounts: Vec<GraphqlAccount>,
}

#[derive(Deserialize)]
struct GraphqlAccount {
    #[serde(rename = "r2OperationsAdaptiveGroups", default)]
    r2_operations_adaptive_groups: Vec<AdaptiveGroup>,
}

#[derive(Deserialize)]
struct AdaptiveGroup {
    dimensions: AdaptiveGroupDimensions,
    confidence: AdaptiveGroupConfidence,
}

#[derive(Deserialize)]
struct AdaptiveGroupDimensions {
    #[serde(rename = "bucketName")]
    bucket_name: Option<String>,
    #[serde(rename = "actionType")]
    action_type: String,
}

#[derive(Deserialize)]
struct AdaptiveGroupConfidence {
    sum: AdaptiveGroupConfidenceSum,
}

#[derive(Deserialize)]
struct AdaptiveGroupConfidenceSum {
    requests: ConfidenceValue,
}

#[derive(Deserialize)]
struct ConfidenceValue {
    estimate: f64,
    lower: f64,
    upper: f64,
    #[serde(rename = "isValid")]
    is_valid: bool,
}

impl AdaptiveGroup {
    fn sampled_count(&self) -> SampledOperationCount {
        let requests = &self.confidence.sum.requests;
        SampledOperationCount {
            estimate: requests.estimate.round().max(0.0) as u64,
            lower_bound: requests.lower.round().max(0.0) as u64,
            upper_bound: requests.upper.round().max(0.0) as u64,
            is_valid: requests.is_valid,
        }
    }
}
