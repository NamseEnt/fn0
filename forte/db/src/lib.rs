mod turso;

use anyhow::Result;
use turso::{TursoDatabase, TursoTransaction};
use wstd::http::body::Bytes;

/// Batch operation for atomic execution
pub enum BatchOp<'a> {
    Put {
        pk: &'a str,
        sk: &'a str,
        data: &'a [u8],
    },
    Delete {
        pk: &'a str,
        sk: &'a str,
    },
}

/// Create a Turso database instance from environment variables.
/// # Environment Variables
/// - `TURSO_URL`: The URL of the Turso database.
/// - `TURSO_AUTH_TOKEN`: The authentication token for the Turso database.
pub fn turso() -> Database {
    let url = std::env::var("TURSO_URL").unwrap_or("http://127.0.0.1:8080".to_string());
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default();
    Database {
        inner: DatabaseInner::Turso(TursoDatabase::new(url, auth_token)),
    }
}

pub struct Database {
    inner: DatabaseInner,
}

impl Database {
    pub async fn get(&self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.get(pk, sk).await,
        }
    }

    pub async fn put(&self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.put(pk, sk, data).await,
        }
    }

    pub async fn delete(&self, pk: &str, sk: &str) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.delete(pk, sk).await,
        }
    }

    pub async fn query<S1: AsRef<str>, S2: AsRef<str>>(
        &self,
        pk: S1,
        after_sk: Option<S2>,
        limit: usize,
    ) -> Result<Vec<(String, Bytes)>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.query(pk, after_sk, limit).await,
        }
    }

    pub async fn scan(
        &self,
        after: Option<(&str, &str)>,
        limit: usize,
    ) -> Result<Vec<(String, String, Bytes)>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.scan(after, limit).await,
        }
    }

    /// Execute multiple operations atomically.
    /// All operations succeed or all fail (rollback on any error).
    pub async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.batch(ops).await,
        }
    }

    /// Begin an interactive transaction.
    /// Use commit() to save changes or rollback() to discard them.
    pub async fn transaction(&self) -> Result<Transaction<'_>> {
        match &self.inner {
            DatabaseInner::Turso(db) => Ok(Transaction {
                inner: TransactionInner::Turso(db.transaction().await?),
            }),
        }
    }
}

enum DatabaseInner {
    Turso(TursoDatabase),
}

/// An interactive transaction that allows multiple operations with explicit commit/rollback.
pub struct Transaction<'a> {
    inner: TransactionInner<'a>,
}

enum TransactionInner<'a> {
    Turso(TursoTransaction<'a>),
}

impl<'a> Transaction<'a> {
    pub async fn get(&mut self, pk: &str, sk: &str) -> Result<Option<Bytes>> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.get(pk, sk).await,
        }
    }

    pub async fn put(&mut self, pk: &str, sk: &str, data: &[u8]) -> Result<()> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.put(pk, sk, data).await,
        }
    }

    pub async fn delete(&mut self, pk: &str, sk: &str) -> Result<()> {
        match &mut self.inner {
            TransactionInner::Turso(tx) => tx.delete(pk, sk).await,
        }
    }

    /// Commit the transaction, making all changes permanent.
    pub async fn commit(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.commit().await,
        }
    }

    /// Rollback the transaction, discarding all changes.
    pub async fn rollback(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.rollback().await,
        }
    }
}
