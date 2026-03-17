use bytes::Buf;
use libsql::Row;

use super::*;

#[derive(Clone, Copy)]
pub struct Deployment {
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
            deployments.push(row.into());
        }

        Ok(deployments)
    }

    pub async fn insert_deployment(&self, code_id: u64, code_version: u64) -> Result<()> {
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

        let mut value = Vec::with_capacity(16);
        value.extend_from_slice(&code_id.to_le_bytes());
        value.extend_from_slice(&code_version.to_le_bytes());

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
            deployments.push(row.into());
        }

        Ok(deployments)
    }
}

impl From<Row> for Deployment {
    fn from(row: Row) -> Self {
        let bytes: [u8; 16] = row.get(0).unwrap();
        assert_eq!(bytes.len(), 16);

        let mut cursor = bytes.as_slice();
        Deployment {
            code_id: cursor.get_u64_le(),
            code_version: cursor.get_u64_le(),
        }
    }
}
