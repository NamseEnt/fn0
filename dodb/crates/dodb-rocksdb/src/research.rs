use std::path::Path;

use dodb_core::{Error, TransactionMutation, TransactionRequest};
use dodb_service::{ExecutionBudget, Request, Response, TransactionOutcome};

use super::{
    DEFAULT_REVISION_RANGE_SIZE, RevisionAllocatorMode, TenantDatabase, enforce_response_budget,
};

pub struct SynchronousRocksDbTenant {
    database: TenantDatabase,
}

impl SynchronousRocksDbTenant {
    pub fn open(path: &Path, allocator: RevisionAllocatorMode) -> Result<Self, Error> {
        Ok(Self {
            database: TenantDatabase::open_with_allocator(
                path,
                allocator,
                DEFAULT_REVISION_RANGE_SIZE,
            )?,
        })
    }

    pub fn execute(&self, request: Request, budget: ExecutionBudget) -> Result<Response, Error> {
        let response = match request {
            Request::Get { key } => Response::Get(self.database.get_with_metrics(
                dodb_core::TenantId::new(0),
                &key,
                None,
                None,
            )?),
            Request::Put { key, value } => {
                let result = self.database.transact_with_metrics(
                    TransactionRequest::new(
                        Vec::new(),
                        vec![TransactionMutation::Put { key, value }],
                    ),
                    None,
                    0,
                )?;
                Response::Put(
                    result
                        .revision
                        .ok_or_else(|| Error::invariant("put transaction returned no revision"))?,
                )
            }
            Request::Delete { key } => {
                let result = self.database.transact_with_metrics(
                    TransactionRequest::new(Vec::new(), vec![TransactionMutation::Delete { key }]),
                    None,
                    0,
                )?;
                Response::Delete(
                    result.revision.ok_or_else(|| {
                        Error::invariant("delete transaction returned no revision")
                    })?,
                )
            }
            Request::Transact { request } => {
                let result = self.database.transact_with_metrics(request, None, 0)?;
                Response::Transact(match result.commit_lsn {
                    Some(commit_lsn) => TransactionOutcome::committed(commit_lsn),
                    None => TransactionOutcome::conditions_satisfied(),
                })
            }
            Request::Query {
                pk,
                exclusive_after_sk,
                limit,
            } => {
                return self
                    .database
                    .query_with_metrics(
                        dodb_core::TenantId::new(0),
                        &pk,
                        exclusive_after_sk.as_ref(),
                        limit,
                        budget,
                        None,
                        None,
                    )
                    .map(Response::Query);
            }
            Request::Scan {
                exclusive_after_key,
                limit,
            } => {
                return self
                    .database
                    .scan(exclusive_after_key.as_ref(), limit, budget)
                    .map(Response::Scan);
            }
        };
        enforce_response_budget(&response, budget)?;
        Ok(response)
    }
}
