use std::path::Path;

use dodb_core::{Error, TransactionMutation, TransactionRequest};
use dodb_service::{ExecutionBudget, Request, Response, TransactionOutcome};

use super::{
    ConditionKeyLockMode, DEFAULT_REVISION_RANGE_SIZE, EngineHealthObserver, RevisionAllocatorMode,
    TenantDatabase, enforce_response_budget,
};

pub struct SynchronousRocksDbTenant {
    database: TenantDatabase,
}

impl SynchronousRocksDbTenant {
    pub fn open(path: &Path, allocator: RevisionAllocatorMode) -> Result<Self, Error> {
        Self::open_with_condition_key_lock_mode(path, allocator, ConditionKeyLockMode::Exclusive)
    }

    pub fn open_with_condition_key_lock_mode(
        path: &Path,
        allocator: RevisionAllocatorMode,
        condition_key_lock_mode: ConditionKeyLockMode,
    ) -> Result<Self, Error> {
        Ok(Self {
            database: TenantDatabase::open_with_modes(
                path,
                allocator,
                DEFAULT_REVISION_RANGE_SIZE,
                condition_key_lock_mode,
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
                budget,
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

    pub fn engine_health_observer(&self) -> EngineHealthObserver {
        self.engine_health_observer_named("tenant_0")
    }

    pub fn engine_health_observer_named(
        &self,
        database_name: impl Into<String>,
    ) -> EngineHealthObserver {
        let mut observer = EngineHealthObserver::new();
        observer.add_database(database_name.into(), &self.database.handle);
        observer
    }
}

pub fn materialized_transaction_observation(
    encoded: &[u8],
) -> Result<dodb_core::ObservedState, Error> {
    super::decode_state(encoded).map(|state| state.observed())
}

pub fn header_transaction_observation(encoded: &[u8]) -> Result<dodb_core::ObservedState, Error> {
    let stored_length = encoded.len();
    let prefix = &encoded[..encoded.len().min(10)];
    if prefix.len() < 10 || prefix[0] != super::VALUE_FORMAT {
        return Err(Error::corruption(
            "unsupported RocksDB logical value format",
        ));
    }
    let revision = dodb_core::Revision::new(u64::from_be_bytes(
        prefix[2..10]
            .try_into()
            .map_err(|_| Error::corruption("invalid RocksDB revision bytes"))?,
    ));
    match prefix[1] {
        super::PRESENT_TAG => Ok(dodb_core::ObservedState::present(revision)),
        super::MISSING_TAG if stored_length == 10 => {
            Ok(dodb_core::ObservedState::missing(revision))
        }
        super::MISSING_TAG => Err(Error::corruption("tombstone contains value bytes")),
        _ => Err(Error::corruption("unknown RocksDB logical value state")),
    }
}
