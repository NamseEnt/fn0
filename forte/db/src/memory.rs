use crate::turso::StoredDoc;
use crate::{BatchOp, DbOp, DbResult};
use anyhow::{Result, bail};
use bytes::Bytes;
use libsql_hrana::proto::*;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Clone)]
struct MemDoc {
    data: Vec<u8>,
    version: i64,
}

type Store = BTreeMap<(String, String), MemDoc>;

#[derive(Clone)]
pub(crate) struct MemoryDatabase {
    store: Rc<RefCell<Store>>,
}

impl MemoryDatabase {
    pub(crate) fn new() -> Self {
        Self {
            store: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    pub(crate) async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        let store = self.store.borrow();
        Ok(store
            .get(&(pk.to_string(), sk.to_string()))
            .map(|doc| doc.data.clone().into()))
    }

    pub(crate) async fn get_with_version(&self, pk: &str, sk: &str) -> Result<Option<StoredDoc>> {
        let store = self.store.borrow();
        Ok(store
            .get(&(pk.to_string(), sk.to_string()))
            .map(|doc| StoredDoc {
                data: doc.data.clone().into(),
                version: doc.version,
            }))
    }

    pub(crate) async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        upsert(&mut self.store.borrow_mut(), pk, sk, data);
        Ok(())
    }

