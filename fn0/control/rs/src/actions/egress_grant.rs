use crate::docs::*;
use forte_sdk::*;
use serde::{Deserialize, Serialize};

const GRANT_CONFLICT_RETRIES: usize = 8;

#[derive(Deserialize)]
pub struct Input {
    pub project_id: String,
    pub requested_bytes: u64,
    pub minimum_bytes: u64,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub enum Output {
    Granted { month: String, granted_bytes: u64 },
    QuotaExhausted { month: String },
    QuotaNotConfigured,
    Unauthorized,
    Error,
}

pub async fn handler(req: ForteRequest<'_, Input>) -> Output {
    if req
        .headers
        .get("x-fn0-internal-egress-grant")
        .and_then(|value| value.to_str().ok())
        != Some("true")
    {
        return Output::Unauthorized;
    }
    if req.body.project_id.is_empty() || req.body.minimum_bytes > req.body.requested_bytes {
        return Output::Error;
    }
    match grant_egress(
        &doc_db::database(),
        &req.body.project_id,
        req.body.requested_bytes,
        req.body.minimum_bytes,
        now(),
    )
    .await
    {
        Ok(output) => output,
        Err(error) => {
            tracing::error!(project_id = %req.body.project_id, "egress grant failed: {error:#}");
            Output::Error
        }
    }
}

fn usage_month(current_time: DateTime) -> String {
    current_time.format("%Y-%m").to_string()
}

async fn grant_egress(
    db: &doc_db::Database,
    project_id: &str,
    requested_bytes: u64,
    minimum_bytes: u64,
    current_time: DateTime,
) -> anyhow::Result<Output> {
    let month = usage_month(current_time);
    for attempt_number in 0..GRANT_CONFLICT_RETRIES {
        let project_id = project_id.to_string();
        let month_for_trx = month.clone();
        let result = db
            .trx(|trx| {
                let project_id = project_id.clone();
                let month = month_for_trx.clone();
                async move {
                    let Some(quota) = trx
                        .get(ProjectEgressQuotaDocGet {
                            project_id: project_id.as_str(),
                        })
                        .await?
                    else {
                        return trx.commit::<_, ()>(Output::QuotaNotConfigured);
                    };
                    let usage = trx
                        .get(ProjectEgressUsageDocGet {
                            project_id: project_id.as_str(),
                            month: month.as_str(),
                        })
                        .await?;
                    let already_granted = usage.as_ref().map_or(0, |usage| usage.granted_bytes);
                    let remaining_bytes = match quota.monthly_egress_limit {
                        MonthlyEgressLimit::Bytes(limit) => limit.saturating_sub(already_granted),
                        MonthlyEgressLimit::Unlimited => u64::MAX - already_granted,
                    };
                    if remaining_bytes == 0 || remaining_bytes < minimum_bytes {
                        return trx.commit::<_, ()>(Output::QuotaExhausted { month });
                    }
                    let granted_bytes = requested_bytes.min(remaining_bytes);
                    match usage {
                        Some(mut usage) => usage.granted_bytes += granted_bytes,
                        None => {
                            trx.create(ProjectEgressUsageDoc {
                                project_id,
                                month: month.clone(),
                                granted_bytes,
                            })?;
                        }
                    }
                    trx.commit::<_, ()>(Output::Granted {
                        month,
                        granted_bytes,
                    })
                }
            })
            .await;
        match result {
            doc_db::TrxResult::Committed(output) => return Ok(output),
            doc_db::TrxResult::Cancelled(()) => unreachable!(),
            doc_db::TrxResult::Conflict(error) if attempt_number + 1 < GRANT_CONFLICT_RETRIES => {
                tracing::debug!(?error, "egress grant conflict retry");
            }
            doc_db::TrxResult::Conflict(error) => {
                anyhow::bail!("egress grant conflict: {error:?}")
            }
            doc_db::TrxResult::Err(error) => return Err(error),
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::{Output, grant_egress, usage_month};
    use crate::docs::{
        DbRequest, MonthlyEgressLimit, ProjectEgressQuotaDoc, ProjectEgressQuotaDocPut,
        ProjectEgressUsageDocGet,
    };
    use forte_sdk::{DateTime, chrono};

    fn september() -> DateTime {
        chrono::DateTime::parse_from_rfc3339("2026-09-15T12:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc)
    }

    async fn configure(db: &doc_db::Database, project_id: &str, limit: MonthlyEgressLimit) {
        ProjectEgressQuotaDocPut(ProjectEgressQuotaDoc {
            project_id: project_id.to_string(),
            monthly_egress_limit: limit,
        })
        .send_with(db)
        .await
        .unwrap();
    }

    async fn granted_total(db: &doc_db::Database, project_id: &str, month: &str) -> u64 {
        (ProjectEgressUsageDocGet { project_id, month })
            .send_with(db)
            .await
            .unwrap()
            .map_or(0, |usage| usage.granted_bytes)
    }

    #[test]
    fn project_without_quota_document_is_refused() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            let output = grant_egress(&db, "project", 100, 1, september())
                .await
                .unwrap();
            assert_eq!(output, Output::QuotaNotConfigured);
        });
    }

    #[test]
    fn grants_up_to_the_exact_limit_then_refuses() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            configure(&db, "project", MonthlyEgressLimit::Bytes(1_000)).await;
            assert_eq!(
                grant_egress(&db, "project", 600, 1, september())
                    .await
                    .unwrap(),
                Output::Granted {
                    month: "2026-09".to_string(),
                    granted_bytes: 600
                }
            );
            assert_eq!(
                grant_egress(&db, "project", 600, 1, september())
                    .await
                    .unwrap(),
                Output::Granted {
                    month: "2026-09".to_string(),
                    granted_bytes: 400
                }
            );
            assert_eq!(
                grant_egress(&db, "project", 1, 1, september())
                    .await
                    .unwrap(),
                Output::QuotaExhausted {
                    month: "2026-09".to_string()
                }
            );
            assert_eq!(granted_total(&db, "project", "2026-09").await, 1_000);
        });
    }

    #[test]
    fn partial_remaining_below_minimum_is_refused_without_charging() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            configure(&db, "project", MonthlyEgressLimit::Bytes(100)).await;
            grant_egress(&db, "project", 90, 90, september())
                .await
                .unwrap();
            assert_eq!(
                grant_egress(&db, "project", 50, 20, september())
                    .await
                    .unwrap(),
                Output::QuotaExhausted {
                    month: "2026-09".to_string()
                }
            );
            assert_eq!(granted_total(&db, "project", "2026-09").await, 90);
        });
    }

    #[test]
    fn budgets_are_isolated_per_project_and_per_month() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            configure(&db, "first", MonthlyEgressLimit::Bytes(10)).await;
            configure(&db, "second", MonthlyEgressLimit::Bytes(10)).await;
            grant_egress(&db, "first", 10, 10, september())
                .await
                .unwrap();
            assert!(matches!(
                grant_egress(&db, "second", 10, 10, september())
                    .await
                    .unwrap(),
                Output::Granted { .. }
            ));
            let october = september() + chrono::Duration::days(20);
            assert_eq!(usage_month(october), "2026-10");
            assert_eq!(
                grant_egress(&db, "first", 10, 10, october).await.unwrap(),
                Output::Granted {
                    month: "2026-10".to_string(),
                    granted_bytes: 10
                }
            );
        });
    }

    #[test]
    fn unlimited_quota_always_grants_the_request() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            configure(&db, "project", MonthlyEgressLimit::Unlimited).await;
            for _ in 0..3 {
                assert!(matches!(
                    grant_egress(&db, "project", 1 << 40, 1, september())
                        .await
                        .unwrap(),
                    Output::Granted {
                        granted_bytes: 1_099_511_627_776,
                        ..
                    }
                ));
            }
        });
    }

    #[test]
    fn concurrent_grants_never_exceed_the_limit() {
        futures::executor::block_on(async {
            let db = doc_db::memory();
            configure(&db, "project", MonthlyEgressLimit::Bytes(1_000)).await;
            let grants = (0..16).map(|_| grant_egress(&db, "project", 100, 100, september()));
            let outputs = futures::future::join_all(grants).await;
            let granted_sum: u64 = outputs
                .into_iter()
                .map(|output| match output {
                    Ok(Output::Granted { granted_bytes, .. }) => granted_bytes,
                    _ => 0,
                })
                .sum();
            assert_eq!(granted_sum, 1_000);
            assert_eq!(granted_total(&db, "project", "2026-09").await, 1_000);
        });
    }
}
