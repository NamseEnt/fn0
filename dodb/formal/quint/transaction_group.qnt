module transaction_group {
  import common.* from "./common"

  type RequestSequence = SequenceA | SequenceB | SequenceC | SequenceD
  type RequestResult = RequestInvalid | RequestConflict | RequestConditionsSatisfied | RequestCommitted(int)
  type Phase = Processing | ReadyToPublish | Done
  type SerialEvaluation = {
    finalState: Key -> LogicalState,
    results: List[RequestResult],
    nextCommitOrdinal: int,
  }
  type ReferenceStep = {
    nextState: Key -> LogicalState,
    result: RequestResult,
    nextCommitOrdinal: int,
  }

  pure val EmptyTransaction: Transaction = {
    conditions: Set(),
    mutations: Set(),
  }

  pure val PutAValueA: Transaction = {
    conditions: Set(),
    mutations: Set(Put((KeyA, ValueA))),
  }

  pure val PutBValueA: Transaction = {
    conditions: Set(),
    mutations: Set(Put((KeyB, ValueA))),
  }

  pure val PutBValueB: Transaction = {
    conditions: Set(),
    mutations: Set(Put((KeyB, ValueB))),
  }

  pure val PutAValueB: Transaction = {
    conditions: Set(),
    mutations: Set(Put((KeyA, ValueB))),
  }

  pure val RequireARevisionZeroAndPutBValueA: Transaction = {
    conditions: Set(RevisionEquals((KeyA, 0))),
    mutations: Set(Put((KeyB, ValueA))),
  }

  pure val RequireARevisionOne: Transaction = {
    conditions: Set(RevisionEquals((KeyA, 1))),
    mutations: Set(),
  }

  pure val RequireAMissingAndPutAValueA: Transaction = {
    conditions: Set(NotExists(KeyA)),
    mutations: Set(Put((KeyA, ValueA))),
  }

  pure val RequireAMissingAndPutAValueB: Transaction = {
    conditions: Set(NotExists(KeyA)),
    mutations: Set(Put((KeyA, ValueB))),
  }

  pure def requestsFor(sequence: RequestSequence): List[Transaction] =
    match (sequence) {
      | SequenceA => List(PutAValueA, RequireARevisionZeroAndPutBValueA, PutBValueB)
      | SequenceB => List(PutAValueA, RequireARevisionOne, PutBValueB)
      | SequenceC => List(RequireAMissingAndPutAValueA, RequireAMissingAndPutAValueB)
      | SequenceD => List(EmptyTransaction, PutBValueA)
    }

  var activeSequence: RequestSequence
  var baseState: Key -> LogicalState
  var stagedState: Key -> LogicalState
  var publishedState: Key -> LogicalState
  var requests: List[Transaction]
  var index: int
  var results: List[RequestResult]
  var nextCommitOrdinal: int
  var phase: Phase

  action init = {
    nondet selectedSequence = Set(SequenceA, SequenceB, SequenceC, SequenceD).oneOf()
    all {
      activeSequence' = selectedSequence,
      baseState' = InitialState,
      stagedState' = InitialState,
      publishedState' = InitialState,
      requests' = requestsFor(selectedSequence),
      index' = 0,
      results' = List(),
      nextCommitOrdinal' = 1,
      phase' = Processing,
    }
  }

  action processInvalid = {
    val request = requests.nth(index)
    all {
      phase == Processing,
      index < requests.length(),
      request.conditions.size() + request.mutations.size() == 0,
      results' = results.append(RequestInvalid),
      index' = index + 1,
      stagedState' = stagedState,
      nextCommitOrdinal' = nextCommitOrdinal,
      phase' = phase,
      activeSequence' = activeSequence,
      baseState' = baseState,
      publishedState' = publishedState,
      requests' = requests,
    }
  }

  action processConflict = {
    val request = requests.nth(index)
    all {
      phase == Processing,
      index < requests.length(),
      request.conditions.size() + request.mutations.size() > 0,
      not (allConditionsSatisfied(request.conditions, stagedState)),
      results' = results.append(RequestConflict),
      index' = index + 1,
      stagedState' = stagedState,
      nextCommitOrdinal' = nextCommitOrdinal,
      phase' = phase,
      activeSequence' = activeSequence,
      baseState' = baseState,
      publishedState' = publishedState,
      requests' = requests,
    }
  }

  action processConditionOnly = {
    val request = requests.nth(index)
    all {
      phase == Processing,
      index < requests.length(),
      request.conditions.size() + request.mutations.size() > 0,
      allConditionsSatisfied(request.conditions, stagedState),
      request.mutations.size() == 0,
      results' = results.append(RequestConditionsSatisfied),
      index' = index + 1,
      stagedState' = stagedState,
      nextCommitOrdinal' = nextCommitOrdinal,
      phase' = phase,
      activeSequence' = activeSequence,
      baseState' = baseState,
      publishedState' = publishedState,
      requests' = requests,
    }
  }

  action processMutation = {
    val request = requests.nth(index)
    all {
      phase == Processing,
      index < requests.length(),
      request.conditions.size() + request.mutations.size() > 0,
      allConditionsSatisfied(request.conditions, stagedState),
      request.mutations.size() > 0,
      results' = results.append(RequestCommitted(nextCommitOrdinal)),
      index' = index + 1,
      stagedState' = applyMutations(stagedState, request.mutations, nextCommitOrdinal),
      nextCommitOrdinal' = nextCommitOrdinal + 1,
      phase' = phase,
      activeSequence' = activeSequence,
      baseState' = baseState,
      publishedState' = publishedState,
      requests' = requests,
    }
  }

  action finishProcessing = all {
    phase == Processing,
    index == requests.length(),
    activeSequence' = activeSequence,
    baseState' = baseState,
    stagedState' = stagedState,
    publishedState' = publishedState,
    requests' = requests,
    index' = index,
    results' = results,
    nextCommitOrdinal' = nextCommitOrdinal,
    phase' = ReadyToPublish,
  }

  action PublishGroup = all {
    phase == ReadyToPublish,
    activeSequence' = activeSequence,
    baseState' = baseState,
    stagedState' = stagedState,
    publishedState' = stagedState,
    requests' = requests,
    index' = index,
    results' = results,
    nextCommitOrdinal' = nextCommitOrdinal,
    phase' = Done,
  }

  action terminalStutter = all {
    phase == Done,
    activeSequence' = activeSequence,
    baseState' = baseState,
    stagedState' = stagedState,
    publishedState' = publishedState,
    requests' = requests,
    index' = index,
    results' = results,
    nextCommitOrdinal' = nextCommitOrdinal,
    phase' = phase,
  }

  action step = any {
    processInvalid,
    processConflict,
    processConditionOnly,
    processMutation,
    finishProcessing,
    PublishGroup,
    terminalStutter,
  }

  pure def referenceStateRevision(state: Key -> LogicalState, key: Key): int =
    match (state.get(key)) {
      | Present(statePayload) => statePayload._2
      | Missing(revision) => revision
    }

  pure def referenceConditionSatisfied(condition: Condition, state: Key -> LogicalState): bool =
    match (condition) {
      | RevisionEquals(conditionPayload) => referenceStateRevision(state, conditionPayload._1) == conditionPayload._2
      | Exists(key) => match (state.get(key)) {
          | Present(_) => true
          | Missing(_) => false
        }
      | NotExists(key) => match (state.get(key)) {
          | Present(_) => false
          | Missing(_) => true
        }
    }

  pure def referenceConditionsSatisfied(conditions: Set[Condition], state: Key -> LogicalState): bool =
    conditions.forall(condition => referenceConditionSatisfied(condition, state))

  pure def referenceApplyMutation(state: Key -> LogicalState, mutation: Mutation, ordinal: int): Key -> LogicalState =
    match (mutation) {
      | Put(mutationPayload) => state.set(mutationPayload._1, Present((mutationPayload._2, ordinal)))
      | Delete(key) => state.set(key, Missing(ordinal))
    }

  pure def referenceApplyMutations(state: Key -> LogicalState, mutations: Set[Mutation], ordinal: int): Key -> LogicalState =
    mutations.fold(state, (currentState, mutation) => referenceApplyMutation(currentState, mutation, ordinal))

  pure def referenceTransaction(state: Key -> LogicalState, request: Transaction, ordinal: int): ReferenceStep =
    if (request.conditions.size() + request.mutations.size() == 0) {
      { nextState: state, result: RequestInvalid, nextCommitOrdinal: ordinal }
    } else if (not (referenceConditionsSatisfied(request.conditions, state))) {
      { nextState: state, result: RequestConflict, nextCommitOrdinal: ordinal }
    } else if (request.mutations.size() == 0) {
      { nextState: state, result: RequestConditionsSatisfied, nextCommitOrdinal: ordinal }
    } else {
      {
        nextState: referenceApplyMutations(state, request.mutations, ordinal),
        result: RequestCommitted(ordinal),
        nextCommitOrdinal: ordinal + 1,
      }
    }

  pure def serialRun(initialState: Key -> LogicalState, transactionRequests: List[Transaction]): SerialEvaluation =
    foldl(transactionRequests, { finalState: initialState, results: List(), nextCommitOrdinal: 1 },
      (evaluation, request) => {
        val current = referenceTransaction(evaluation.finalState, request, evaluation.nextCommitOrdinal)
        {
          finalState: current.nextState,
          results: evaluation.results.append(current.result),
          nextCommitOrdinal: current.nextCommitOrdinal,
        }
      })

  val TypeOK = all {
    activeSequence == SequenceA or activeSequence == SequenceB or activeSequence == SequenceC or activeSequence == SequenceD,
    requests == requestsFor(activeSequence),
    index >= 0,
    index <= requests.length(),
    nextCommitOrdinal >= 1,
    nextCommitOrdinal <= requests.length() + 1,
    results.length() == index,
    index <= requests.length(),
    Keys.forall(key => referenceStateRevision(baseState, key) >= 0),
    Keys.forall(key => referenceStateRevision(stagedState, key) >= 0),
    Keys.forall(key => referenceStateRevision(publishedState, key) >= 0),
  }

  val ProcessedCountMatchesIndex = results.length() == index

  val BaseStateRemainsInitial = baseState == InitialState

  val PublishedStateIsPrivateUntilPublish = phase == Done or publishedState == baseState

  val ProcessedPrefixMatchesSerial = {
    val reference = serialRun(baseState, requests.slice(0, index))
    all {
      stagedState == reference.finalState,
      results == reference.results,
      nextCommitOrdinal == reference.nextCommitOrdinal,
    }
  }

  val PublishedResultMatchesSerial =
    phase != Done or {
      val reference = serialRun(baseState, requests)
      all {
        publishedState == reference.finalState,
        results == reference.results,
        nextCommitOrdinal == reference.nextCommitOrdinal,
      }
    }

  val state_invariants = all {
    TypeOK,
    ProcessedCountMatchesIndex,
    BaseStateRemainsInitial,
    PublishedStateIsPrivateUntilPublish,
    ProcessedPrefixMatchesSerial,
    PublishedResultMatchesSerial,
  }

  pure val SequenceATest = {
    val result = serialRun(InitialState, requestsFor(SequenceA))
    all {
      result.results == List(RequestCommitted(1), RequestConflict, RequestCommitted(2)),
      result.finalState.get(KeyA) == Present((ValueA, 1)),
      result.finalState.get(KeyB) == Present((ValueB, 2)),
      result.nextCommitOrdinal == 3,
    }
  }

  pure val SequenceBTest = {
    val result = serialRun(InitialState, requestsFor(SequenceB))
    all {
      result.results == List(RequestCommitted(1), RequestConditionsSatisfied, RequestCommitted(2)),
      result.finalState.get(KeyA) == Present((ValueA, 1)),
      result.finalState.get(KeyB) == Present((ValueB, 2)),
      result.nextCommitOrdinal == 3,
    }
  }

  pure val SequenceCTest = {
    val result = serialRun(InitialState, requestsFor(SequenceC))
    all {
      result.results == List(RequestCommitted(1), RequestConflict),
      result.finalState.get(KeyA) == Present((ValueA, 1)),
      result.finalState.get(KeyB) == Missing(0),
      result.nextCommitOrdinal == 2,
    }
  }

  pure val SequenceDTest = {
    val result = serialRun(InitialState, requestsFor(SequenceD))
    all {
      result.results == List(RequestInvalid, RequestCommitted(1)),
      result.finalState.get(KeyA) == Missing(0),
      result.finalState.get(KeyB) == Present((ValueA, 1)),
      result.nextCommitOrdinal == 2,
    }
  }
}