    pub(crate) async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        self.store
            .borrow_mut()
            .remove(&(pk.to_string(), sk.to_string()));
        Ok(())
    }

    pub(crate) async fn query<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        pk: S1,
        after_sk: Option<S2>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        let store = self.store.borrow();
        Ok(query_store(
            &store,
            pk.as_ref(),
            after_sk.as_ref().map(|s| s.as_ref()),
            limit,
        ))
    }

    pub(crate) async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        let store = self.store.borrow();
        Ok(scan_store(&store, after, limit))
    }

    pub(crate) async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        let mut store = self.store.borrow_mut();
        for op in ops {
            match op {
                BatchOp::Put { pk, sk, data } => {
                    upsert(&mut store, pk, sk, data);
                }
                BatchOp::Delete { pk, sk } => {
                    store.remove(&(pk.to_string(), sk.to_string()));
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn execute_raw(
        &self,
        sql: &str,
        args: Vec<Value>,
        want_rows: bool,
    ) -> Result<Vec<Vec<Value>>> {
        let result = execute_sql_on_store(&mut self.store.borrow_mut(), sql, args)?;
        if want_rows {
            Ok(result.rows.into_iter().map(|row| row.values).collect())
        } else {
            Ok(vec![])
        }
    }

    pub(crate) async fn execute_ops(&self, ops: Vec<DbOp>) -> Result<Vec<DbResult>> {
        let mut results = Vec::new();
        for op in &ops {
            match op {
                DbOp::Get { pk, sk } => {
                    let data = self
                        .store
                        .borrow()
                        .get(&(pk.clone(), sk.clone()))
                        .map(|doc| Bytes::from(doc.data.clone()));
                    results.push(DbResult::Single(data));
                }
                DbOp::Query {
                    pk,
                    after_sk,
                    limit,
                } => {
                    let store = self.store.borrow();
                    let items =
                        query_store(&store, pk, after_sk.as_deref(), limit.unwrap_or(usize::MAX));
                    results.push(DbResult::Multiple(items));
                }
                DbOp::Put { pk, sk, data } => {
                    upsert(&mut self.store.borrow_mut(), pk, sk, data);
                    results.push(DbResult::Done);
                }
                DbOp::Delete { pk, sk } => {
                    self.store.borrow_mut().remove(&(pk.clone(), sk.clone()));
                    results.push(DbResult::Done);
                }
            }
        }
        Ok(results)
    }

    pub(crate) async fn transaction(&self) -> Result<MemoryTransaction> {
        Ok(MemoryTransaction {
            db: self.clone(),
            working: self.store.borrow().clone(),
        })
    }

    pub(crate) async fn begin_immediate_with_reads(
        &self,
        keys: &[(String, String)],
    ) -> Result<(MemoryTransaction, Vec<Option<StoredDoc>>)> {
        let working = self.store.borrow().clone();
        let docs = keys
            .iter()
            .map(|(pk, sk)| {
                working.get(&(pk.clone(), sk.clone())).map(|doc| StoredDoc {
                    data: doc.data.clone().into(),
                    version: doc.version,
                })
            })
            .collect();
        Ok((
            MemoryTransaction {
                db: self.clone(),
                working,
            },
            docs,
        ))
    }
}

pub(crate) struct MemoryTransaction {
    db: MemoryDatabase,
    working: Store,
}

impl MemoryTransaction {
    pub(crate) async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        Ok(self
            .working
            .get(&(pk.to_string(), sk.to_string()))
            .map(|doc| doc.data.clone().into()))
    }

    pub(crate) async fn get_with_version(
        &mut self,
        pk: &str,
        sk: &str,
    ) -> Result<Option<StoredDoc>> {
        Ok(self
            .working
            .get(&(pk.to_string(), sk.to_string()))
            .map(|doc| StoredDoc {
                data: doc.data.clone().into(),
                version: doc.version,
            }))
    }

    pub(crate) async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        upsert(&mut self.working, pk, sk, data);
        Ok(())
    }

    pub(crate) async fn delete(&mut self, pk: &str, sk: &str) -> Result<()> {
        self.working.remove(&(pk.to_string(), sk.to_string()));
        Ok(())
    }

    pub(crate) async fn execute_stmt(
        &mut self,
        sql: &str,
        args: Vec<Value>,
        _want_rows: bool,
    ) -> Result<StmtResult> {
        execute_sql_on_store(&mut self.working, sql, args)
    }

    pub(crate) async fn commit(self) -> Result<()> {
        *self.db.store.borrow_mut() = self.working;
        Ok(())
    }

    pub(crate) async fn rollback(self) -> Result<()> {
        Ok(())
    }

    pub(crate) async fn batch_get_with_version(
        &mut self,
        keys: &[(String, String)],
    ) -> Result<Vec<Option<StoredDoc>>> {
        let docs = keys
            .iter()
            .map(|(pk, sk)| {
                self.working
                    .get(&(pk.clone(), sk.clone()))
                    .map(|doc| StoredDoc {
                        data: doc.data.clone().into(),
                        version: doc.version,
                    })
            })
            .collect();
        Ok(docs)
    }

    pub(crate) async fn apply_writes_and_commit(
        &mut self,
        writes: &[crate::WriteOp],
    ) -> Result<crate::CommitOutcome> {
        use crate::WriteOp;

        let mut staged = self.working.clone();
        let mut affected_counts: Vec<u64> = Vec::with_capacity(writes.len());
        let mut conflict: Option<crate::ConflictInfo> = None;

        for (i, op) in writes.iter().enumerate() {
            match op {
                WriteOp::Insert { pk, sk, data } => {
                    let key = (pk.clone(), sk.clone());
                    if staged.contains_key(&key) {
                        conflict = Some(crate::ConflictInfo {
                            step_index: i,
                            message: format!("UNIQUE constraint failed: docs.pk, docs.sk ({pk}/{sk})"),
                        });
                        affected_counts.push(0);
                        break;
                    }
                    staged.insert(
                        key,
                        MemDoc {
                            data: data.clone(),
                            version: 0,
                        },
                    );
                    affected_counts.push(1);
                }
                WriteOp::Update {
                    pk,
                    sk,
                    expected_version,
                    data,
                } => {
                    let key = (pk.clone(), sk.clone());
                    match staged.get_mut(&key) {
                        Some(doc) if doc.version == *expected_version => {
                            doc.data = data.clone();
                            doc.version += 1;
                            affected_counts.push(1);
                        }
                        _ => affected_counts.push(0),
                    }
                }
                WriteOp::Delete {
                    pk,
                    sk,
                    expected_version,
                } => {
                    let key = (pk.clone(), sk.clone());
                    match staged.get(&key) {
                        Some(doc) if doc.version == *expected_version => {
                            staged.remove(&key);
                            affected_counts.push(1);
                        }
                        _ => affected_counts.push(0),
                    }
                }
            }
        }

        while affected_counts.len() < writes.len() {
            affected_counts.push(0);
        }

        if conflict.is_none() {
            *self.db.store.borrow_mut() = staged;
            self.working.clear();
        }

        Ok(crate::CommitOutcome {
            affected_counts,
            conflict,
        })
    }
}

// --- Helper functions ---

fn upsert(store: &mut Store, pk: &str, sk: &str, data: &[u8]) {
    let key = (pk.to_string(), sk.to_string());
    let new_version = store.get(&key).map_or(0, |doc| doc.version + 1);
    store.insert(
        key,
        MemDoc {
            data: data.to_vec(),
            version: new_version,
        },
    );
}

fn query_store(
    store: &Store,
    pk: &str,
    after_sk: Option<&str>,
    limit: usize,
) -> Vec<(String, Bytes)> {
    let mut items = Vec::new();
    for ((k_pk, k_sk), doc) in store.iter() {
        if k_pk != pk {
            continue;
        }
        if let Some(after) = after_sk
            && k_sk.as_str() <= after
        {
            continue;
        }
        items.push((k_sk.clone(), Bytes::from(doc.data.clone())));
        if items.len() >= limit {
            break;
        }
    }
    items
}

