module common {
  type Key = KeyA | KeyB
  type Value = ValueA | ValueB
  type LogicalState = | Present((Value, int)) | Missing(int)
  type Condition = | RevisionEquals((Key, int)) | Exists(Key) | NotExists(Key)
  type Mutation = | Put((Key, Value)) | Delete(Key)
  type Transaction = { conditions: Set[Condition], mutations: Set[Mutation] }
  type TransactionPhase = Idle | Validating | Validated | Prepared | Committed | Conflict | ConditionSatisfied | InvalidRequest
  type TransactionOutcome = NoOutcome | MutationCommitted(int) | ConditionOnlySucceeded | ConflictResult | InvalidResult
  type CommitIdentity = NoCommit | HasCommit(int)

  pure val Keys: Set[Key] = Set(KeyA, KeyB)
  pure val Values: Set[Value] = Set(ValueA, ValueB)
  pure val InitialRevision = 0
  pure val InitialLsn = 1
  pure val MaximumSetupCommits = 2

  pure val InitialState: Key -> LogicalState = Keys.mapBy(key => Missing(InitialRevision))

  pure def mutationKey(mutation: Mutation): Key =
    match (mutation) {
      | Put(mutationPayload) => mutationPayload._1
      | Delete(key) => key
    }

  pure def conditionSatisfied(condition: Condition, state: Key -> LogicalState): bool =
    match (condition) {
      | RevisionEquals(conditionPayload) => stateRevision(state, conditionPayload._1) == conditionPayload._2
      | Exists(key) => match (state.get(key)) {
          | Present(_) => true
          | Missing(_) => false
        }
      | NotExists(key) => match (state.get(key)) {
          | Present(_) => false
          | Missing(_) => true
        }
    }

  pure def allConditionsSatisfied(conditions: Set[Condition], state: Key -> LogicalState): bool =
    conditions.forall(condition => conditionSatisfied(condition, state))

  pure def applyMutation(state: Key -> LogicalState, mutation: Mutation, commitLsn: int): Key -> LogicalState =
    match (mutation) {
      | Put(mutationPayload) => state.set(mutationPayload._1, Present((mutationPayload._2, commitLsn)))
      | Delete(key) => state.set(key, Missing(commitLsn))
    }

  pure def applyMutations(state: Key -> LogicalState, mutations: Set[Mutation], commitLsn: int): Key -> LogicalState =
    mutations.fold(state, (currentState, mutation) => applyMutation(currentState, mutation, commitLsn))

  pure def stateRevision(state: Key -> LogicalState, key: Key): int =
    match (state.get(key)) {
      | Present(statePayload) => statePayload._2
      | Missing(revision) => revision
    }

  pure def stateIsMissing(state: Key -> LogicalState, key: Key): bool =
    match (state.get(key)) {
      | Present(_) => false
      | Missing(_) => true
    }

  pure def stateWellFormed(state: Key -> LogicalState, nextLsn: int): bool =
    Keys.forall(key => {
      val revision = stateRevision(state, key)
      revision >= InitialRevision and revision < nextLsn
    })
}
