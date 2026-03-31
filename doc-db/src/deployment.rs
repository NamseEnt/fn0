use super::*;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Deployment {
    pub subdomain: String,
    pub code_id: u64,
    pub code_version: u64,
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

        let value = serde_json::to_string(&Deployment {
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