fn scan_store(
    store: &Store,
    after: Option<(&str, &str)>,
    limit: usize,
) -> Vec<(String, String, Bytes)> {
    let mut items = Vec::new();
    for ((k_pk, k_sk), doc) in store.iter() {
        if let Some((after_pk, after_sk)) = after
            && (k_pk.as_str(), k_sk.as_str()) <= (after_pk, after_sk)
        {
            continue;
        }
        items.push((k_pk.clone(), k_sk.clone(), Bytes::from(doc.data.clone())));
        if items.len() >= limit {
            break;
        }
    }
    items
}

fn extract_text(value: &Value) -> Result<String> {
    match value {
        Value::Text { value } => Ok(value.to_string()),
        _ => bail!("memory backend: expected Text value, got {:?}", value),
    }
}

fn extract_blob(value: &Value) -> Result<Vec<u8>> {
    match value {
        Value::Blob { value } => Ok(value.to_vec()),
        _ => bail!("memory backend: expected Blob value, got {:?}", value),
    }
}

fn extract_integer(value: &Value) -> Result<i64> {
    match value {
        Value::Integer { value } => Ok(*value),
        _ => bail!("memory backend: expected Integer value, got {:?}", value),
    }
}

fn empty_result() -> StmtResult {
    StmtResult {
        cols: vec![],
        rows: vec![],
        affected_row_count: 0,
        last_insert_rowid: None,
        replication_index: None,
        rows_read: 0,
        rows_written: 0,
        query_duration_ms: 0.0,
    }
}

