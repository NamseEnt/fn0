use super::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Deployment {
    Deploy {
        subdomain: String,
        code_id: u64,
        code_version: u64,
    },
    Undeploy {
        subdomain: String,
    },
}

impl DocDb {
    pub async fn all_deployments(&self) -> Result<Vec<Deployment>> {
        let mut deployments = vec![];

        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = 'deployments' ORDER BY sk ASC",
                libsql::params!(),
            )
            .await?;

        while let Some(row) = rows.next().await? {
            let json_str: String = row.get(0)?;
            if let Ok(d) = serde_json::from_str(&json_str) {
                deployments.push(d);
            }
        }

        Ok(deployments)
    }

    pub async fn insert_deployment(&self, subdomain: &str, code_id: u64, code_version: u64) -> Result<()> {
        let conn = self.db.connect()?;

        let next_sk: u64 = conn
            .query(
                "SELECT COALESCE(MAX(sk), 0) + 1 FROM docs WHERE pk = 'deployments'",
                libsql::params!(),
            )
            .await?
            .next()
            .await?
            .map(|row| row.get::<u64>(0).unwrap())
            .unwrap_or(1);

        let value = serde_json::to_string(&Deployment::Deploy {
            subdomain: subdomain.to_string(),
            code_id,
            code_version,
        }).unwrap();

        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES ('deployments', ?, ?)",
            libsql::params!(next_sk, value),
        )
        .await?;

        Ok(())
    }

    pub async fn insert_undeployment_with_job(
        &self,
        subdomain: &str,
        job_id: &str,
        job_payload: &str,
    ) -> Result<()> {
        let conn = self.db.connect()?;
        let tx = conn.transaction().await?;

        let next_sk: u64 = tx
            .query(
                "SELECT COALESCE(MAX(sk), 0) + 1 FROM docs WHERE pk = 'deployments'",
                libsql::params!(),
            )
            .await?
            .next()
            .await?
            .map(|row| row.get::<u64>(0).unwrap())
            .unwrap_or(1);

        let value = serde_json::to_string(&Deployment::Undeploy {
            subdomain: subdomain.to_string(),
        })
        .unwrap();

        tx.execute(
            "INSERT INTO docs (pk, sk, value) VALUES ('deployments', ?, ?)",
            libsql::params!(next_sk, value),
        )
        .await?;

        tx.execute(
            "INSERT INTO hq_jobs (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) VALUES (?, 'delete_wasm', ?, 'pending', 0, 3, datetime('now'), datetime('now'))",
            libsql::params!(job_id, job_payload),
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn deployments_after(&self, sk: u64) -> Result<Vec<Deployment>> {
        let mut deployments = vec![];

        let conn = self.db.connect()?;
        let mut rows = conn
            .query(
                "SELECT value FROM docs WHERE pk = 'deployments' AND sk > ? ORDER BY sk ASC",
                libsql::params!(sk),
            )
            .await?;

        while let Some(row) = rows.next().await? {
            let json_str: String = row.get(0)?;
            if let Ok(d) = serde_json::from_str(&json_str) {
                deployments.push(d);
            }
        }

        Ok(deployments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deploy_serde_roundtrip() {
        let deploy = Deployment::Deploy {
            subdomain: "alice-app".to_string(),
            code_id: 1,
            code_version: 2,
        };
        let json = serde_json::to_string(&deploy).unwrap();
        let parsed: Deployment = serde_json::from_str(&json).unwrap();
        match parsed {
            Deployment::Deploy {
                subdomain,
                code_id,
                code_version,
            } => {
                assert_eq!(subdomain, "alice-app");
                assert_eq!(code_id, 1);
                assert_eq!(code_version, 2);
            }
            _ => panic!("expected Deploy"),
        }
    }

    #[test]
    fn undeploy_serde_roundtrip() {
        let undeploy = Deployment::Undeploy {
            subdomain: "alice-app".to_string(),
        };
        let json = serde_json::to_string(&undeploy).unwrap();
        let parsed: Deployment = serde_json::from_str(&json).unwrap();
        match parsed {
            Deployment::Undeploy { subdomain } => {
                assert_eq!(subdomain, "alice-app");
            }
            _ => panic!("expected Undeploy"),
        }
    }

    #[test]
    fn backward_compat_old_deploy_format() {
        let old_json = r#"{"subdomain":"alice-app","code_id":1,"code_version":1}"#;
        let parsed: Deployment = serde_json::from_str(old_json).unwrap();
        match parsed {
            Deployment::Deploy {
                subdomain,
                code_id,
                code_version,
            } => {
                assert_eq!(subdomain, "alice-app");
                assert_eq!(code_id, 1);
                assert_eq!(code_version, 1);
            }
            _ => panic!("old format should parse as Deploy"),
        }
    }

    #[test]
    fn undeploy_only_subdomain_json() {
        let json = r#"{"subdomain":"bob-app"}"#;
        let parsed: Deployment = serde_json::from_str(json).unwrap();
        match parsed {
            Deployment::Undeploy { subdomain } => {
                assert_eq!(subdomain, "bob-app");
            }
            _ => panic!("subdomain-only JSON should parse as Undeploy"),
        }
    }

    #[tokio::test]
    async fn insert_and_query_deployments() {
        let db = DocDb::new_test_db().await.unwrap();

        assert!(db.all_deployments().await.unwrap().is_empty());

        db.insert_deployment("alice-app", 1, 1).await.unwrap();
        db.insert_deployment("bob-app", 2, 1).await.unwrap();

        let all = db.all_deployments().await.unwrap();
        assert_eq!(all.len(), 2);

        let after = db.deployments_after(1).await.unwrap();
        assert_eq!(after.len(), 1);
    }

    #[tokio::test]
    async fn undeployment_with_job_is_atomic() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();

        db.insert_deployment("alice-app", 1, 1).await.unwrap();
        db.insert_undeployment_with_job("alice-app", "job-1", r#"{"s3_key":"alice-app.wasm"}"#)
            .await
            .unwrap();

        let all = db.all_deployments().await.unwrap();
        assert_eq!(all.len(), 2);

        match &all[0] {
            Deployment::Deploy { subdomain, .. } => assert_eq!(subdomain, "alice-app"),
            _ => panic!("first should be Deploy"),
        }
        match &all[1] {
            Deployment::Undeploy { subdomain } => assert_eq!(subdomain, "alice-app"),
            _ => panic!("second should be Undeploy"),
        }

        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.task_name, "delete_wasm");
        assert_eq!(job.id, "job-1");
    }

    #[tokio::test]
    async fn job_lifecycle_claim_complete() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();

        db.insert_undeployment_with_job("app", "j1", "{}").await.unwrap();

        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.id, "j1");

        db.complete_job("j1").await.unwrap();

        assert!(db.claim_job().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn job_lifecycle_retry_then_fail() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();

        db.insert_undeployment_with_job("app", "j2", "{}").await.unwrap();

        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.retry_count, 0);

        db.retry_job("j2").await.unwrap();

        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.retry_count, 1);

        db.fail_job("j2").await.unwrap();

        assert!(db.claim_job().await.unwrap().is_none());
    }
}
