use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use doc_db::{DodbConfig, DodbConnection, RawStatement, RawTransactionOutcome, Value};
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Cursor;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

const MAX_SNAPSHOT_PAGE_DATA_BYTES: usize = 8 * 1024 * 1024;
use dodb_client::DodbConnection as NativeDodbConnection;
use dodb_core::{DocumentKey, TenantId, TransactionMutation, TransactionRequest};
use dodb_protocol::ProtocolLimits;
use dodb_service::Request as DodbRequest;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Row {
    pub pk: String,
    pub sk: String,
    pub data: Vec<u8>,
}

impl Row {
    fn key(&self) -> (&str, &str) {
        (&self.pk, &self.sk)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SourcePage {
    Rows(Vec<Row>),
    MissingTable,
}

#[async_trait]
pub trait MigrationSource: Send + Sync {
    async fn page(
        &self,
        project_id: &str,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<SourcePage>;
}

#[async_trait]
trait BatchDestination: Send + Sync {
    async fn transact(&self, request: TransactionRequest) -> Result<()>;
    async fn get(&self, row: &Row) -> Result<Option<Vec<u8>>>;
}

#[async_trait]
impl BatchDestination for dodb_client::DodbClient {
    async fn transact(&self, request: TransactionRequest) -> Result<()> {
        dodb_client::DodbClient::transact(self, request)
            .await
            .map(|_| ())
            .map_err(anyhow::Error::new)
    }

    async fn get(&self, row: &Row) -> Result<Option<Vec<u8>>> {
        match dodb_client::DodbClient::get(
            self,
            DocumentKey::new(row.pk.as_bytes().to_vec(), row.sk.as_bytes().to_vec()),
        )
        .await?
        {
            dodb_core::RevisionState::Present { value, .. } => Ok(Some(value)),
            dodb_core::RevisionState::Missing { .. } => Ok(None),
        }
    }
}

#[derive(Clone)]
pub struct TursoSource {
    group_token: String,
    host_suffix: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SnapshotManifest {
    pub format_version: u32,
    pub captured_at: String,
    pub project_ids: Vec<String>,
    pub active_project_ids: Vec<String>,
    pub databases: Vec<SnapshotDatabase>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SnapshotDatabase {
    pub project_id: String,
    pub filename: String,
    pub source_state: String,
    pub row_count: u64,
    pub data_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct SnapshotSource {
    directory: PathBuf,
    manifest: SnapshotManifest,
}

impl SnapshotSource {
    pub async fn open(directory: impl AsRef<Path>) -> Result<Self> {
        let directory = directory.as_ref().to_owned();
        let manifest_path = directory.join("manifest.json");
        let manifest: SnapshotManifest = serde_json::from_slice(
            &std::fs::read(&manifest_path)
                .with_context(|| format!("failed to read {}", manifest_path.display()))?,
        )
        .context("invalid snapshot manifest")?;
        if manifest.format_version != 1 || manifest.captured_at.is_empty() {
            bail!("unsupported or malformed snapshot manifest")
        }
        if manifest_path.is_symlink() || !manifest_path.is_file() {
            bail!("snapshot manifest must be a regular file")
        }
        if manifest.project_ids.first().map(String::as_str) != Some("fn0-control") {
            bail!("snapshot manifest project list must start with fn0-control")
        }
        let mut expected_projects = vec!["fn0-control".to_owned()];
        let mut canonical = BTreeSet::new();
        for project_id in &manifest.active_project_ids {
            tenant_id(project_id)
                .with_context(|| format!("invalid snapshot project ID {project_id:?}"))?;
            if project_id == "fn0-control" || !canonical.insert(project_id.clone()) {
                bail!("duplicate or noncanonical active project ID {project_id:?}")
            }
        }
        expected_projects.extend(canonical);
        if manifest.project_ids != expected_projects {
            bail!("snapshot manifest project IDs do not match its active project list")
        }
        if manifest.databases.len() != manifest.project_ids.len() {
            bail!("snapshot manifest database count does not match project IDs")
        }
        for (project_id, database) in manifest.project_ids.iter().zip(&manifest.databases) {
            if database.project_id != *project_id
                || database.filename != format!("{project_id}.sqlite")
            {
                bail!("snapshot manifest database entry is not canonical for {project_id}")
            }
            if !matches!(
                database.source_state.as_str(),
                "present" | "missing_database"
            ) || (database.source_state == "missing_database"
                && (database.row_count != 0 || database.data_bytes != 0))
            {
                bail!("snapshot source state or counts are invalid for {project_id}")
            }
            tenant_id(project_id)?;
            let path = directory.join(&database.filename);
            if path.is_symlink() || !path.is_file() {
                bail!("snapshot database must be a regular file for {project_id}")
            }
            let actual_hash = file_sha256(&path)?;
            if actual_hash != database.sha256 {
                bail!("snapshot checksum mismatch for {project_id}")
            }
            let db = libsql::Builder::new_local(&path)
                .flags(libsql::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .build()
                .await?;
            let connection = db.connect()?;
            let mut integrity = connection.query("PRAGMA integrity_check", ()).await?;
            let integrity = integrity
                .next()
                .await?
                .ok_or_else(|| anyhow!("missing integrity result for {project_id}"))?
                .get::<String>(0)?;
            if integrity != "ok" {
                bail!("snapshot SQLite integrity check failed for {project_id}: {integrity}")
            }
            let columns = connection.query("PRAGMA table_info(docs)", ()).await?;
            let mut found = BTreeSet::new();
            let mut columns = columns;
            while let Some(column) = columns.next().await? {
                found.insert(column.get::<String>(1)?);
            }
            if !found.is_empty()
                && !["pk", "sk", "data", "version"]
                    .iter()
                    .all(|name| found.contains(*name))
            {
                bail!("snapshot docs table has invalid columns for {project_id}")
            }
            let counts = connection
                .query(
                    "SELECT COUNT(*), COALESCE(SUM(length(data)), 0) FROM docs",
                    (),
                )
                .await;
            match counts {
                Ok(mut counts) => {
                    let row = counts
                        .next()
                        .await?
                        .ok_or_else(|| anyhow!("missing count row for {project_id}"))?;
                    if row.get::<i64>(0)? as u64 != database.row_count
                        || row.get::<i64>(1)? as u64 != database.data_bytes
                    {
                        bail!("snapshot manifest row or byte count mismatch for {project_id}")
                    }
                }
                Err(error)
                    if is_missing_docs_table_error(&error.to_string())
                        && database.row_count == 0
                        && database.data_bytes == 0 => {}
                Err(error) => return Err(error.into()),
            }
        }
        let control_db = libsql::Builder::new_local(directory.join("fn0-control.sqlite"))
            .flags(libsql::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .build()
            .await?;
        let control_connection = control_db.connect()?;
        let mut discovered = BTreeSet::new();
        match control_connection
            .query("SELECT pk, sk, data FROM docs ORDER BY pk, sk", ())
            .await
        {
            Ok(mut rows) => {
                while let Some(row) = rows.next().await? {
                    let pk = row.get::<String>(0)?;
                    if !pk.starts_with("ProjectDoc/") {
                        continue;
                    }
                    let sk = row.get::<String>(1)?;
                    let data = row.get::<Vec<u8>>(2)?;
                    let document: serde_json::Value = serde_json::from_slice(&data)
                        .with_context(|| format!("malformed ProjectDoc at ({pk:?}, {sk:?})"))?;
                    let project_id = document
                        .get("project_id")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| {
                            anyhow!("ProjectDoc at ({pk:?}, {sk:?}) has no string project_id")
                        })?;
                    if project_id == "fn0-control" {
                        continue;
                    }
                    tenant_id(project_id).with_context(|| {
                        format!("ProjectDoc at ({pk:?}, {sk:?}) has invalid project_id")
                    })?;
                    if !discovered.insert(project_id.to_owned()) {
                        bail!("duplicate project_id {project_id:?} in fn0-control snapshot")
                    }
                }
            }
            Err(error)
                if is_missing_docs_table_error(&error.to_string())
                    && manifest.databases[0].row_count == 0 => {}
            Err(error) => return Err(error.into()),
        }
        if discovered.into_iter().collect::<Vec<_>>() != manifest.active_project_ids {
            bail!(
                "snapshot manifest active project list does not match fn0-control ProjectDoc rows"
            )
        }
        Ok(Self {
            directory,
            manifest,
        })
    }

    pub fn manifest(&self) -> &SnapshotManifest {
        &self.manifest
    }
}

fn file_sha256(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to read snapshot {}", path.display()))?;
    let mut reader = std::io::BufReader::new(file);
    let mut digest = Sha256::new();
    std::io::copy(&mut reader, &mut digest)
        .with_context(|| format!("failed to hash snapshot {}", path.display()))?;
    Ok(format!("{:x}", digest.finalize()))
}

#[async_trait]
impl MigrationSource for SnapshotSource {
    async fn page(
        &self,
        project_id: &str,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<SourcePage> {
        validate_page_size(limit)?;
        if !self
            .manifest
            .project_ids
            .iter()
            .any(|item| item == project_id)
        {
            bail!("project {project_id} is not present in the snapshot manifest")
        }
        let path = self.directory.join(format!("{project_id}.sqlite"));
        let database = libsql::Builder::new_local(&path)
            .flags(libsql::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .build()
            .await?;
        let connection = database.connect()?;
        let query = match after {
            Some(_) => {
                "SELECT pk, sk, data FROM docs WHERE pk > ?1 OR (pk = ?1 AND sk > ?2) ORDER BY pk, sk LIMIT ?3"
            }
            None => "SELECT pk, sk, data FROM docs ORDER BY pk, sk LIMIT ?1",
        };
        let statement = match connection.prepare(query).await {
            Ok(statement) => statement,
            Err(error)
                if is_missing_docs_table_error(&error.to_string())
                    && after.is_none()
                    && self
                        .manifest
                        .databases
                        .iter()
                        .find(|item| item.project_id == project_id)
                        .is_some_and(|item| item.row_count == 0) =>
            {
                return Ok(SourcePage::MissingTable);
            }
            Err(error) => return Err(error.into()),
        };
        let mut rows = match after {
            Some((pk, sk)) => {
                statement
                    .query(libsql::params![pk, sk, limit as i64])
                    .await?
            }
            None => statement.query([limit as i64]).await?,
        };
        let mut result = Vec::new();
        let mut data_bytes = 0usize;
        while let Some(row) = rows.next().await? {
            let data = row.get::<Vec<u8>>(2)?;
            if !result.is_empty()
                && data_bytes.saturating_add(data.len()) > MAX_SNAPSHOT_PAGE_DATA_BYTES
            {
                break;
            }
            data_bytes = data_bytes.saturating_add(data.len());
            result.push(Row {
                pk: row.get(0)?,
                sk: row.get(1)?,
                data,
            });
        }
        if result.is_empty()
            && self
                .manifest
                .databases
                .iter()
                .find(|item| item.project_id == project_id)
                .is_some_and(|item| item.row_count == 0)
        {
            return Ok(SourcePage::Rows(result));
        }
        Ok(SourcePage::Rows(result))
    }
}

impl TursoSource {
    pub fn from_env() -> Result<Self> {
        let group_token =
            std::env::var("TURSO_GROUP_TOKEN").context("TURSO_GROUP_TOKEN must be set")?;
        let host_suffix =
            std::env::var("TURSO_DB_HOST_SUFFIX").context("TURSO_DB_HOST_SUFFIX must be set")?;
        if host_suffix.trim().is_empty() {
            bail!("TURSO_DB_HOST_SUFFIX must not be empty")
        }
        Ok(Self {
            group_token,
            host_suffix,
        })
    }

    fn database(&self, project_id: &str) -> doc_db::Database {
        doc_db::turso_with_config(
            format!("https://{project_id}{}", self.host_suffix),
            self.group_token.clone(),
        )
    }
}

#[async_trait]
impl MigrationSource for TursoSource {
    async fn page(
        &self,
        project_id: &str,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<SourcePage> {
        validate_page_size(limit)?;
        let limit =
            i64::try_from(limit).context("source page size exceeds SQLite integer range")?;
        let (sql, args) = match after {
            Some((after_pk, after_sk)) => (
                "SELECT pk, sk, data FROM docs WHERE pk > ? OR (pk = ? AND sk > ?) ORDER BY pk, sk LIMIT ?",
                vec![
                    Value::Text {
                        value: after_pk.to_owned().into(),
                    },
                    Value::Text {
                        value: after_pk.to_owned().into(),
                    },
                    Value::Text {
                        value: after_sk.to_owned().into(),
                    },
                    Value::Integer { value: limit },
                ],
            ),
            None => (
                "SELECT pk, sk, data FROM docs ORDER BY pk, sk LIMIT ?",
                vec![Value::Integer { value: limit }],
            ),
        };
        let outcome = self
            .database(project_id)
            .execute_raw_transactional_readonly(&[RawStatement {
                sql: sql.to_owned(),
                args,
            }])
            .await;
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(error) if is_missing_docs_table_error(&format!("{error:#}")) => {
                if after.is_some() {
                    bail!("Turso docs table disappeared while scanning project {project_id}")
                }
                return Ok(SourcePage::MissingTable);
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Turso source query failed for project {project_id}")
                });
            }
        };

        parse_source_outcome(project_id, after.is_some(), outcome)
    }
}

fn parse_source_outcome(
    project_id: &str,
    has_cursor: bool,
    outcome: RawTransactionOutcome,
) -> Result<SourcePage> {
    match outcome {
        RawTransactionOutcome::Committed {
            mut statement_results,
        } => {
            let result = statement_results
                .pop()
                .ok_or_else(|| anyhow!("Turso returned no result for project {project_id}"))?;
            Ok(SourcePage::Rows(parse_rows(project_id, result.rows)?))
        }
        RawTransactionOutcome::RolledBack {
            failed_statement_index: _,
            error_message,
        } if is_missing_docs_table_error(&error_message) => {
            if has_cursor {
                bail!("Turso docs table disappeared while scanning project {project_id}")
            }
            Ok(SourcePage::MissingTable)
        }
        RawTransactionOutcome::RolledBack {
            failed_statement_index,
            error_message,
        } => bail!(
            "Turso source query failed for project {project_id} at statement {failed_statement_index}: {error_message}"
        ),
    }
}

pub fn parse_rows(project_id: &str, raw_rows: Vec<Vec<Value>>) -> Result<Vec<Row>> {
    raw_rows
        .into_iter()
        .enumerate()
        .map(|(row_index, raw_row)| {
            if raw_row.len() != 3 {
                let key_columns = raw_row
                    .iter()
                    .take(2)
                    .map(|value| format!("{value:?}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                bail!(
                    "invalid Turso row for project {project_id} at row {row_index} key columns [{key_columns}]: expected 3 columns, got {}",
                    raw_row.len(),
                )
            }
            let pk = match &raw_row[0] {
                Value::Text { value } => value.to_string(),
                _ => bail!("invalid Turso row for project {project_id} at row {row_index} key pk={:?}: pk is not TEXT", raw_row[0]),
            };
            let sk = match &raw_row[1] {
                Value::Text { value } => value.to_string(),
                _ => bail!("invalid Turso row for project {project_id} at row {row_index} key ({pk:?}, sk={:?}): sk is not TEXT", raw_row[1]),
            };
            let data = match &raw_row[2] {
                Value::Blob { value } => value.to_vec(),
                _ => bail!("invalid Turso row for project {project_id} at key ({pk:?}, {sk:?}): data is not BLOB"),
            };
            Ok(Row { pk, sk, data })
        })
        .collect()
}

pub fn is_missing_docs_table_error(error_message: &str) -> bool {
    let message = error_message.to_ascii_lowercase();
    let Some((_, table_name)) = message.split_once("no such table:") else {
        return false;
    };
    let table_name = table_name
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_matches(['"', '\'', '`'])
        .rsplit('.')
        .next()
        .unwrap_or_default();
    table_name == "docs"
}

#[derive(Clone, Debug, Serialize)]
pub struct MismatchSample {
    pub project_id: String,
    pub kind: String,
    pub pk: String,
    pub sk: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProjectStats {
    pub project_id: String,
    pub tenant_id: u64,
    pub source_rows: u64,
    pub destination_rows: u64,
    pub source_data_bytes: u64,
    pub destination_data_bytes: u64,
    pub missing_rows: u64,
    pub extra_rows: u64,
    pub different_rows: u64,
    pub migrated_rows: u64,
    pub skipped_equal: u64,
    pub verified: bool,
}

impl ProjectStats {
    fn new(project_id: &str, tenant_id: u64) -> Self {
        Self {
            project_id: project_id.to_owned(),
            tenant_id,
            source_rows: 0,
            destination_rows: 0,
            source_data_bytes: 0,
            destination_data_bytes: 0,
            missing_rows: 0,
            extra_rows: 0,
            different_rows: 0,
            migrated_rows: 0,
            skipped_equal: 0,
            verified: false,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MigrationReport {
    pub mode: String,
    pub project_count: usize,
    pub projects: Vec<ProjectStats>,
    pub totals: ProjectStats,
    pub mismatch_samples: Vec<MismatchSample>,
    pub verified: bool,
    pub transaction_requests: u64,
    pub maximum_batch_size: usize,
    pub elapsed_ms: u128,
}

fn tenant_id(project_id: &str) -> Result<u64> {
    Ok(doc_db::dodb_tenant_id(project_id)?.get())
}

pub fn normalize_project_subset(project_ids: &[String]) -> Result<Option<Vec<String>>> {
    if project_ids.is_empty() {
        return Ok(None);
    }
    let mut unique = BTreeSet::new();
    for project_id in project_ids {
        tenant_id(project_id)
            .with_context(|| format!("invalid requested project ID {project_id:?}"))?;
        if !unique.insert(project_id.clone()) {
            bail!("duplicate requested project ID {project_id:?}")
        }
    }
    Ok(Some(unique.into_iter().collect()))
}

pub fn validate_page_size(page_size: usize) -> Result<()> {
    if !(1..=4096).contains(&page_size) {
        bail!("page size must be between 1 and 4096")
    }
    Ok(())
}

pub async fn discover_projects(
    source: &dyn MigrationSource,
    page_size: usize,
) -> Result<Vec<String>> {
    validate_page_size(page_size)?;
    let mut projects = BTreeSet::new();
    let mut pager = SourcePager::new(source, "fn0-control", page_size);
    while let Some(row) = pager.next().await? {
        if !row.pk.starts_with("ProjectDoc/") {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&row.data).with_context(|| {
            format!("malformed ProjectDoc JSON at ({:?}, {:?})", row.pk, row.sk)
        })?;
        let project_id = value
            .get("project_id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "ProjectDoc at ({:?}, {:?}) has no string project_id",
                    row.pk,
                    row.sk
                )
            })?
            .to_owned();
        tenant_id(&project_id).with_context(|| {
            format!(
                "ProjectDoc at ({:?}, {:?}) has invalid project_id",
                row.pk, row.sk
            )
        })?;
        if !projects.insert(project_id.clone()) {
            bail!("duplicate project_id {project_id:?} in fn0-control ProjectDoc rows")
        }
    }
    Ok(projects.into_iter().collect())
}

pub async fn migration_projects(
    source: &dyn MigrationSource,
    page_size: usize,
    subset: Option<Vec<String>>,
) -> Result<(Vec<String>, bool)> {
    validate_page_size(page_size)?;
    if let Some(projects) = subset {
        return Ok((projects, true));
    }
    let mut active_projects = discover_projects(source, page_size).await?;
    active_projects.retain(|project_id| project_id != "fn0-control");
    active_projects.sort();
    let mut projects = Vec::with_capacity(active_projects.len() + 1);
    projects.push("fn0-control".to_owned());
    projects.extend(active_projects);
    Ok((projects, false))
}

pub async fn inventory(source: &dyn MigrationSource, page_size: usize) -> Result<MigrationReport> {
    validate_page_size(page_size)?;
    let (projects, _) = migration_projects(source, page_size, None).await?;
    let mut results = Vec::with_capacity(projects.len());
    for project_id in projects {
        let mut stats = ProjectStats::new(&project_id, tenant_id(&project_id)?);
        let mut pager = SourcePager::new(source, &project_id, page_size);
        while let Some(row) = pager.next().await? {
            stats.source_rows += 1;
            stats.source_data_bytes = stats
                .source_data_bytes
                .checked_add(row.data.len() as u64)
                .context("source byte count overflow")?;
        }
        results.push(stats);
    }
    Ok(make_report("inventory", results, vec![]))
}

pub async fn migrate(
    source: &dyn MigrationSource,
    connection: &DodbConnection,
    project_ids: Vec<String>,
    page_size: usize,
    full_mode: bool,
    mismatch_limit: usize,
) -> Result<MigrationReport> {
    validate_page_size(page_size)?;
    let initial_projects = if full_mode {
        migration_projects(source, page_size, None).await?.0
    } else {
        normalize_project_subset(&project_ids)?.unwrap_or_default()
    };
    if initial_projects.is_empty() {
        bail!("migration project set is empty")
    }
    for project_id in &initial_projects {
        tenant_id(project_id)
            .with_context(|| format!("invalid migration project ID {project_id:?}"))?;
    }

    let mut copied = BTreeMap::new();
    for project_id in &initial_projects {
        let destination = doc_db::dodb_with_connection(connection, project_id)?;
        let mut stats = ProjectStats::new(project_id, tenant_id(project_id)?);
        let mut pager = SourcePager::new(source, project_id, page_size);
        while let Some(row) = pager.next().await? {
            match destination.get(&row.pk, &row.sk).await? {
                Some(existing) if existing.as_ref() == row.data => {
                    stats.skipped_equal += 1;
                }
                _ => match destination.put(&row.pk, &row.sk, &row.data).await {
                    Ok(()) => stats.migrated_rows += 1,
                    Err(write_error) => match destination.get(&row.pk, &row.sk).await {
                        Ok(Some(existing)) if existing.as_ref() == row.data => {
                            stats.migrated_rows += 1;
                        }
                        Ok(_) => {
                            return Err(write_error).with_context(|| {
                                format!(
                                    "dodb put outcome did not reconcile for project {} key ({:?}, {:?})",
                                    project_id, row.pk, row.sk
                                )
                            });
                        }
                        Err(read_error) => {
                            return Err(write_error).with_context(|| {
                                format!(
                                    "dodb put failed and reconciliation get also failed for project {} key ({:?}, {:?}): {read_error:#}",
                                    project_id, row.pk, row.sk
                                )
                            });
                        }
                    },
                },
            }
        }
        copied.insert(project_id.clone(), stats);
    }

    if full_mode {
        let final_projects = migration_projects(source, page_size, None).await?.0;
        if final_projects != initial_projects {
            bail!(
                "active project set changed during migration: before={initial_projects:?}, after={final_projects:?}"
            )
        }
    }

    let mut results = Vec::with_capacity(initial_projects.len());
    let mut samples = Vec::new();
    for project_id in &initial_projects {
        let destination = doc_db::dodb_with_connection(connection, project_id)?;
        let mut stats = copied
            .remove(project_id)
            .expect("copied project stats exist");
        compare_project(
            source,
            &destination,
            &mut stats,
            page_size,
            mismatch_limit,
            &mut samples,
        )
        .await?;
        results.push(stats);
    }
    let report = make_report("migrate", results, samples);
    Ok(report)
}

pub async fn migrate_snapshot_batched(
    source: &SnapshotSource,
    connection: &DodbConnection,
    native_connection: &NativeDodbConnection,
    project_ids: Vec<String>,
    mismatch_limit: usize,
) -> Result<MigrationReport> {
    let projects = if project_ids.is_empty() {
        source.manifest.project_ids.clone()
    } else {
        normalize_project_subset(&project_ids)?.unwrap_or_default()
    };
    if projects.is_empty() {
        bail!("migration project set is empty")
    }
    let limits = ProtocolLimits::default();
    let started = std::time::Instant::now();
    let mut request_count = 0u64;
    let mut maximum_batch = 0usize;
    let mut results = Vec::with_capacity(projects.len());
    let mut mismatch_samples = Vec::new();
    for project_id in &projects {
        let tenant = doc_db::dodb_tenant_id(project_id)?;
        let client = native_connection.for_tenant(tenant);
        let database = doc_db::dodb_with_connection(connection, project_id)?;
        let mut stats = ProjectStats::new(project_id, tenant.get());
        let mut pager = SourcePager::new(source, project_id, 256);
        let mut batch = Vec::with_capacity(256);
        while let Some(row) = pager.next().await? {
            if batch.len() == limits.max_mutations {
                request_count += 1;
                maximum_batch = maximum_batch.max(batch.len());
                apply_snapshot_batch(&client, tenant, &batch, limits).await?;
                stats.migrated_rows += batch.len() as u64;
                batch.clear();
            }
            batch.push(row);
            if encoded_batch_size(tenant, &batch, limits)?.is_none() {
                let overflow_row = batch.pop().expect("batch has the new source row");
                if batch.is_empty() {
                    bail!("one source row exceeds the dodb transaction request frame limit")
                }
                request_count += 1;
                maximum_batch = maximum_batch.max(batch.len());
                apply_snapshot_batch(&client, tenant, &batch, limits).await?;
                stats.migrated_rows += batch.len() as u64;
                batch.clear();
                batch.push(overflow_row);
                if encoded_batch_size(tenant, &batch, limits)?.is_none() {
                    bail!("one source row exceeds the dodb transaction request frame limit")
                }
            }
        }
        if !batch.is_empty() {
            request_count += 1;
            maximum_batch = maximum_batch.max(batch.len());
            apply_snapshot_batch(&client, tenant, &batch, limits).await?;
            stats.migrated_rows += batch.len() as u64;
        }
        compare_project(
            source,
            &database,
            &mut stats,
            256,
            mismatch_limit,
            &mut mismatch_samples,
        )
        .await?;
        results.push(stats);
    }
    let mut report = make_report("migrate", results, mismatch_samples);
    report.transaction_requests = request_count;
    report.maximum_batch_size = maximum_batch;
    report.elapsed_ms = started.elapsed().as_millis();
    eprintln!(
        "snapshot import: rows={} maximum_batch={} transaction_requests={} elapsed_ms={}",
        report.totals.migrated_rows,
        report.maximum_batch_size,
        report.transaction_requests,
        report.elapsed_ms
    );
    Ok(report)
}

fn encoded_batch_size(
    tenant: TenantId,
    rows: &[Row],
    limits: ProtocolLimits,
) -> Result<Option<usize>> {
    if rows
        .iter()
        .any(|row| row.data.len() > limits.max_value_size)
    {
        bail!("source value exceeds the dodb protocol value limit")
    }
    let mutations = rows
        .iter()
        .map(|row| TransactionMutation::Put {
            key: DocumentKey::new(row.pk.as_bytes().to_vec(), row.sk.as_bytes().to_vec()),
            value: row.data.clone(),
        })
        .collect();
    let request = DodbRequest::Transact {
        request: TransactionRequest::new(Vec::new(), mutations),
    };
    match dodb_protocol::encode_request(tenant, &request, limits) {
        Ok(encoded) => Ok(Some(encoded.len())),
        Err(dodb_protocol::ProtocolError::PayloadTooLarge { maximum, .. })
            if maximum == limits.max_request_frame_size =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

async fn apply_snapshot_batch<D: BatchDestination>(
    client: &D,
    tenant: TenantId,
    rows: &[Row],
    limits: ProtocolLimits,
) -> Result<()> {
    let mutations = rows
        .iter()
        .map(|row| TransactionMutation::Put {
            key: DocumentKey::new(row.pk.as_bytes().to_vec(), row.sk.as_bytes().to_vec()),
            value: row.data.clone(),
        })
        .collect();
    let request = TransactionRequest::new(Vec::new(), mutations);
    match client.transact(request.clone()).await {
        Ok(_) => Ok(()),
        Err(write_error) => {
            let mut equal_rows = 0usize;
            for row in rows {
                if client
                    .get(row)
                    .await?
                    .is_some_and(|value| value == row.data)
                {
                    equal_rows += 1;
                }
            }
            if equal_rows == rows.len() {
                return Ok(());
            }
            if equal_rows != 0 {
                bail!(
                    "uncertain dodb batch has mixed destination state; batch was not retried: {write_error}"
                )
            }
            if encoded_batch_size(tenant, rows, limits)?.is_none() {
                bail!("batch no longer fits dodb protocol limits")
            }
            client.transact(request).await.map_err(|retry_error| anyhow!("dodb batch failed and atomic retry did not succeed; original={write_error}; retry={retry_error}"))?;
            Ok(())
        }
    }
}

pub async fn verify(
    source: &dyn MigrationSource,
    connection: &DodbConnection,
    project_ids: Vec<String>,
    page_size: usize,
    full_mode: bool,
    mismatch_limit: usize,
) -> Result<MigrationReport> {
    validate_page_size(page_size)?;
    let projects = if full_mode {
        migration_projects(source, page_size, None).await?.0
    } else {
        normalize_project_subset(&project_ids)?.unwrap_or_default()
    };
    if projects.is_empty() {
        bail!("verification project set is empty")
    }
    let mut results = Vec::with_capacity(projects.len());
    let mut samples = Vec::new();
    for project_id in projects {
        let destination = doc_db::dodb_with_connection(connection, &project_id)?;
        let mut stats = ProjectStats::new(&project_id, tenant_id(&project_id)?);
        compare_project(
            source,
            &destination,
            &mut stats,
            page_size,
            mismatch_limit,
            &mut samples,
        )
        .await?;
        results.push(stats);
    }
    Ok(make_report("verify", results, samples))
}

async fn compare_project(
    source: &dyn MigrationSource,
    destination: &doc_db::Database,
    stats: &mut ProjectStats,
    page_size: usize,
    mismatch_limit: usize,
    samples: &mut Vec<MismatchSample>,
) -> Result<()> {
    let mut source_pager = SourcePager::new(source, &stats.project_id, page_size);
    let mut destination_pager = DestinationPager::new(destination, page_size);
    let mut source_row = source_pager.next().await?;
    let mut destination_row = destination_pager.next().await?;
    while source_row.is_some() || destination_row.is_some() {
        match (source_row.as_ref(), destination_row.as_ref()) {
            (Some(source), Some(destination)) if source.key() == destination.key() => {
                stats.source_rows += 1;
                stats.destination_rows += 1;
                add_bytes(&mut stats.source_data_bytes, source.data.len())?;
                add_bytes(&mut stats.destination_data_bytes, destination.data.len())?;
                if source.data != destination.data {
                    stats.different_rows += 1;
                    add_sample(
                        samples,
                        mismatch_limit,
                        &stats.project_id,
                        "different",
                        source,
                    );
                }
                source_row = source_pager.next().await?;
                destination_row = destination_pager.next().await?;
            }
            (Some(source), Some(destination)) if source.key() < destination.key() => {
                stats.source_rows += 1;
                add_bytes(&mut stats.source_data_bytes, source.data.len())?;
                stats.missing_rows += 1;
                add_sample(
                    samples,
                    mismatch_limit,
                    &stats.project_id,
                    "missing",
                    source,
                );
                source_row = source_pager.next().await?;
            }
            (Some(_), Some(destination)) => {
                stats.destination_rows += 1;
                add_bytes(&mut stats.destination_data_bytes, destination.data.len())?;
                stats.extra_rows += 1;
                add_sample(
                    samples,
                    mismatch_limit,
                    &stats.project_id,
                    "extra",
                    destination,
                );
                destination_row = destination_pager.next().await?;
            }
            (Some(source), None) => {
                stats.source_rows += 1;
                add_bytes(&mut stats.source_data_bytes, source.data.len())?;
                stats.missing_rows += 1;
                add_sample(
                    samples,
                    mismatch_limit,
                    &stats.project_id,
                    "missing",
                    source,
                );
                source_row = source_pager.next().await?;
            }
            (None, Some(destination)) => {
                stats.destination_rows += 1;
                add_bytes(&mut stats.destination_data_bytes, destination.data.len())?;
                stats.extra_rows += 1;
                add_sample(
                    samples,
                    mismatch_limit,
                    &stats.project_id,
                    "extra",
                    destination,
                );
                destination_row = destination_pager.next().await?;
            }
            (None, None) => break,
        }
    }
    stats.verified = stats.missing_rows == 0 && stats.extra_rows == 0 && stats.different_rows == 0;
    Ok(())
}

fn add_bytes(total: &mut u64, amount: usize) -> Result<()> {
    *total = total
        .checked_add(amount as u64)
        .context("data byte count overflow")?;
    Ok(())
}

fn add_sample(
    samples: &mut Vec<MismatchSample>,
    limit: usize,
    project_id: &str,
    kind: &str,
    row: &Row,
) {
    if samples.len() < limit {
        samples.push(MismatchSample {
            project_id: project_id.to_owned(),
            kind: kind.to_owned(),
            pk: row.pk.clone(),
            sk: row.sk.clone(),
        });
    }
}

fn make_report(
    mode: &str,
    projects: Vec<ProjectStats>,
    mismatch_samples: Vec<MismatchSample>,
) -> MigrationReport {
    let mut totals = ProjectStats::new("total", 0);
    for project in &projects {
        totals.source_rows += project.source_rows;
        totals.destination_rows += project.destination_rows;
        totals.source_data_bytes += project.source_data_bytes;
        totals.destination_data_bytes += project.destination_data_bytes;
        totals.missing_rows += project.missing_rows;
        totals.extra_rows += project.extra_rows;
        totals.different_rows += project.different_rows;
        totals.migrated_rows += project.migrated_rows;
        totals.skipped_equal += project.skipped_equal;
    }
    totals.verified = projects.iter().all(|project| project.verified);
    MigrationReport {
        mode: mode.to_owned(),
        project_count: projects.len(),
        verified: totals.verified,
        projects,
        totals,
        mismatch_samples,
        transaction_requests: 0,
        maximum_batch_size: 0,
        elapsed_ms: 0,
    }
}

struct SourcePager<'a> {
    source: &'a dyn MigrationSource,
    project_id: &'a str,
    page_size: usize,
    cursor: Option<(String, String)>,
    rows: VecDeque<Row>,
    finished: bool,
}

impl<'a> SourcePager<'a> {
    fn new(source: &'a dyn MigrationSource, project_id: &'a str, page_size: usize) -> Self {
        Self {
            source,
            project_id,
            page_size,
            cursor: None,
            rows: VecDeque::new(),
            finished: false,
        }
    }

    async fn next(&mut self) -> Result<Option<Row>> {
        while self.rows.is_empty() && !self.finished {
            let page = self
                .source
                .page(
                    self.project_id,
                    self.cursor
                        .as_ref()
                        .map(|(pk, sk)| (pk.as_str(), sk.as_str())),
                    self.page_size,
                )
                .await?;
            let SourcePage::Rows(page) = page else {
                if self.cursor.is_some() {
                    bail!(
                        "Turso docs table disappeared while scanning project {}",
                        self.project_id
                    )
                }
                self.finished = true;
                return Ok(None);
            };
            validate_source_page(self.project_id, self.cursor.as_ref(), &page)?;
            if page.is_empty() {
                self.finished = true;
                return Ok(None);
            }
            if let Some(last) = page.last() {
                self.cursor = Some((last.pk.clone(), last.sk.clone()));
            }
            self.rows.extend(page);
        }
        Ok(self.rows.pop_front())
    }
}

struct DestinationPager<'a> {
    database: &'a doc_db::Database,
    page_size: usize,
    cursor: Option<(String, String)>,
    rows: VecDeque<Row>,
    finished: bool,
}

impl<'a> DestinationPager<'a> {
    fn new(database: &'a doc_db::Database, page_size: usize) -> Self {
        Self {
            database,
            page_size,
            cursor: None,
            rows: VecDeque::new(),
            finished: false,
        }
    }

    async fn next(&mut self) -> Result<Option<Row>> {
        while self.rows.is_empty() && !self.finished {
            let page = self
                .database
                .scan(
                    self.cursor
                        .as_ref()
                        .map(|(pk, sk)| (pk.as_str(), sk.as_str())),
                    self.page_size,
                )
                .await?;
            if page.len() < self.page_size {
                self.finished = true;
            }
            let parsed = page
                .into_iter()
                .map(|(pk, sk, data)| Row {
                    pk,
                    sk,
                    data: data.to_vec(),
                })
                .collect::<Vec<_>>();
            validate_source_page("dodb destination", self.cursor.as_ref(), &parsed)?;
            if let Some(last) = parsed.last() {
                self.cursor = Some((last.pk.clone(), last.sk.clone()));
            }
            self.rows.extend(parsed);
        }
        Ok(self.rows.pop_front())
    }
}

fn validate_source_page(
    project_id: &str,
    cursor: Option<&(String, String)>,
    rows: &[Row],
) -> Result<()> {
    let mut previous = cursor.cloned();
    for row in rows {
        let key = (row.pk.clone(), row.sk.clone());
        if previous.as_ref().is_some_and(|previous| key <= *previous) {
            bail!(
                "source page for {project_id} is not strictly ordered after cursor: ({:?}, {:?})",
                row.pk,
                row.sk
            )
        }
        previous = Some(key);
    }
    Ok(())
}

pub fn dodb_config(address: &str, server_name: &str, root_cert: &[u8]) -> Result<DodbConfig> {
    let address = address
        .parse::<SocketAddr>()
        .context("invalid dodb address")?;
    if root_cert.is_empty() {
        bail!("dodb root certificate is empty")
    }
    let mut reader = Cursor::new(root_cert);
    let certificates = rustls_pemfile::certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .map(|certificate| certificate.to_vec())
        .collect::<Vec<_>>();
    if certificates.is_empty() {
        bail!("dodb root certificate file contains no PEM certificates")
    }
    Ok(DodbConfig::new(address, server_name, certificates))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use dodb_server::{
        DodbServer, DodbServerConfig, LocalTenantService, LocalTenantServiceConfig, ServerTlsConfig,
    };
    use rcgen::generate_simple_self_signed;
    use std::sync::Arc;

    #[derive(Clone, Default)]
    struct FakeSource {
        databases: BTreeMap<String, Vec<Row>>,
    }

    #[async_trait]
    impl MigrationSource for FakeSource {
        async fn page(
            &self,
            project_id: &str,
            after: Option<(&str, &str)>,
            limit: usize,
        ) -> Result<SourcePage> {
            let Some(rows) = self.databases.get(project_id) else {
                return Ok(SourcePage::MissingTable);
            };
            let mut ordered = rows.clone();
            ordered.sort();
            let filtered = ordered
                .iter()
                .filter(|row| {
                    after.is_none_or(|(pk, sk)| (row.pk.as_str(), row.sk.as_str()) > (pk, sk))
                })
                .take(limit)
                .cloned()
                .collect();
            Ok(SourcePage::Rows(filtered))
        }
    }

    fn row(pk: &str, sk: &str, data: &[u8]) -> Row {
        Row {
            pk: pk.into(),
            sk: sk.into(),
            data: data.into(),
        }
    }

    #[derive(Default)]
    struct FakeBatchDestination {
        values: std::sync::Mutex<BTreeMap<(String, String), Vec<u8>>>,
        transaction_count: std::sync::atomic::AtomicUsize,
        get_count: std::sync::atomic::AtomicUsize,
        fail_first: std::sync::atomic::AtomicBool,
        commit_before_error: bool,
    }

    #[async_trait]
    impl BatchDestination for FakeBatchDestination {
        async fn transact(&self, request: TransactionRequest) -> Result<()> {
            self.transaction_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let first_error = self
                .fail_first
                .swap(false, std::sync::atomic::Ordering::SeqCst);
            if !first_error || self.commit_before_error {
                let mut values = self.values.lock().unwrap();
                for mutation in request.mutations {
                    let TransactionMutation::Put { key, value } = mutation else {
                        bail!("expected put mutation")
                    };
                    values.insert(
                        (
                            String::from_utf8(key.pk.into_bytes())?,
                            String::from_utf8(key.sk.into_bytes())?,
                        ),
                        value,
                    );
                }
            }
            if first_error {
                bail!("simulated unknown mutation outcome")
            }
            Ok(())
        }

        async fn get(&self, row: &Row) -> Result<Option<Vec<u8>>> {
            self.get_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(self
                .values
                .lock()
                .unwrap()
                .get(&(row.pk.clone(), row.sk.clone()))
                .cloned())
        }
    }

    #[tokio::test]
    async fn batch_import_uses_one_transaction_and_no_normal_path_gets() {
        let destination = FakeBatchDestination::default();
        let source_rows = vec![row("a", "1", b"one"), row("a", "2", b"two")];
        apply_snapshot_batch(
            &destination,
            TenantId::new(1),
            &source_rows,
            ProtocolLimits::default(),
        )
        .await
        .unwrap();
        assert_eq!(
            destination
                .transaction_count
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        assert_eq!(
            destination
                .get_count
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        assert_eq!(destination.values.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn uncertain_atomic_batch_accepts_all_equal_and_retries_all_missing() {
        let committed = FakeBatchDestination {
            fail_first: std::sync::atomic::AtomicBool::new(true),
            commit_before_error: true,
            ..FakeBatchDestination::default()
        };
        let source_rows = vec![row("a", "1", b"one"), row("a", "2", b"two")];
        apply_snapshot_batch(
            &committed,
            TenantId::new(1),
            &source_rows,
            ProtocolLimits::default(),
        )
        .await
        .unwrap();
        assert_eq!(
            committed
                .transaction_count
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        assert_eq!(
            committed
                .get_count
                .load(std::sync::atomic::Ordering::SeqCst),
            2
        );

        let not_committed = FakeBatchDestination {
            fail_first: std::sync::atomic::AtomicBool::new(true),
            commit_before_error: false,
            ..FakeBatchDestination::default()
        };
        apply_snapshot_batch(
            &not_committed,
            TenantId::new(1),
            &source_rows,
            ProtocolLimits::default(),
        )
        .await
        .unwrap();
        assert_eq!(
            not_committed
                .transaction_count
                .load(std::sync::atomic::Ordering::SeqCst),
            2
        );
        assert_eq!(
            not_committed
                .get_count
                .load(std::sync::atomic::Ordering::SeqCst),
            2
        );
        assert_eq!(not_committed.values.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn uncertain_batch_with_mixed_state_fails_closed_without_assuming_partial_commit() {
        let destination = FakeBatchDestination {
            values: std::sync::Mutex::new(BTreeMap::from([(
                ("a".to_owned(), "1".to_owned()),
                b"one".to_vec(),
            )])),
            fail_first: std::sync::atomic::AtomicBool::new(true),
            commit_before_error: false,
            ..FakeBatchDestination::default()
        };
        let source_rows = vec![row("a", "1", b"one"), row("a", "2", b"two")];
        let error = apply_snapshot_batch(
            &destination,
            TenantId::new(1),
            &source_rows,
            ProtocolLimits::default(),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("mixed destination state"));
        assert_eq!(
            destination
                .transaction_count
                .load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        assert_eq!(
            destination
                .get_count
                .load(std::sync::atomic::Ordering::SeqCst),
            2
        );
    }

    async fn write_snapshot_database(path: &Path, rows: &[Row]) -> Result<()> {
        let database = libsql::Builder::new_local(path).build().await?;
        let connection = database.connect()?;
        connection.execute("CREATE TABLE docs (pk TEXT NOT NULL, sk TEXT NOT NULL, data BLOB NOT NULL, version INTEGER NOT NULL, PRIMARY KEY (pk, sk))", ()).await?;
        for row in rows {
            connection
                .execute(
                    "INSERT INTO docs (pk, sk, data, version) VALUES (?1, ?2, ?3, 1)",
                    libsql::params![row.pk.as_str(), row.sk.as_str(), row.data.as_slice()],
                )
                .await?;
        }
        Ok(())
    }

    async fn write_snapshot_manifest(
        directory: &Path,
        project_ids: Vec<String>,
        active_project_ids: Vec<String>,
    ) -> Result<()> {
        let mut databases = Vec::new();
        for project_id in &project_ids {
            let path = directory.join(format!("{project_id}.sqlite"));
            let db = libsql::Builder::new_local(&path)
                .flags(libsql::OpenFlags::SQLITE_OPEN_READ_ONLY)
                .build()
                .await?;
            let connection = db.connect()?;
            let mut counts = connection
                .query(
                    "SELECT COUNT(*), COALESCE(SUM(length(data)), 0) FROM docs",
                    (),
                )
                .await?;
            let count_row = counts.next().await?.unwrap();
            databases.push(SnapshotDatabase {
                project_id: project_id.clone(),
                filename: format!("{project_id}.sqlite"),
                source_state: "present".to_owned(),
                row_count: count_row.get::<i64>(0)? as u64,
                data_bytes: count_row.get::<i64>(1)? as u64,
                sha256: file_sha256(&path)?,
            });
        }
        let manifest = SnapshotManifest {
            format_version: 1,
            captured_at: "2026-09-25T00:00:00Z".to_owned(),
            project_ids,
            active_project_ids,
            databases,
        };
        std::fs::write(
            directory.join("manifest.json"),
            serde_json::to_vec(&manifest)?,
        )?;
        Ok(())
    }

    async fn test_connection() -> (
        DodbConnection,
        Arc<DodbServer<LocalTenantService>>,
        tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
        tempfile::TempDir,
        NativeDodbConnection,
    ) {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let directory = tempfile::tempdir().unwrap();
        let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
        let certificate = certified.cert.der().to_vec();
        let private_key = certified.signing_key.serialize_der();
        let service = Arc::new(
            LocalTenantService::new(LocalTenantServiceConfig {
                data_dir: directory.path().to_owned(),
                ..LocalTenantServiceConfig::default()
            })
            .unwrap(),
        );
        let server = Arc::new(
            DodbServer::bind(
                service,
                DodbServerConfig {
                    listen_addr: "127.0.0.1:0".parse().unwrap(),
                    tls: ServerTlsConfig::from_der(vec![certificate.clone()], private_key).unwrap(),
                    protocol_limits: dodb_protocol::ProtocolLimits::default(),
                    max_connections: 8,
                    max_concurrent_streams: 64,
                    max_concurrent_requests: 64,
                },
            )
            .unwrap(),
        );
        let running_server = Arc::clone(&server);
        let task = tokio::spawn(async move { running_server.run().await });
        let connection = DodbConnection::connect(&DodbConfig::new(
            server.local_addr().unwrap(),
            "localhost",
            vec![certificate.clone()],
        ))
        .await
        .unwrap();
        let native_connection = NativeDodbConnection::connect(
            "0.0.0.0:0".parse().unwrap(),
            server.local_addr().unwrap(),
            "localhost",
            dodb_client::ClientTlsConfig::from_der(vec![certificate]).unwrap(),
            ProtocolLimits::default(),
        )
        .await
        .unwrap();
        (connection, server, task, directory, native_connection)
    }

    #[tokio::test]
    async fn sqlite_snapshot_reads_ordered_blob_rows_and_validates_manifest() {
        let directory = tempfile::tempdir().unwrap();
        let control_rows = vec![
            row(
                "ProjectDoc/fn0-control",
                "",
                br#"{"project_id":"fn0-control"}"#,
            ),
            row(
                "ProjectDoc/00000001",
                "doc",
                br#"{"project_id":"00000001"}"#,
            ),
            row("Settings", "main", b"control"),
        ];
        let project_rows = vec![row("a", "one", &[0, 255, 1]), row("a", "two", b"second")];
        write_snapshot_database(&directory.path().join("fn0-control.sqlite"), &control_rows)
            .await
            .unwrap();
        write_snapshot_database(&directory.path().join("00000001.sqlite"), &project_rows)
            .await
            .unwrap();
        write_snapshot_manifest(
            directory.path(),
            vec!["fn0-control".into(), "00000001".into()],
            vec!["00000001".into()],
        )
        .await
        .unwrap();
        let source = SnapshotSource::open(directory.path()).await.unwrap();
        let first_page = source.page("00000001", None, 1).await.unwrap();
        let SourcePage::Rows(first_page) = first_page else {
            panic!("expected snapshot rows")
        };
        assert_eq!(first_page, vec![row("a", "one", &[0, 255, 1])]);
        let next_page = source
            .page("00000001", Some(("a", "one")), 1)
            .await
            .unwrap();
        let SourcePage::Rows(next_page) = next_page else {
            panic!("expected snapshot rows")
        };
        assert_eq!(next_page, vec![row("a", "two", b"second")]);
    }

    #[tokio::test]
    async fn sqlite_snapshot_rejects_checksum_counts_schema_and_active_project_mismatches() {
        let directory = tempfile::tempdir().unwrap();
        let control_rows = vec![row(
            "ProjectDoc/00000001",
            "doc",
            br#"{"project_id":"00000001"}"#,
        )];
        write_snapshot_database(&directory.path().join("fn0-control.sqlite"), &control_rows)
            .await
            .unwrap();
        write_snapshot_database(
            &directory.path().join("00000001.sqlite"),
            &[row("pk", "sk", b"value")],
        )
        .await
        .unwrap();
        write_snapshot_manifest(
            directory.path(),
            vec!["fn0-control".into(), "00000001".into()],
            vec!["00000001".into()],
        )
        .await
        .unwrap();
        let original = std::fs::read(directory.path().join("00000001.sqlite")).unwrap();
        std::fs::write(directory.path().join("00000001.sqlite"), b"not sqlite").unwrap();
        assert!(
            format!(
                "{:#}",
                SnapshotSource::open(directory.path()).await.unwrap_err()
            )
            .contains("checksum mismatch")
        );
        std::fs::write(directory.path().join("00000001.sqlite"), original).unwrap();
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.path().join("manifest.json")).unwrap())
                .unwrap();
        manifest["databases"][1]["row_count"] = serde_json::json!(2);
        std::fs::write(
            directory.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            format!(
                "{:#}",
                SnapshotSource::open(directory.path()).await.unwrap_err()
            )
            .contains("count mismatch")
        );
        manifest["databases"][1]["row_count"] = serde_json::json!(1);
        manifest["active_project_ids"] = serde_json::json!(["00000002"]);
        std::fs::write(
            directory.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert!(
            format!(
                "{:#}",
                SnapshotSource::open(directory.path()).await.unwrap_err()
            )
            .contains("project IDs")
        );
        let malformed_directory = tempfile::tempdir().unwrap();
        write_snapshot_database(
            &malformed_directory.path().join("fn0-control.sqlite"),
            &control_rows,
        )
        .await
        .unwrap();
        let malformed_path = malformed_directory.path().join("00000001.sqlite");
        let malformed_db = libsql::Builder::new_local(&malformed_path)
            .build()
            .await
            .unwrap();
        malformed_db
            .connect()
            .unwrap()
            .execute("CREATE TABLE docs (wrong TEXT)", ())
            .await
            .unwrap();
        drop(malformed_db);
        let mut malformed_manifest = serde_json::json!({"format_version":1,"captured_at":"test","project_ids":["fn0-control","00000001"],"active_project_ids":["00000001"],"databases":[
            {"project_id":"fn0-control","filename":"fn0-control.sqlite","row_count":1,"data_bytes":control_rows[0].data.len(),"sha256":file_sha256(&malformed_directory.path().join("fn0-control.sqlite")).unwrap()},
            {"project_id":"00000001","filename":"00000001.sqlite","row_count":0,"data_bytes":0,"sha256":file_sha256(&malformed_path).unwrap()}
        ]});
        std::fs::write(
            malformed_directory.path().join("manifest.json"),
            serde_json::to_vec(&malformed_manifest).unwrap(),
        )
        .unwrap();
        assert!(
            format!(
                "{:#}",
                SnapshotSource::open(malformed_directory.path())
                    .await
                    .unwrap_err()
            )
            .contains("invalid columns")
        );
        malformed_manifest["databases"][1]["sha256"] = serde_json::json!("0".repeat(64));
        std::fs::write(
            malformed_directory.path().join("manifest.json"),
            serde_json::to_vec(&malformed_manifest).unwrap(),
        )
        .unwrap();
        assert!(
            format!(
                "{:#}",
                SnapshotSource::open(malformed_directory.path())
                    .await
                    .unwrap_err()
            )
            .contains("checksum mismatch")
        );
    }

    #[tokio::test]
    async fn snapshot_missing_docs_table_is_an_empty_source() {
        let directory = tempfile::tempdir().unwrap();
        let control_path = directory.path().join("fn0-control.sqlite");
        let control_database = libsql::Builder::new_local(&control_path)
            .build()
            .await
            .unwrap();
        let control_connection = control_database.connect().unwrap();
        drop(control_connection);
        drop(control_database);
        let manifest = SnapshotManifest {
            format_version: 1,
            captured_at: "test".into(),
            project_ids: vec!["fn0-control".into()],
            active_project_ids: vec![],
            databases: vec![SnapshotDatabase {
                project_id: "fn0-control".into(),
                filename: "fn0-control.sqlite".into(),
                source_state: "present".into(),
                row_count: 0,
                data_bytes: 0,
                sha256: file_sha256(&control_path).unwrap(),
            }],
        };
        std::fs::write(
            directory.path().join("manifest.json"),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        let source = SnapshotSource::open(directory.path()).await.unwrap();
        assert_eq!(
            source.page("fn0-control", None, 1).await.unwrap(),
            SourcePage::MissingTable
        );
    }

    #[tokio::test]
    async fn snapshot_batch_migration_resumes_and_preserves_destination_only_rows() {
        let directory = tempfile::tempdir().unwrap();
        let control_rows = vec![row(
            "ProjectDoc/00000001",
            "doc",
            br#"{"project_id":"00000001"}"#,
        )];
        let source_rows = vec![row("pk", "a", &[0, 255]), row("pk", "b", b"expected")];
        write_snapshot_database(&directory.path().join("fn0-control.sqlite"), &control_rows)
            .await
            .unwrap();
        write_snapshot_database(&directory.path().join("00000001.sqlite"), &source_rows)
            .await
            .unwrap();
        write_snapshot_manifest(
            directory.path(),
            vec!["fn0-control".into(), "00000001".into()],
            vec!["00000001".into()],
        )
        .await
        .unwrap();
        let source = SnapshotSource::open(directory.path()).await.unwrap();
        let (connection, server, task, _dodb_directory, native_connection) =
            test_connection().await;
        let database = destination(&connection, "00000001").await;
        database.put("extra", "row", b"retain").await.unwrap();
        let first = migrate_snapshot_batched(&source, &connection, &native_connection, vec![], 20)
            .await
            .unwrap();
        assert!(!first.verified);
        assert_eq!(first.projects[1].extra_rows, 1);
        assert_eq!(
            database
                .get("extra", "row")
                .await
                .unwrap()
                .unwrap()
                .as_ref(),
            b"retain"
        );
        let second = migrate_snapshot_batched(&source, &connection, &native_connection, vec![], 20)
            .await
            .unwrap();
        assert!(!second.verified);
        assert_eq!(second.projects[1].source_rows, 2);
        assert_eq!(second.projects[1].different_rows, 0);
        assert_eq!(second.projects[1].missing_rows, 0);
        assert_eq!(second.projects[1].extra_rows, 1);
        native_connection.close();
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn snapshot_project_discovery_rejects_invalid_ids() {
        let directory = tempfile::tempdir().unwrap();
        write_snapshot_database(
            &directory.path().join("fn0-control.sqlite"),
            &[row("ProjectDoc/x", "doc", br#"{"project_id":"INVALID"}"#)],
        )
        .await
        .unwrap();
        write_snapshot_database(&directory.path().join("00000001.sqlite"), &[])
            .await
            .unwrap();
        write_snapshot_manifest(
            directory.path(),
            vec!["fn0-control".into(), "00000001".into()],
            vec!["00000001".into()],
        )
        .await
        .unwrap();
        assert!(
            format!(
                "{:#}",
                SnapshotSource::open(directory.path()).await.unwrap_err()
            )
            .contains("invalid project_id")
        );
    }

    #[tokio::test]
    #[ignore = "synthetic batch performance sanity test"]
    async fn synthetic_snapshot_import_uses_bounded_transaction_batches() {
        let row_count = 18_795usize;
        let directory = tempfile::tempdir().unwrap();
        let control_rows = vec![row(
            "ProjectDoc/00000001",
            "doc",
            br#"{"project_id":"00000001"}"#,
        )];
        write_snapshot_database(&directory.path().join("fn0-control.sqlite"), &control_rows)
            .await
            .unwrap();
        let project_path = directory.path().join("00000001.sqlite");
        let project = libsql::Builder::new_local(&project_path)
            .build()
            .await
            .unwrap();
        let project_connection = project.connect().unwrap();
        project_connection.execute("CREATE TABLE docs (pk TEXT NOT NULL, sk TEXT NOT NULL, data BLOB NOT NULL, version INTEGER NOT NULL, PRIMARY KEY (pk, sk))", ()).await.unwrap();
        project_connection.execute("BEGIN", ()).await.unwrap();
        for row_index in 0..row_count {
            let sk = format!("{row_index:08}");
            let data = format!("synthetic-payload-{row_index:08}");
            project_connection
                .execute(
                    "INSERT INTO docs VALUES ('synthetic', ?1, ?2, 1)",
                    libsql::params![sk, data.as_bytes()],
                )
                .await
                .unwrap();
        }
        project_connection.execute("COMMIT", ()).await.unwrap();
        drop(project_connection);
        drop(project);
        write_snapshot_manifest(
            directory.path(),
            vec!["fn0-control".into(), "00000001".into()],
            vec!["00000001".into()],
        )
        .await
        .unwrap();
        let source = SnapshotSource::open(directory.path()).await.unwrap();
        let (connection, server, task, _dodb_directory, native_connection) =
            test_connection().await;
        let started = std::time::Instant::now();
        let report = migrate_snapshot_batched(
            &source,
            &connection,
            &native_connection,
            vec!["00000001".into()],
            0,
        )
        .await
        .unwrap();
        let elapsed = started.elapsed();
        assert!(report.verified);
        assert_eq!(report.totals.source_rows, row_count as u64);
        assert_eq!(report.transaction_requests, 74);
        assert_eq!(report.maximum_batch_size, 256);
        println!(
            "synthetic rows={} batch_size={} transactions={} elapsed_ms={}",
            row_count,
            report.maximum_batch_size,
            report.transaction_requests,
            elapsed.as_millis()
        );
        native_connection.close();
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    async fn destination(connection: &DodbConnection, project_id: &str) -> doc_db::Database {
        doc_db::dodb_with_connection(connection, project_id).unwrap()
    }

    #[tokio::test]
    async fn migration_is_resumable_preserves_bytes_and_verifies_exact_rows() {
        let source = FakeSource {
            databases: BTreeMap::from([(
                "00000001".into(),
                vec![
                    row("p", "a", &[0, 255, 1]),
                    row("p", "b", b"two"),
                    row("q", "a", b"three"),
                ],
            )]),
        };
        let (connection, server, task, _directory, _native_connection) = test_connection().await;
        let database = destination(&connection, "00000001").await;
        database.put("p", "b", b"two").await.unwrap();
        let report = migrate(&source, &connection, vec!["00000001".into()], 1, false, 20)
            .await
            .unwrap();
        assert!(report.verified);
        assert_eq!(report.projects[0].source_rows, 3);
        assert_eq!(report.projects[0].source_data_bytes, 11);
        assert_eq!(report.projects[0].destination_data_bytes, 11);
        assert_eq!(report.projects[0].migrated_rows, 2);
        assert_eq!(report.projects[0].skipped_equal, 1);
        assert_eq!(
            database.get("p", "a").await.unwrap().unwrap(),
            Bytes::from_static(&[0, 255, 1])
        );
        let rerun = migrate(&source, &connection, vec!["00000001".into()], 2, false, 20)
            .await
            .unwrap();
        assert_eq!(rerun.projects[0].migrated_rows, 0);
        assert_eq!(rerun.projects[0].skipped_equal, 3);
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn verification_reports_missing_extra_and_different_rows() {
        let source = FakeSource {
            databases: BTreeMap::from([(
                "00000002".into(),
                vec![
                    row("p", "different", b"source"),
                    row("p", "missing", b"missing"),
                ],
            )]),
        };
        let (connection, server, task, _directory, _native_connection) = test_connection().await;
        let database = destination(&connection, "00000002").await;
        database
            .put("p", "different", b"destination")
            .await
            .unwrap();
        database.put("p", "extra", b"extra").await.unwrap();
        let report = verify(&source, &connection, vec!["00000002".into()], 1, false, 20)
            .await
            .unwrap();
        let stats = &report.projects[0];
        assert!(!report.verified);
        assert_eq!(stats.missing_rows, 1);
        assert_eq!(stats.extra_rows, 1);
        assert_eq!(stats.different_rows, 1);
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn migration_replaces_different_destination_values() {
        let source = FakeSource {
            databases: BTreeMap::from([(
                "00000004".into(),
                vec![row("pk", "sk", b"source-value")],
            )]),
        };
        let (connection, server, task, _directory, _native_connection) = test_connection().await;
        let database = destination(&connection, "00000004").await;
        database.put("pk", "sk", b"old-value").await.unwrap();
        let report = migrate(
            &source,
            &connection,
            vec!["00000004".into()],
            256,
            false,
            20,
        )
        .await
        .unwrap();
        assert!(report.verified);
        assert_eq!(report.projects[0].migrated_rows, 1);
        assert_eq!(
            database.get("pk", "sk").await.unwrap().unwrap().as_ref(),
            b"source-value"
        );
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn empty_source_verifies_against_empty_destination() {
        let source = FakeSource::default();
        let (connection, server, task, _directory, _native_connection) = test_connection().await;
        let report = verify(
            &source,
            &connection,
            vec!["00000003".into()],
            256,
            false,
            20,
        )
        .await
        .unwrap();
        assert!(report.verified);
        assert_eq!(report.projects[0].source_rows, 0);
        assert_eq!(report.projects[0].destination_rows, 0);
        connection.close();
        server.shutdown().await;
        task.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn project_discovery_validates_and_sorts_project_docs() {
        let control_rows = vec![
            row(
                "ProjectDoc/zzzzzzzz",
                "doc",
                br#"{"project_id":"zzzzzzzz"}"#,
            ),
            row(
                "OtherDoc/00000001",
                "doc",
                br#"{"project_id":"not-a-project"}"#,
            ),
            row(
                "ProjectDoc/00000001",
                "doc",
                br#"{"project_id":"00000001"}"#,
            ),
        ];
        let source = FakeSource {
            databases: BTreeMap::from([("fn0-control".into(), control_rows)]),
        };
        assert_eq!(
            discover_projects(&source, 1).await.unwrap(),
            vec!["00000001", "zzzzzzzz"]
        );
        assert_eq!(tenant_id("fn0-control").unwrap(), u64::MAX);
        assert!(normalize_project_subset(&["local".into()]).is_err());
        let (full_set, is_subset) = migration_projects(&source, 1, None).await.unwrap();
        assert!(!is_subset);
        assert_eq!(full_set, vec!["fn0-control", "00000001", "zzzzzzzz"]);
        let (subset, is_subset) = migration_projects(&source, 1, Some(vec!["zzzzzzzz".into()]))
            .await
            .unwrap();
        assert!(is_subset);
        assert_eq!(subset, vec!["zzzzzzzz"]);
    }

    #[tokio::test]
    async fn inventory_counts_source_rows_without_destination_access() {
        let source = FakeSource {
            databases: BTreeMap::from([
                (
                    "fn0-control".into(),
                    vec![
                        row(
                            "ProjectDoc/00000001",
                            "doc",
                            br#"{"project_id":"00000001"}"#,
                        ),
                        row("Settings/main", "doc", b"cfg"),
                    ],
                ),
                ("00000001".into(), vec![row("pk", "sk", &[0, 255])]),
            ]),
        };
        let report = inventory(&source, 1).await.unwrap();
        assert_eq!(report.project_count, 2);
        assert_eq!(report.totals.source_rows, 3);
        assert_eq!(report.projects[0].project_id, "fn0-control");
        assert_eq!(report.projects[0].source_rows, 2);
        assert_eq!(report.projects[1].project_id, "00000001");
        assert_eq!(report.projects[1].source_data_bytes, 2);
    }

    #[tokio::test]
    async fn project_discovery_rejects_duplicate_malformed_missing_and_invalid_ids() {
        let cases = [
            (
                "duplicate",
                vec![
                    row("ProjectDoc/a", "one", br#"{"project_id":"00000001"}"#),
                    row("ProjectDoc/b", "two", br#"{"project_id":"00000001"}"#),
                ],
                "duplicate project_id",
            ),
            (
                "malformed",
                vec![row("ProjectDoc/a", "one", b"{")],
                "malformed ProjectDoc JSON",
            ),
            (
                "missing",
                vec![row("ProjectDoc/a", "one", br#"{"name":"project"}"#)],
                "no string project_id",
            ),
            (
                "invalid",
                vec![row("ProjectDoc/a", "one", br#"{"project_id":"ABC12345"}"#)],
                "invalid project_id",
            ),
        ];
        for (case_name, rows, expected_error) in cases {
            let source = FakeSource {
                databases: BTreeMap::from([("fn0-control".into(), rows)]),
            };
            let error = discover_projects(&source, 1).await.unwrap_err();
            assert!(
                format!("{error:#}").contains(expected_error),
                "case {case_name} returned {error:#}"
            );
        }
    }

    #[test]
    fn parses_turso_raw_values_and_rejects_unexpected_types() {
        let rows = vec![vec![
            Value::Text { value: "pk".into() },
            Value::Text { value: "sk".into() },
            Value::Blob {
                value: vec![0, 255].into(),
            },
        ]];
        assert_eq!(
            parse_rows("00000001", rows).unwrap(),
            vec![row("pk", "sk", &[0, 255])]
        );
        assert!(
            parse_rows(
                "00000001",
                vec![vec![Value::Null, Value::Null, Value::Null]]
            )
            .unwrap_err()
            .to_string()
            .contains("pk is not TEXT")
        );
        assert!(
            parse_rows(
                "00000001",
                vec![vec![
                    Value::Text { value: "pk".into() },
                    Value::Integer { value: 1 },
                    Value::Blob {
                        value: vec![].into()
                    }
                ]]
            )
            .unwrap_err()
            .to_string()
            .contains("sk is not TEXT")
        );
        assert!(
            parse_rows(
                "00000001",
                vec![vec![
                    Value::Text { value: "pk".into() },
                    Value::Text { value: "sk".into() },
                    Value::Integer { value: 1 }
                ]]
            )
            .unwrap_err()
            .to_string()
            .contains("data is not BLOB")
        );
        assert!(
            parse_rows("00000001", vec![vec![Value::Text { value: "pk".into() }]])
                .unwrap_err()
                .to_string()
                .contains("expected 3 columns")
        );
    }

    #[test]
    fn missing_docs_table_match_is_exact() {
        assert!(is_missing_docs_table_error("no such table: docs"));
        assert!(is_missing_docs_table_error("no such table: main.docs"));
        assert!(is_missing_docs_table_error(
            "execute_raw_transactional error: no such table: docs"
        ));
        assert!(!is_missing_docs_table_error("no such table: docs_backup"));
        assert!(!is_missing_docs_table_error("no such column: version"));
    }

    #[test]
    fn dodb_config_parses_the_pem_root_certificate_file() {
        let certified = generate_simple_self_signed(vec!["dodb.internal".to_owned()]).unwrap();
        let root_certificate_pem = certified.cert.pem();
        assert!(
            dodb_config(
                "127.0.0.1:18445",
                "dodb.internal",
                root_certificate_pem.as_bytes()
            )
            .is_ok()
        );
    }

    #[test]
    fn source_result_treats_only_missing_docs_as_empty() {
        let missing = parse_source_outcome(
            "00000001",
            false,
            RawTransactionOutcome::RolledBack {
                failed_statement_index: 0,
                error_message: "no such table: docs".to_owned(),
            },
        )
        .unwrap();
        assert_eq!(missing, SourcePage::MissingTable);
        let unexpected = parse_source_outcome(
            "00000001",
            false,
            RawTransactionOutcome::RolledBack {
                failed_statement_index: 0,
                error_message: "permission denied".to_owned(),
            },
        )
        .unwrap_err();
        assert!(format!("{unexpected:#}").contains("permission denied"));
        assert!(
            parse_source_outcome(
                "00000001",
                true,
                RawTransactionOutcome::RolledBack {
                    failed_statement_index: 0,
                    error_message: "no such table: docs".to_owned(),
                },
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn raw_readonly_api_rejects_non_select_statements() {
        let database = doc_db::memory();
        let result = database
            .execute_raw_transactional_readonly(&[RawStatement {
                sql: "INSERT INTO docs (pk) VALUES ('mutate')".to_owned(),
                args: vec![],
            }])
            .await;
        let error = match result {
            Ok(_) => panic!("non-SELECT statement was accepted"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("accepts SELECT statements only"));
    }
}
