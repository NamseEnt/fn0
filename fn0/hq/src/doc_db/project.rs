use super::*;

pub struct ProjectInfo {
    pub code_id: u64,
    pub subdomain: String,
}

impl DocDb {
    pub async fn get_project(
        &self,
        github_username: &str,
        project_name: &str,
    ) -> Result<Option<ProjectInfo>> {
        let conn = self.db.connect()?;
        let pk = format!("project:{github_username}/{project_name}");
        let subdomain = format!("{github_username}-{project_name}");

        let existing = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params!(pk),
            )
            .await?
            .next()
            .await?;

        match existing {
            Some(row) => {
                let bytes: Vec<u8> = row.get(0)?;
                let code_id = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                Ok(Some(ProjectInfo { code_id, subdomain }))
            }
            None => Ok(None),
        }
    }

    pub async fn get_or_create_project(
        &self,
        github_username: &str,
        project_name: &str,
    ) -> Result<ProjectInfo> {
        let conn = self.db.connect()?;
        let pk = format!("project:{github_username}/{project_name}");
        let subdomain = format!("{github_username}-{project_name}");

        let existing = conn
            .query(
                "SELECT value FROM docs WHERE pk = ? AND sk = 0",
                libsql::params!(pk.clone()),
            )
            .await?
            .next()
            .await?;

        if let Some(row) = existing {
            let bytes: Vec<u8> = row.get(0)?;
            let code_id = u64::from_le_bytes(bytes[..8].try_into().unwrap());
            return Ok(ProjectInfo { code_id, subdomain });
        }

        let code_id: u64 = conn
            .query(
                "SELECT COALESCE(MAX(sk), 0) + 1 FROM docs WHERE pk = 'next_code_id'",
                libsql::params!(),
            )
            .await?
            .next()
            .await?
            .map(|row| row.get::<u64>(0).unwrap())
            .unwrap_or(1);

        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES ('next_code_id', ?, X'')",
            libsql::params!(code_id),
        )
        .await?;

        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES (?, 0, ?)",
            libsql::params!(pk, code_id.to_le_bytes().to_vec()),
        )
        .await?;

        Ok(ProjectInfo { code_id, subdomain })
    }

    pub async fn next_code_version(&self, code_id: u64) -> Result<u64> {
        let conn = self.db.connect()?;
        let pk = format!("code_version:{code_id}");

        let version: u64 = conn
            .query(
                "SELECT COALESCE(MAX(sk), 0) + 1 FROM docs WHERE pk = ?",
                libsql::params!(pk.clone()),
            )
            .await?
            .next()
            .await?
            .map(|row| row.get::<u64>(0).unwrap())
            .unwrap_or(1);

        conn.execute(
            "INSERT INTO docs (pk, sk, value) VALUES (?, ?, X'')",
            libsql::params!(pk, version),
        )
        .await?;

        Ok(version)
    }
}
