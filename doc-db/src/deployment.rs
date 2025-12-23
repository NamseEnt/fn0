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
