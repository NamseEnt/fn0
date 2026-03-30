mod turso;

use anyhow::Result;
use turso::{TursoDatabase, TursoTransaction};
use wstd::http::body::Bytes;

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

pub fn turso() -> Database {
    let url = std::env::var("TURSO_URL").unwrap_or("http://127.0.0.1:8080".to_string());
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").unwrap_or_default();
    turso_with_config(url, auth_token)
}

pub fn turso_with_config(url: String, auth_token: String) -> Database {
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

    pub async fn batch(&self, ops: &[BatchOp<'_>]) -> Result<()> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.batch(ops).await,
        }
    }

    pub async fn transaction(&self) -> Result<Transaction<'_>> {
        match &self.inner {
            DatabaseInner::Turso(db) => Ok(Transaction {
                inner: TransactionInner::Turso(db.transaction().await?),
            }),
        }
    }

    pub async fn execute_ops(&self, ops: Vec<DbOp>) -> Result<Vec<DbResult>> {
        match &self.inner {
            DatabaseInner::Turso(db) => db.execute_ops(ops).await,
        }
    }
}

enum DatabaseInner {
    Turso(TursoDatabase),
}

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

    pub async fn commit(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.commit().await,
        }
    }

    pub async fn rollback(self) -> Result<()> {
        match self.inner {
            TransactionInner::Turso(tx) => tx.rollback().await,
        }
    }
}

pub enum DbOp {
    Get { pk: String, sk: String },
    Query { pk: String, after_sk: Option<String>, limit: Option<usize> },
    Put { pk: String, sk: String, data: Vec<u8> },
    Delete { pk: String, sk: String },
}

pub enum DbResult {
    Single(Option<Bytes>),
    Multiple(Vec<(String, Bytes)>),
    Done,
}

pub struct Prepared<O> {
    pub ops: Vec<DbOp>,
    pub parse: Box<dyn FnOnce(&mut std::vec::IntoIter<DbResult>) -> Result<O>>,
}

#[allow(async_fn_in_trait)]
pub trait DbRequest: Sized {
    type Output;
    fn prepare(self) -> Prepared<Self::Output>;

    async fn send(self) -> Result<Self::Output> {
        let prepared = self.prepare();
        let results = turso().execute_ops(prepared.ops).await?;
        let mut iter = results.into_iter();
        (prepared.parse)(&mut iter)
    }
}

macro_rules! impl_db_request_tuple {
    ($($T:ident),+) => {
        #[allow(non_snake_case)]
        impl<$($T: DbRequest),+> DbRequest for ($($T,)+)
        where $($T::Output: 'static),+
        {
            type Output = ($($T::Output,)+);
            fn prepare(self) -> Prepared<Self::Output> {
                let ($($T,)+) = self;
                $(let $T = $T.prepare();)+
                let mut ops = Vec::new();
                $(ops.extend($T.ops);)+
                Prepared {
                    ops,
                    parse: Box::new(move |iter| {
                        Ok(($(($T.parse)(iter)?,)+))
                    }),
                }
            }
        }
    };
}

impl_db_request_tuple!(A);
impl_db_request_tuple!(A, B);
impl_db_request_tuple!(A, B, C);
impl_db_request_tuple!(A, B, C, D);
impl_db_request_tuple!(A, B, C, D, E);
impl_db_request_tuple!(A, B, C, D, E, F);
impl_db_request_tuple!(A, B, C, D, E, F, G);
impl_db_request_tuple!(A, B, C, D, E, F, G, H);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_db_request_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

impl<T: DbRequest> DbRequest for Vec<T> where T::Output: 'static {
    type Output = Vec<T::Output>;
    fn prepare(self) -> Prepared<Self::Output> {
        let mut all_ops = Vec::new();
        let mut parsers: Vec<Box<dyn FnOnce(&mut std::vec::IntoIter<DbResult>) -> Result<T::Output>>> = Vec::new();
        for item in self {
            let p = item.prepare();
            all_ops.extend(p.ops);
            parsers.push(p.parse);
        }
        Prepared {
            ops: all_ops,
            parse: Box::new(move |iter| {
                parsers.into_iter().map(|p| p(iter)).collect()
            }),
        }
    }
}
