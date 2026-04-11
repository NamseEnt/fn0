use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// Key for matching mock rules against operations.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MockKey {
    pub op: MockOp,
    pub pk: String,
    pub sk: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum MockOp {
    Get,
    Put,
    Delete,
}

/// The result a mock rule produces.
#[derive(Clone)]
pub(crate) enum MockResult {
    /// Return Ok with this data (for Get: Some(bytes), None means not found)
    OkGet(Option<Vec<u8>>),
    /// Return Ok(()) for put/delete
    OkVoid,
    /// Return Err with this message
    Err(String),
}

/// A single mock rule.
#[derive(Clone)]
pub(crate) struct MockRule {
    pub key: MockKey,
    pub result: MockResult,
}

/// Shared mock state stored inside a Database.
#[derive(Clone, Default)]
pub(crate) struct MockState {
    rules: Rc<RefCell<VecDeque<MockRule>>>,
}

impl MockState {
    pub(crate) fn push(&self, rule: MockRule) {
        self.rules.borrow_mut().push_back(rule);
    }

    /// Try to match and consume a rule for the given operation.
    pub(crate) fn try_match(&self, op: MockOp, pk: &str, sk: &str) -> Option<MockResult> {
        let mut rules = self.rules.borrow_mut();
        let pos = rules.iter().position(|r| {
            r.key.op == op && r.key.pk == pk && r.key.sk == sk
        })?;
        Some(rules.remove(pos).unwrap().result)
    }

    pub(crate) fn clear(&self) {
        self.rules.borrow_mut().clear();
    }
}

// --- Public builder API ---

/// Builder for configuring a mock rule on a Get operation.
pub struct MockGetBuilder<'a> {
    db: &'a crate::Database,
    pk: String,
    sk: String,
}

impl<'a> MockGetBuilder<'a> {
    pub(crate) fn new(db: &'a crate::Database, pk: String, sk: String) -> Self {
        Self { db, pk, sk }
    }

    /// Mock this get to return the given data.
    pub fn returns(self, data: impl Into<Vec<u8>>) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Get,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::OkGet(Some(data.into())),
        });
    }

    /// Mock this get to return None (not found).
    pub fn returns_none(self) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Get,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::OkGet(None),
        });
    }

    /// Mock this get to return an error.
    pub fn returns_err(self, msg: impl Into<String>) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Get,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::Err(msg.into()),
        });
    }
}

/// Builder for configuring a mock rule on a Put operation.
pub struct MockPutBuilder<'a> {
    db: &'a crate::Database,
    pk: String,
    sk: String,
}

impl<'a> MockPutBuilder<'a> {
    pub(crate) fn new(db: &'a crate::Database, pk: String, sk: String) -> Self {
        Self { db, pk, sk }
    }

    /// Mock this put to succeed.
    pub fn returns_ok(self) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Put,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::OkVoid,
        });
    }

    /// Mock this put to return an error.
    pub fn returns_err(self, msg: impl Into<String>) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Put,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::Err(msg.into()),
        });
    }
}

/// Builder for configuring a mock rule on a Delete operation.
pub struct MockDeleteBuilder<'a> {
    db: &'a crate::Database,
    pk: String,
    sk: String,
}

impl<'a> MockDeleteBuilder<'a> {
    pub(crate) fn new(db: &'a crate::Database, pk: String, sk: String) -> Self {
        Self { db, pk, sk }
    }

    /// Mock this delete to succeed.
    pub fn returns_ok(self) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Delete,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::OkVoid,
        });
    }

    /// Mock this delete to return an error.
    pub fn returns_err(self, msg: impl Into<String>) {
        self.db.add_mock_rule(MockRule {
            key: MockKey {
                op: MockOp::Delete,
                pk: self.pk,
                sk: self.sk,
            },
            result: MockResult::Err(msg.into()),
        });
    }
}
