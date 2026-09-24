use anyhow::{Context, Result, anyhow, bail};
use async_trait::async_trait;
use doc_db::{DodbConfig, DodbConnection, RawStatement, RawTransactionOutcome, Value};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::Cursor;
use std::net::SocketAddr;

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

#[derive(Clone)]
pub struct TursoSource {
    group_token: String,
    host_suffix: String,
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
            if let Some(last) = page.last() {
                self.cursor = Some((last.pk.clone(), last.sk.clone()));
            }
            if page.len() < self.page_size {
                self.finished = true;
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

    async fn test_connection() -> (
        DodbConnection,
        Arc<DodbServer<LocalTenantService>>,
        tokio::task::JoinHandle<Result<(), dodb_server::ServerError>>,
        tempfile::TempDir,
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
            vec![certificate],
        ))
        .await
        .unwrap();
        (connection, server, task, directory)
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
        let (connection, server, task, _directory) = test_connection().await;
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
        let (connection, server, task, _directory) = test_connection().await;
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
        let (connection, server, task, _directory) = test_connection().await;
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
        let (connection, server, task, _directory) = test_connection().await;
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