/// Executes a SQL statement against the in-memory store by matching known
/// query patterns used internally by forte-db.
///
/// Supported patterns:
/// - Schema DDL (CREATE TABLE, ALTER TABLE) → no-op
/// - Transaction control (BEGIN, COMMIT, ROLLBACK) → no-op
/// - All standard CRUD on the `docs` table (SELECT, INSERT/UPSERT, UPDATE, DELETE)
/// - Conditional writes with version checks (used by the optimistic transaction system)
fn execute_sql_on_store(store: &mut Store, sql: &str, args: Vec<Value>) -> Result<StmtResult> {
    let sql = sql.trim();

    // Schema operations → no-op
    if sql.starts_with("CREATE TABLE") || sql.starts_with("ALTER TABLE") {
        return Ok(empty_result());
    }

    // Transaction control → no-op (handled at MemoryTransaction level)
    if sql == "BEGIN" || sql == "BEGIN TRANSACTION" || sql == "COMMIT" || sql == "ROLLBACK" {
        return Ok(empty_result());
    }

    // SELECT data FROM docs WHERE pk = ? AND sk = ?
    if sql == "SELECT data FROM docs WHERE pk = ? AND sk = ?" {
        let pk = extract_text(&args[0])?;
        let sk = extract_text(&args[1])?;
        return match store.get(&(pk, sk)) {
            Some(doc) => Ok(StmtResult {
                rows: vec![Row {
                    values: vec![Value::Blob {
                        value: doc.data.clone().into(),
                    }],
                }],
                ..empty_result()
            }),
            None => Ok(empty_result()),
        };
    }

    // SELECT data, version FROM docs WHERE pk = ? AND sk = ?
    if sql == "SELECT data, version FROM docs WHERE pk = ? AND sk = ?" {
        let pk = extract_text(&args[0])?;
        let sk = extract_text(&args[1])?;
        return match store.get(&(pk, sk)) {
            Some(doc) => Ok(StmtResult {
                rows: vec![Row {
                    values: vec![
                        Value::Blob {
                            value: doc.data.clone().into(),
                        },
                        Value::Integer { value: doc.version },
                    ],
                }],
                ..empty_result()
            }),
            None => Ok(empty_result()),
        };
    }

    // UPSERT: INSERT ... ON CONFLICT ... DO UPDATE (from Database::put)
    if sql.starts_with("INSERT INTO docs (pk, sk, data, version) VALUES")
        && sql.contains("ON CONFLICT")
    {
        let pk = extract_text(&args[0])?;
        let sk = extract_text(&args[1])?;
        let data = extract_blob(&args[2])?;
        upsert(store, &pk, &sk, &data);
        return Ok(StmtResult {
            affected_row_count: 1,
            ..empty_result()
        });
    }

    // Conditional INSERT: INSERT ... SELECT ... WHERE NOT EXISTS (from Trx commit)
    if sql.starts_with("INSERT INTO docs (pk, sk, data, version) SELECT")
        && sql.contains("WHERE NOT EXISTS")
    {
        let pk = extract_text(&args[0])?;
        let sk = extract_text(&args[1])?;
        let data = extract_blob(&args[2])?;
        let key = (pk, sk);
        if store.contains_key(&key) {
            return Ok(StmtResult {
                affected_row_count: 0,
                ..empty_result()
            });
        }
        store.insert(key, MemDoc { data, version: 0 });
        return Ok(StmtResult {
            affected_row_count: 1,
            ..empty_result()
        });
    }

    // Conditional UPDATE with version check (from Trx commit)
    if sql.starts_with("UPDATE docs SET data")
        && sql.contains("version = version + 1")
        && sql.contains("AND version = ?")
    {
        let data = extract_blob(&args[0])?;
        let pk = extract_text(&args[1])?;
        let sk = extract_text(&args[2])?;
        let expected_version = extract_integer(&args[3])?;
        let key = (pk, sk);
        return match store.get(&key) {
            Some(doc) if doc.version == expected_version => {
                store.insert(
                    key,
                    MemDoc {
                        data,
                        version: expected_version + 1,
                    },
                );
                Ok(StmtResult {
                    affected_row_count: 1,
                    ..empty_result()
                })
            }
            _ => Ok(StmtResult {
                affected_row_count: 0,
                ..empty_result()
            }),
        };
    }

    // DELETE with version check (from Trx commit) — must be checked before simple DELETE
    if sql == "DELETE FROM docs WHERE pk = ? AND sk = ? AND version = ?" {
        let pk = extract_text(&args[0])?;
        let sk = extract_text(&args[1])?;
        let expected_version = extract_integer(&args[2])?;
        let key = (pk, sk);
        return match store.get(&key) {
            Some(doc) if doc.version == expected_version => {
                store.remove(&key);
                Ok(StmtResult {
                    affected_row_count: 1,
                    ..empty_result()
                })
            }
            _ => Ok(StmtResult {
                affected_row_count: 0,
                ..empty_result()
            }),
        };
    }

    // Simple DELETE
    if sql == "DELETE FROM docs WHERE pk = ? AND sk = ?" {
        let pk = extract_text(&args[0])?;
        let sk = extract_text(&args[1])?;
        let removed = store.remove(&(pk, sk)).is_some();
        return Ok(StmtResult {
            affected_row_count: if removed { 1 } else { 0 },
            ..empty_result()
        });
    }

    // SELECT sk, data FROM docs WHERE pk = ? [AND sk > ?] ORDER BY sk [LIMIT ?]
    if sql.starts_with("SELECT sk, data FROM docs") {
        let pk = extract_text(&args[0])?;
        let mut arg_idx = 1;

        let has_sk_filter = sql.contains("AND sk > ?");
        let after_sk = if has_sk_filter {
            let sk = extract_text(&args[arg_idx])?;
            arg_idx += 1;
            Some(sk)
        } else {
            None
        };

        let has_limit = sql.contains("LIMIT ?");
        let limit = if has_limit {
            extract_integer(&args[arg_idx])? as usize
        } else {
            usize::MAX
        };

        let items = query_store(store, &pk, after_sk.as_deref(), limit);
        let rows = items
            .into_iter()
            .map(|(sk, data)| Row {
                values: vec![
                    Value::Text { value: sk.into() },
                    Value::Blob { value: data },
                ],
            })
            .collect();

        return Ok(StmtResult {
            rows,
            ..empty_result()
        });
    }

    // SELECT pk, sk, data FROM docs [WHERE (pk, sk) > (?, ?)] ORDER BY pk, sk LIMIT ?
    if sql.starts_with("SELECT pk, sk, data FROM docs") {
        let has_after = sql.contains("(pk, sk) > (?, ?)");
        let mut arg_idx = 0;
        let after = if has_after {
            let pk = extract_text(&args[arg_idx])?;
            let sk = extract_text(&args[arg_idx + 1])?;
            arg_idx += 2;
            Some((pk, sk))
        } else {
            None
        };

        let limit = extract_integer(&args[arg_idx])? as usize;
        let after_ref = after.as_ref().map(|(pk, sk)| (pk.as_str(), sk.as_str()));
        let items = scan_store(store, after_ref, limit);
        let rows = items
            .into_iter()
            .map(|(pk, sk, data)| Row {
                values: vec![
                    Value::Text { value: pk.into() },
                    Value::Text { value: sk.into() },
                    Value::Blob { value: data },
                ],
            })
            .collect();

        return Ok(StmtResult {
            rows,
            ..empty_result()
        });
    }

    bail!(
        "Unsupported SQL in memory backend: {}. \
         The in-memory backend supports the standard forte-db operations. \
         Use a real database for arbitrary SQL.",
        sql
    )
}
