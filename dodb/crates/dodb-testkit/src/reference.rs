use std::collections::BTreeMap;

use dodb_core::{
    DocumentKey, Error, Lsn, PrimaryKey, ReadSet, Result, Revision, RevisionState, SortKey,
    TransactionCondition, TransactionMutation, TransactionRequest, TxnId, WriteIntent, WriteSet,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Document {
    pub key: DocumentKey,
    pub value: Vec<u8>,
    pub revision: Revision,
}

#[derive(Clone, Debug)]
pub struct ReferenceDb {
    states: BTreeMap<DocumentKey, RevisionState>,
    next_lsn: Lsn,
    next_txn_id: TxnId,
}

impl Default for ReferenceDb {
    fn default() -> Self {
        Self::new()
    }
}

impl ReferenceDb {
    pub fn new() -> Self {
        Self {
            states: BTreeMap::new(),
            next_lsn: Lsn::new(1),
            next_txn_id: TxnId::new(1),
        }
    }

    pub fn get(&self, key: &DocumentKey) -> RevisionState {
        self.states
            .get(key)
            .cloned()
            .unwrap_or_else(|| RevisionState::missing(Revision::ZERO))
    }

    pub fn put(&mut self, key: DocumentKey, value: impl Into<Vec<u8>>) -> Result<Lsn> {
        let mut transaction = self.begin();
        transaction.put(self, key, value);
        self.commit(transaction)?.ok_or_else(|| {
            Error::invariant("a transaction containing a put produced no commit LSN")
        })
    }

    pub fn delete(&mut self, key: DocumentKey) -> Result<Lsn> {
        let mut transaction = self.begin();
        transaction.delete(self, key);
        self.commit(transaction)?.ok_or_else(|| {
            Error::invariant("a transaction containing a delete produced no commit LSN")
        })
    }

    pub fn begin(&mut self) -> ReferenceTransaction {
        let txn_id = self.next_txn_id;
        self.next_txn_id = TxnId::new(
            txn_id
                .get()
                .checked_add(1)
                .expect("transaction id exhaustion is an internal test-model invariant"),
        );
        ReferenceTransaction {
            txn_id,
            conditions: Vec::new(),
            read_set: ReadSet::default(),
            write_set: WriteSet::default(),
        }
    }

    /// Validates every point in the read set before applying any write.
    /// Therefore a successful multi-key commit is atomic in this model.
    pub fn commit(&mut self, transaction: ReferenceTransaction) -> Result<Option<Lsn>> {
        let conditions = transaction
            .read_set
            .iter()
            .map(|(key, revision)| TransactionCondition::RevisionEquals {
                key: key.clone(),
                expected_revision: *revision,
            })
            .chain(transaction.conditions.iter().cloned())
            .collect();
        let mutations = transaction
            .write_set
            .iter()
            .map(|(key, intent)| match intent {
                WriteIntent::Put(value) => TransactionMutation::Put {
                    key: key.clone(),
                    value: value.clone(),
                },
                WriteIntent::Delete => TransactionMutation::Delete { key: key.clone() },
            })
            .collect();
        self.transact(TransactionRequest::new(conditions, mutations))
    }

    /// Applies one validated request atomically at the reference model's
    /// serialization point.
    pub fn transact(&mut self, request: TransactionRequest) -> Result<Option<Lsn>> {
        self.transact_at(request, self.next_lsn)
    }

    /// Applies a request using a commit LSN supplied by a physical engine.
    /// This lets differential tests replay real WAL commit identities while
    /// the ordinary reference path continues to use synthetic LSNs.
    pub fn transact_at(
        &mut self,
        request: TransactionRequest,
        commit_lsn: Lsn,
    ) -> Result<Option<Lsn>> {
        request.validate()?;
        for condition in &request.conditions {
            let actual = self.get(condition.key());
            let satisfied = match condition {
                TransactionCondition::RevisionEquals {
                    expected_revision, ..
                } => actual.revision() == *expected_revision,
                TransactionCondition::Exists { .. } => !actual.is_missing(),
                TransactionCondition::NotExists { .. } => actual.is_missing(),
            };
            if !satisfied {
                return Err(Error::conflict(dodb_core::TransactionConflict {
                    key: condition.key().clone(),
                    expected: condition.expectation(),
                    actual: actual.observed(),
                }));
            }
        }

        if request.mutations.is_empty() {
            return Ok(None);
        }
        if commit_lsn == Lsn::ZERO {
            return Err(Error::invalid_request(
                "a committed transaction LSN must be non-zero",
            ));
        }

        let revision = Revision::from(commit_lsn);
        for mutation in &request.mutations {
            let (key, state) = match mutation {
                TransactionMutation::Put { key, value } => {
                    (key, RevisionState::present(value.clone(), revision))
                }
                TransactionMutation::Delete { key } => (key, RevisionState::missing(revision)),
            };
            self.states.insert(key.clone(), state);
        }
        let next_lsn = Lsn::new(
            commit_lsn
                .get()
                .checked_add(1)
                .ok_or_else(|| Error::invariant("synthetic LSN exhausted"))?,
        );
        if next_lsn > self.next_lsn {
            self.next_lsn = next_lsn;
        }
        Ok(Some(commit_lsn))
    }

    pub fn transact_get(&self, keys: &[DocumentKey]) -> Vec<RevisionState> {
        keys.iter().map(|key| self.get(key)).collect()
    }

    pub fn query(
        &self,
        pk: &PrimaryKey,
        exclusive_after_sk: Option<&SortKey>,
        limit: usize,
    ) -> Vec<Document> {
        self.states
            .iter()
            .filter_map(|(key, state)| {
                if &key.pk != pk || exclusive_after_sk.is_some_and(|cursor| &key.sk <= cursor) {
                    return None;
                }
                match state {
                    RevisionState::Present { value, revision } => Some(Document {
                        key: key.clone(),
                        value: value.clone(),
                        revision: *revision,
                    }),
                    RevisionState::Missing { .. } => None,
                }
            })
            .take(limit)
            .collect()
    }

    pub fn scan(&self, exclusive_after_key: Option<&DocumentKey>, limit: usize) -> Vec<Document> {
        self.states
            .iter()
            .filter_map(|(key, state)| {
                if exclusive_after_key.is_some_and(|cursor| key <= cursor) {
                    return None;
                }
                match state {
                    RevisionState::Present { value, revision } => Some(Document {
                        key: key.clone(),
                        value: value.clone(),
                        revision: *revision,
                    }),
                    RevisionState::Missing { .. } => None,
                }
            })
            .take(limit)
            .collect()
    }

    pub fn present_count(&self) -> usize {
        self.states
            .values()
            .filter(|state| matches!(state, RevisionState::Present { .. }))
            .count()
    }
}

pub struct ReferenceTransaction {
    pub txn_id: TxnId,
    pub conditions: Vec<TransactionCondition>,
    pub read_set: ReadSet,
    pub write_set: WriteSet,
}

impl ReferenceTransaction {
    pub fn read(&mut self, db: &ReferenceDb, key: &DocumentKey) -> RevisionState {
        let observed = db.get(key);
        self.read_set.record(key.clone(), observed.revision());

        match self.write_set.get(key) {
            Some(WriteIntent::Put(value)) => RevisionState::present(
                value.clone(),
                self.read_set
                    .get(key)
                    .expect("write keys are always recorded in the read set"),
            ),
            Some(WriteIntent::Delete) => RevisionState::missing(
                self.read_set
                    .get(key)
                    .expect("write keys are always recorded in the read set"),
            ),
            None => observed,
        }
    }

    pub fn put(&mut self, db: &ReferenceDb, key: DocumentKey, value: impl Into<Vec<u8>>) {
        self.observe(db, &key);
        self.write_set.put(key, value);
    }

    pub fn delete(&mut self, db: &ReferenceDb, key: DocumentKey) {
        self.observe(db, &key);
        self.write_set.delete(key);
    }

    pub fn revision_equals(&mut self, key: DocumentKey, expected_revision: Revision) {
        self.conditions.push(TransactionCondition::RevisionEquals {
            key,
            expected_revision,
        });
    }

    pub fn exists(&mut self, key: DocumentKey) {
        self.conditions.push(TransactionCondition::Exists { key });
    }

    pub fn not_exists(&mut self, key: DocumentKey) {
        self.conditions
            .push(TransactionCondition::NotExists { key });
    }

    fn observe(&mut self, db: &ReferenceDb, key: &DocumentKey) {
        let state = db.get(key);
        self.read_set.record(key.clone(), state.revision());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodb_core::ObservedState;

    fn key(pk: u8, sk: u8) -> DocumentKey {
        DocumentKey::new(vec![pk], vec![sk])
    }

    #[test]
    fn missing_state_has_zero_revision_until_deleted() {
        let mut db = ReferenceDb::new();
        let document = key(1, 1);
        assert_eq!(db.get(&document), RevisionState::missing(Revision::ZERO));
        db.delete(document.clone()).unwrap();
        assert_eq!(db.get(&document), RevisionState::missing(Revision::new(1)));
    }

    #[test]
    fn write_write_conflict_is_detected() {
        let mut db = ReferenceDb::new();
        let a = key(1, 1);
        db.put(a.clone(), b"a").unwrap();
        let mut t1 = db.begin();
        match t1.read(&db, &a) {
            RevisionState::Present { revision, .. } => assert_eq!(revision, Revision::new(1)),
            RevisionState::Missing { .. } => panic!("seed value should be present"),
        }
        let mut t2 = db.begin();
        t2.put(&db, a.clone(), b"new-a");
        db.commit(t2).unwrap();
        t1.put(&db, a.clone(), b"t1-a");
        assert!(matches!(db.commit(t1), Err(Error::Conflict(_))));
    }

    #[test]
    fn read_write_conflict_is_detected_even_when_writing_another_key() {
        let mut db = ReferenceDb::new();
        let a = key(1, 1);
        let b = key(1, 2);
        db.put(a.clone(), b"a").unwrap();
        let mut t1 = db.begin();
        t1.read(&db, &a);
        let mut t2 = db.begin();
        t2.put(&db, a.clone(), b"changed");
        db.commit(t2).unwrap();
        t1.put(&db, b.clone(), b"b");
        assert!(matches!(db.commit(t1), Err(Error::Conflict(_))));
        assert_eq!(db.get(&b), RevisionState::missing(Revision::ZERO));
    }

    #[test]
    fn missing_aba_is_detected() {
        let mut db = ReferenceDb::new();
        let x = key(2, 1);
        let y = key(2, 2);
        let mut t1 = db.begin();
        assert_eq!(t1.read(&db, &x), RevisionState::missing(Revision::ZERO));
        db.put(x.clone(), b"x").unwrap();
        db.delete(x.clone()).unwrap();
        assert_eq!(db.get(&x), RevisionState::missing(Revision::new(2)));
        t1.put(&db, y.clone(), b"y");
        assert!(matches!(db.commit(t1), Err(Error::Conflict(_))));
        assert_eq!(db.get(&y), RevisionState::missing(Revision::ZERO));
    }

    #[test]
    fn disjoint_transactions_do_not_conflict() {
        let mut db = ReferenceDb::new();
        let a = key(3, 1);
        let b = key(3, 2);
        let mut t1 = db.begin();
        let mut t2 = db.begin();
        t1.read(&db, &a);
        t2.read(&db, &b);
        t1.put(&db, a.clone(), b"a");
        t2.put(&db, b.clone(), b"b");
        db.commit(t1).unwrap();
        db.commit(t2).unwrap();
        assert_eq!(db.present_count(), 2);
    }

    #[test]
    fn multiple_read_dependencies_are_all_validated() {
        let mut db = ReferenceDb::new();
        let a = key(4, 1);
        let b = key(4, 2);
        let c = key(4, 3);
        db.put(a.clone(), b"a").unwrap();
        db.put(b.clone(), b"b").unwrap();
        let mut t1 = db.begin();
        t1.read(&db, &a);
        t1.read(&db, &b);
        let mut t2 = db.begin();
        t2.put(&db, b.clone(), b"new-b");
        db.commit(t2).unwrap();
        t1.put(&db, c.clone(), b"c");
        assert!(matches!(db.commit(t1), Err(Error::Conflict(_))));
    }

    #[test]
    fn multi_key_commit_is_atomic_and_uses_one_lsn() {
        let mut db = ReferenceDb::new();
        let a = key(5, 1);
        let b = key(5, 2);
        let mut transaction = db.begin();
        transaction.put(&db, a.clone(), b"a");
        transaction.put(&db, b.clone(), b"b");
        assert_eq!(db.commit(transaction).unwrap(), Some(Lsn::new(1)));
        assert_eq!(db.get(&a).revision(), Revision::new(1));
        assert_eq!(db.get(&b).revision(), Revision::new(1));

        let c = key(5, 3);
        let mut doomed = db.begin();
        doomed.read(&db, &a);
        doomed.put(&db, b.clone(), b"overwrite-b");
        doomed.put(&db, c.clone(), b"c");
        db.put(a.clone(), b"external-a").unwrap();
        assert!(db.commit(doomed).is_err());
        assert_eq!(db.get(&c), RevisionState::missing(Revision::ZERO));
    }

    #[test]
    fn explicit_conditions_report_structured_conflicts_and_missing_semantics() {
        let mut db = ReferenceDb::new();
        let key = key(6, 1);
        let never = db.get(&key).revision();
        db.put(key.clone(), b"value").unwrap();
        let deleted_revision = db.delete(key.clone()).unwrap();

        let stale = TransactionRequest::new(
            vec![TransactionCondition::RevisionEquals {
                key: key.clone(),
                expected_revision: never,
            }],
            vec![TransactionMutation::Put {
                key: key.clone(),
                value: b"stale".to_vec(),
            }],
        );
        match db.transact(stale) {
            Err(Error::Conflict(conflict)) => {
                assert_eq!(conflict.key, key);
                assert_eq!(
                    conflict.actual,
                    ObservedState::missing(deleted_revision.into())
                );
            }
            other => panic!("expected a structured conflict, got {other:?}"),
        }

        let insert = TransactionRequest::new(
            vec![TransactionCondition::NotExists { key: key.clone() }],
            vec![TransactionMutation::Put {
                key: key.clone(),
                value: b"reinserted".to_vec(),
            }],
        );
        db.transact(insert).unwrap();
        let exists = TransactionRequest::new(
            vec![TransactionCondition::Exists { key: key.clone() }],
            vec![TransactionMutation::Delete { key }],
        );
        db.transact(exists).unwrap();
    }

    #[test]
    fn condition_only_transaction_has_no_logical_commit() {
        let mut db = ReferenceDb::new();
        let key = key(7, 1);
        let before_lsn = db.next_lsn;
        let request = TransactionRequest::new(
            vec![TransactionCondition::NotExists { key: key.clone() }],
            Vec::new(),
        );

        assert_eq!(db.transact(request).unwrap(), None);
        assert_eq!(db.get(&key), RevisionState::missing(Revision::ZERO));
        assert_eq!(db.next_lsn, before_lsn);
    }

    #[test]
    fn query_and_scan_are_ordered_and_exclusive() {
        let mut db = ReferenceDb::new();
        db.put(key(1, 2), b"two").unwrap();
        db.put(key(1, 1), b"one").unwrap();
        db.put(key(2, 1), b"other").unwrap();
        db.delete(key(1, 2)).unwrap();
        let rows = db.query(&PrimaryKey::new(vec![1]), None, 10);
        assert_eq!(
            rows.iter()
                .map(|row| row.key.sk.as_bytes())
                .collect::<Vec<_>>(),
            vec![&[1u8][..]]
        );
        assert!(
            db.query(&PrimaryKey::new(vec![1]), Some(&SortKey::new(vec![1])), 10)
                .is_empty()
        );
        let scan = db.scan(None, 10);
        assert_eq!(scan.len(), 2);
        assert!(
            db.scan(Some(&scan[0].key), 10)
                .iter()
                .all(|row| row.key > scan[0].key)
        );
    }

    #[test]
    fn deterministic_randomized_reference_sequence() {
        let seed = 0x5eed_cafe_u64;
        let mut rng = seed;
        let mut db = ReferenceDb::new();
        for _ in 0..4000 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let operation = rng % 7;
            let pk = (rng >> 8) as u8 % 4;
            let sk = (rng >> 16) as u8 % 16;
            let document_key = key(pk, sk);
            match operation {
                0 | 1 => {
                    db.put(document_key, vec![(rng >> 24) as u8]).unwrap();
                }
                2 => {
                    db.delete(document_key).unwrap();
                }
                3 => {
                    let _ = db.get(&document_key);
                }
                4 => {
                    let _ = db.query(&PrimaryKey::new(vec![pk]), None, 5);
                }
                5 => {
                    let _ = db.scan(None, 5);
                }
                _ => {
                    let mut transaction = db.begin();
                    transaction.read(&db, &document_key);
                    transaction.put(&db, key(pk, sk.wrapping_add(1)), b"txn");
                    db.commit(transaction).unwrap();
                }
            }

            let all = db.scan(None, usize::MAX);
            assert_eq!(all.len(), db.present_count());
            for window in all.windows(2) {
                assert!(window[0].key < window[1].key);
            }
            for row in &all {
                assert_ne!(row.revision, Revision::ZERO);
                assert_eq!(
                    db.get(&row.key),
                    RevisionState::present(row.value.clone(), row.revision)
                );
            }
        }
    }
}
