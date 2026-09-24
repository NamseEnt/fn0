module transaction_semantics {
  import common.* from "./common"

  pure val EmptyTransaction: Transaction = {
    conditions: Set(),
    mutations: Set(),
  }

  pure val ConditionOnlyExists: Transaction = {
    conditions: Set(Exists(KeyA)),
    mutations: Set(),
  }

  pure val ConditionOnlyMissingRevision: Transaction = {
    conditions: Set(RevisionEquals((KeyA, InitialRevision))),
    mutations: Set(),
  }

  pure val ConditionOnlyNotExists: Transaction = {
    conditions: Set(NotExists(KeyA)),
    mutations: Set(),
  }

  pure val PutOneKey: Transaction = {
    conditions: Set(RevisionEquals((KeyA, InitialRevision))),
    mutations: Set(Put((KeyA, ValueA))),
  }

  pure val PutTwoKeys: Transaction = {
    conditions: Set(
      RevisionEquals((KeyA, InitialRevision)),
      RevisionEquals((KeyB, InitialRevision)),
    ),
    mutations: Set(
      Put((KeyA, ValueA)),
      Put((KeyB, ValueB)),
    ),
  }

  pure val DeleteOneKey: Transaction = {
    conditions: Set(RevisionEquals((KeyA, 1))),
    mutations: Set(Delete(KeyA)),
  }

  pure val StaleMissingRevision: Transaction = {
    conditions: Set(RevisionEquals((KeyA, InitialRevision))),
    mutations: Set(Put((KeyB, ValueA))),
  }

  pure val Transactions: Set[Transaction] = Set(
    EmptyTransaction,
    ConditionOnlyExists,
    ConditionOnlyMissingRevision,
    ConditionOnlyNotExists,
    PutOneKey,
    PutTwoKeys,
    DeleteOneKey,
    StaleMissingRevision,
  )

  var committedState: Key -> LogicalState
  var baseState: Key -> LogicalState
  var stagedState: Key -> LogicalState
  var expectedPostState: Key -> LogicalState
  var activeTransaction: Transaction
  var activePhase: TransactionPhase
  var activeOutcome: TransactionOutcome
  var activeCommitIdentity: CommitIdentity
  var nextLsn: int
  var beforeNextLsn: int
  var setupCommitCount: int
  var beforeTransactionState: Key -> LogicalState

  action init = all {
    committedState' = InitialState,
    baseState' = InitialState,
    stagedState' = InitialState,
    expectedPostState' = InitialState,
    activeTransaction' = EmptyTransaction,
    activePhase' = Idle,
    activeOutcome' = NoOutcome,
    activeCommitIdentity' = NoCommit,
    nextLsn' = InitialLsn,
    beforeNextLsn' = InitialLsn,
    setupCommitCount' = 0,
    beforeTransactionState' = InitialState,
  }

  action setupPut = {
    nondet setupKey = Keys.oneOf()
    nondet setupValue = Values.oneOf()
    all {
      activePhase == Idle,
      setupCommitCount < MaximumSetupCommits,
      committedState' = committedState.set(setupKey, Present((setupValue, nextLsn))),
      nextLsn' = nextLsn + 1,
      setupCommitCount' = setupCommitCount + 1,
      baseState' = baseState,
      stagedState' = stagedState,
      expectedPostState' = expectedPostState,
      activeTransaction' = activeTransaction,
      activePhase' = activePhase,
      activeOutcome' = activeOutcome,
      activeCommitIdentity' = activeCommitIdentity,
      beforeNextLsn' = beforeNextLsn,
      beforeTransactionState' = beforeTransactionState,
    }
  }

  action setupDelete = {
    nondet setupKey = Keys.oneOf()
    all {
      activePhase == Idle,
      setupCommitCount < MaximumSetupCommits,
      committedState' = committedState.set(setupKey, Missing(nextLsn)),
      nextLsn' = nextLsn + 1,
      setupCommitCount' = setupCommitCount + 1,
      baseState' = baseState,
      stagedState' = stagedState,
      expectedPostState' = expectedPostState,
      activeTransaction' = activeTransaction,
      activePhase' = activePhase,
      activeOutcome' = activeOutcome,
      activeCommitIdentity' = activeCommitIdentity,
      beforeNextLsn' = beforeNextLsn,
      beforeTransactionState' = beforeTransactionState,
    }
  }

  action beginTransaction = {
    nondet selectedTransaction = Transactions.oneOf()
    all {
      activePhase == Idle,
      activeTransaction' = selectedTransaction,
      baseState' = committedState,
      stagedState' = committedState,
      expectedPostState' = committedState,
      beforeTransactionState' = committedState,
      beforeNextLsn' = nextLsn,
      activeCommitIdentity' = NoCommit,
      activeOutcome' = NoOutcome,
      activePhase' = if (selectedTransaction.conditions.size() == 0 and selectedTransaction.mutations.size() == 0)
        InvalidRequest
        else Validating,
      committedState' = committedState,
      nextLsn' = nextLsn,
      setupCommitCount' = setupCommitCount,
    }
  }

  action validateConditionOnly = all {
    activePhase == Validating,
    activeTransaction.mutations.size() == 0,
    allConditionsSatisfied(activeTransaction.conditions, baseState),
    committedState' = committedState,
    baseState' = baseState,
    stagedState' = stagedState,
    expectedPostState' = expectedPostState,
    activeTransaction' = activeTransaction,
    nextLsn' = nextLsn,
    beforeNextLsn' = beforeNextLsn,
    setupCommitCount' = setupCommitCount,
    beforeTransactionState' = beforeTransactionState,
    activePhase' = ConditionSatisfied,
    activeOutcome' = ConditionOnlySucceeded,
    activeCommitIdentity' = NoCommit,
  }

  action validateMutation = all {
    activePhase == Validating,
    activeTransaction.mutations.size() > 0,
    allConditionsSatisfied(activeTransaction.conditions, baseState),
    committedState' = committedState,
    baseState' = baseState,
    stagedState' = stagedState,
    expectedPostState' = expectedPostState,
    activeTransaction' = activeTransaction,
    activeOutcome' = activeOutcome,
    activeCommitIdentity' = activeCommitIdentity,
    nextLsn' = nextLsn,
    beforeNextLsn' = beforeNextLsn,
    setupCommitCount' = setupCommitCount,
    beforeTransactionState' = beforeTransactionState,
    activePhase' = Validated,
  }

  action rejectConflict = all {
    activePhase == Validating,
    not (allConditionsSatisfied(activeTransaction.conditions, baseState)),
    committedState' = committedState,
    baseState' = baseState,
    stagedState' = stagedState,
    expectedPostState' = expectedPostState,
    activeTransaction' = activeTransaction,
    nextLsn' = nextLsn,
    beforeNextLsn' = beforeNextLsn,
    setupCommitCount' = setupCommitCount,
    beforeTransactionState' = beforeTransactionState,
    activePhase' = Conflict,
    activeOutcome' = ConflictResult,
    activeCommitIdentity' = NoCommit,
  }

  action prepareTransaction = all {
    activePhase == Validated,
    stagedState' = applyMutations(baseState, activeTransaction.mutations, nextLsn),
    expectedPostState' = applyMutations(baseState, activeTransaction.mutations, nextLsn),
    committedState' = committedState,
    baseState' = baseState,
    activeTransaction' = activeTransaction,
    activeOutcome' = activeOutcome,
    nextLsn' = nextLsn,
    beforeNextLsn' = beforeNextLsn,
    setupCommitCount' = setupCommitCount,
    beforeTransactionState' = beforeTransactionState,
    activeCommitIdentity' = HasCommit(nextLsn),
    activePhase' = Prepared,
  }

  action commitPrepared = all {
    activePhase == Prepared,
    committedState' = stagedState,
    nextLsn' = nextLsn + 1,
    baseState' = baseState,
    stagedState' = stagedState,
    expectedPostState' = expectedPostState,
    activeTransaction' = activeTransaction,
    beforeNextLsn' = beforeNextLsn,
    setupCommitCount' = setupCommitCount,
    beforeTransactionState' = beforeTransactionState,
    activeCommitIdentity' = HasCommit(nextLsn),
    activeOutcome' = MutationCommitted(nextLsn),
    activePhase' = Committed,
  }

  action terminalStutter = all {
    activePhase == Committed
      or activePhase == Conflict
      or activePhase == ConditionSatisfied
      or activePhase == InvalidRequest,
    committedState' = committedState,
    baseState' = baseState,
    stagedState' = stagedState,
    expectedPostState' = expectedPostState,
    activeTransaction' = activeTransaction,
    activePhase' = activePhase,
    activeOutcome' = activeOutcome,
    activeCommitIdentity' = activeCommitIdentity,
    nextLsn' = nextLsn,
    beforeNextLsn' = beforeNextLsn,
    setupCommitCount' = setupCommitCount,
    beforeTransactionState' = beforeTransactionState,
  }

  action step = any {
    setupPut,
    setupDelete,
    beginTransaction,
    validateConditionOnly,
    validateMutation,
    rejectConflict,
    prepareTransaction,
    commitPrepared,
    terminalStutter,
  }

  val TypeOK =
    nextLsn > InitialRevision
      and beforeNextLsn > InitialRevision
      and setupCommitCount >= 0
      and setupCommitCount <= MaximumSetupCommits
      and stateWellFormed(committedState, nextLsn)
      and stateWellFormed(baseState, nextLsn)
      and stateWellFormed(stagedState, nextLsn + 1)

  val Atomicity =
    not (isMutationCommitted(activeOutcome)) or committedState == expectedPostState

  val ConflictHasNoEffect =
    activePhase != Conflict
      or (
        committedState == beforeTransactionState
          and nextLsn == beforeNextLsn
          and activeCommitIdentity == NoCommit
          and activeOutcome == ConflictResult
      )

  val OneRevisionPerCommit =
    match activeOutcome {
      | MutationCommitted(commitLsn) => activeTransaction.mutations.forall(mutation => stateRevision(committedState, mutationKey(mutation)) == commitLsn)
      | _ => true
    }

  val MissingRevisionPreserved =
    Keys.forall(key => stateRevision(committedState, key) >= InitialRevision)

  val ConditionOnlyHasNoCommit =
    activePhase != ConditionSatisfied
      or (
        committedState == beforeTransactionState
          and nextLsn == beforeNextLsn
          and activeCommitIdentity == NoCommit
          and activeOutcome == ConditionOnlySucceeded
      )

  val CommittedStateWellFormed =
    stateWellFormed(committedState, nextLsn)

  val EmptyRequestInvalid =
    activeTransaction != EmptyTransaction
      or activePhase == Idle
      or activePhase == InvalidRequest

  val ValidationBeforeMutation =
    not (
      activePhase == Validating
        or activePhase == Validated
        or activePhase == Conflict
        or activePhase == ConditionSatisfied
        or activePhase == InvalidRequest
    ) or stagedState == baseState

  val RevisionConditionSound =
    activePhase != Committed
      or allConditionsSatisfied(activeTransaction.conditions, beforeTransactionState)

  val AbaStaleRevisionRejected =
    beforeTransactionState.get(KeyA) != Missing(2)
      or activeTransaction.conditions.contains(RevisionEquals((KeyA, InitialRevision))) == false
      or activePhase != Committed

  val state_invariants = all {
    TypeOK,
    Atomicity,
    ConflictHasNoEffect,
    OneRevisionPerCommit,
    MissingRevisionPreserved,
    ConditionOnlyHasNoCommit,
    CommittedStateWellFormed,
    EmptyRequestInvalid,
    ValidationBeforeMutation,
    RevisionConditionSound,
    AbaStaleRevisionRejected,
  }

  pure def isMutationCommitted(outcome: TransactionOutcome): bool =
    match outcome {
      | MutationCommitted(_) => true
      | _ => false
    }

  pure val InitialStateTest =
    InitialState.get(KeyA) == Missing(InitialRevision)
      and InitialState.get(KeyB) == Missing(InitialRevision)
      and InitialLsn > InitialRevision

  pure val ConditionPredicatesTest =
    conditionSatisfied(NotExists(KeyA), InitialState)
      and conditionSatisfied(RevisionEquals((KeyA, InitialRevision)), InitialState)
      and not (conditionSatisfied(Exists(KeyA), InitialState))
      and not (conditionSatisfied(RevisionEquals((KeyA, 2)), InitialState))

  pure val MissingRevisionTest =
    stateIsMissing(InitialState.set(KeyA, Missing(2)), KeyA)
      and conditionSatisfied(NotExists(KeyA), InitialState.set(KeyA, Missing(2)))
      and not (conditionSatisfied(RevisionEquals((KeyA, InitialRevision)), InitialState.set(KeyA, Missing(2))))
      and not (conditionSatisfied(RevisionEquals((KeyA, 2)), InitialState))

  pure val ConditionOnlyContractTest =
    ConditionOnlyMissingRevision.mutations.size() == 0
      and ConditionOnlyMissingRevision.conditions.forall(condition => conditionSatisfied(condition, InitialState))
      and NoCommit == NoCommit
}
