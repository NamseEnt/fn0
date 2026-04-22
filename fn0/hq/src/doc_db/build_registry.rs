use super::*;

impl DocDb {
    pub async fn register_build(&self, subdomain: &str, build_id: &str) -> Result<Vec<String>> {
        let conn = self.db.connect()?;
        let tx = conn.transaction().await?;

        let pk = format!("forte_builds:{subdomain}");

        let mut old_ids = Vec::new();
        let mut rows = tx
            .query(
                "SELECT value FROM docs WHERE pk = ? ORDER BY sk ASC",
                libsql::params!(pk.clone()),
            )
            .await?;
        while let Some(row) = rows.next().await? {
            let v: String = row.get(0)?;
            old_ids.push(v);
        }
        drop(rows);

        tx.execute("DELETE FROM docs WHERE pk = ?", libsql::params!(pk.clone()))
            .await?;

        tx.execute(
            "INSERT INTO docs (pk, sk, value) VALUES (?, 1, ?)",
            libsql::params!(pk, build_id.to_string()),
        )
        .await?;

        tx.commit().await?;
        Ok(old_ids)
    }

    pub async fn enqueue_r2_prefix_delete(&self, subdomain: &str, build_id: &str) -> Result<()> {
        let conn = self.db.connect()?;
        let job_id = uuid::Uuid::new_v4().to_string();
        let prefix = format!("{subdomain}/{build_id}/");
        let payload = serde_json::json!({ "prefix": prefix }).to_string();

        conn.execute(
            "INSERT INTO hq_jobs (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) VALUES (?, 'delete_r2_prefix', ?, 'pending', 0, 10, datetime('now'), datetime('now'))",
            libsql::params!(job_id, payload),
        )
        .await?;

        Ok(())
    }

    pub async fn clear_build(&self, subdomain: &str) -> Result<()> {
        let conn = self.db.connect()?;
        let pk = format!("forte_builds:{subdomain}");
        conn.execute("DELETE FROM docs WHERE pk = ?", libsql::params!(pk))
            .await?;
        Ok(())
    }

    pub async fn enqueue_r2_subdomain_delete(&self, subdomain: &str) -> Result<()> {
        let conn = self.db.connect()?;
        let job_id = uuid::Uuid::new_v4().to_string();
        let prefix = format!("{subdomain}/");
        let payload = serde_json::json!({ "prefix": prefix }).to_string();

        conn.execute(
            "INSERT INTO hq_jobs (id, task_name, payload, status, retry_count, max_retries, created_at, updated_at) VALUES (?, 'delete_r2_prefix', ?, 'pending', 0, 10, datetime('now'), datetime('now'))",
            libsql::params!(job_id, payload),
        )
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_build_returns_previous_ids() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();

        let old = db.register_build("alice-app", "build-a").await.unwrap();
        assert!(old.is_empty());

        let old = db.register_build("alice-app", "build-b").await.unwrap();
        assert_eq!(old, vec!["build-a"]);
    }

    #[tokio::test]
    async fn enqueue_r2_prefix_delete_creates_job() {
        let db = DocDb::new_test_db().await.unwrap();
        db.ensure_job_tables().await.unwrap();

        db.enqueue_r2_prefix_delete("alice-app", "build-a")
            .await
            .unwrap();

        let job = db.claim_job().await.unwrap().unwrap();
        assert_eq!(job.task_name, "delete_r2_prefix");
        let v: serde_json::Value = serde_json::from_str(&job.payload).unwrap();
        assert_eq!(v["prefix"], "alice-app/build-a/");
    }
}
