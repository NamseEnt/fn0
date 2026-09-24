module end_to_end_pipeline {
  type Request = PutA | ConditionOnlyA | ConflictingPutB | PutB | ReadBarrierRequest | CheckpointBarrierRequest
  type Response = ACommitted | ConditionSatisfied | ConflictResult | BCommitted | ReadBoth | CheckpointSucceeded
  type LogicalState = { aRevision: int, bRevision: int }
  type Mode = NeedGroup | ProcessingGroup | StagingMutationSegment | SegmentDurableAwaitingResponses | Crashed | Recovered | Done

  pure val MaxPhysicalGroupRequests = 4
  pure val FixedRequests: List[Request] = List(PutA, ConditionOnlyA, ConflictingPutB, PutB, ReadBarrierRequest, CheckpointBarrierRequest)
  pure val ExpectedResponses: List[Response] = List(ACommitted, ConditionSatisfied, ConflictResult, BCommitted, ReadBoth, CheckpointSucceeded)
  pure val Initial: LogicalState = { aRevision: 0, bRevision: 0 }
  pure val AfterA: LogicalState = { aRevision: 1, bRevision: 0 }
  pure val AfterAB: LogicalState = { aRevision: 1, bRevision: 2 }

  pure def referenceStateAfterPrefix(prefixLength: int): LogicalState =
    if (prefixLength >= 4) AfterAB
    else if (prefixLength >= 1) AfterA
    else Initial

  pure def referenceNextCommitOrdinal(state: LogicalState): int =
    if (state == Initial) 1
    else if (state == AfterA) 2
    else 3

  pure def referenceRecoveredState(durableState: LogicalState, checkpointState: LogicalState, historyCleared: bool): LogicalState =
    if (historyCleared) checkpointState else durableState

  var requestIndex: int
  var groupStart: int
  var groupEnd: int
  var hasPhysicalGroup: bool
  var committedState: LogicalState
  var readViewState: LogicalState
  var durableCommittedState: LogicalState
  var durableMutationPrefix: int
  var nextCommitOrdinal: int
  var responses: List[Response]
  var walSyncCount: int
  var checkpointState: LogicalState
  var checkpointOrdinal: int
  var walHistoryCleared: bool
  var historyStartOrdinal: int
  var segmentStart: int
  var segmentEnd: int
  var segmentCursor: int
  var stagedState: LogicalState
  var segmentNextCommitOrdinal: int
  var segmentResponses: List[Response]
  var crashDurableState: LogicalState
  var crashCheckpointState: LogicalState
  var crashWalHistoryCleared: bool
  var crashNextCommitOrdinal: int
  var crashFromVolatileStaging: bool
  var recoveredState: LogicalState
  var recoveredNextCommitOrdinal: int
  var readBarrierObserved: LogicalState
  var sawMutationBarrierSharedPhysicalGroup: bool
  var mode: Mode

  action init = all {
    requestIndex' = 0,
    groupStart' = 0,
    groupEnd' = 0,
    hasPhysicalGroup' = false,
    committedState' = Initial,
    readViewState' = Initial,
    durableCommittedState' = Initial,
    durableMutationPrefix' = 0,
    nextCommitOrdinal' = 1,
    responses' = List(),
    walSyncCount' = 0,
    checkpointState' = Initial,
    checkpointOrdinal' = 0,
    walHistoryCleared' = false,
    historyStartOrdinal' = 0,
    segmentStart' = 0,
    segmentEnd' = 0,
    segmentCursor' = 0,
    stagedState' = Initial,
    segmentNextCommitOrdinal' = 1,
    segmentResponses' = List(),
    crashDurableState' = Initial,
    crashCheckpointState' = Initial,
    crashWalHistoryCleared' = false,
    crashNextCommitOrdinal' = 1,
    crashFromVolatileStaging' = false,
    recoveredState' = Initial,
    recoveredNextCommitOrdinal' = 1,
    readBarrierObserved' = Initial,
    sawMutationBarrierSharedPhysicalGroup' = false,
    mode' = NeedGroup,
  }

  action CollectPhysicalGroup = {
    nondet selectedGroupEnd = (requestIndex + 1).to(requestIndex + MaxPhysicalGroupRequests).oneOf()
    all {
      mode == NeedGroup,
      not (hasPhysicalGroup),
      requestIndex < FixedRequests.length(),
      selectedGroupEnd <= FixedRequests.length(),
      groupEnd' = selectedGroupEnd,
      groupStart' = requestIndex,
      hasPhysicalGroup' = true,
      mode' = ProcessingGroup,
      requestIndex' = requestIndex,
      committedState' = committedState,
      readViewState' = readViewState,
      durableCommittedState' = durableCommittedState,
      durableMutationPrefix' = durableMutationPrefix,
      nextCommitOrdinal' = nextCommitOrdinal,
      responses' = responses,
      walSyncCount' = walSyncCount,
      checkpointState' = checkpointState,
      checkpointOrdinal' = checkpointOrdinal,
      walHistoryCleared' = walHistoryCleared,
      historyStartOrdinal' = historyStartOrdinal,
      segmentStart' = segmentStart,
      segmentEnd' = segmentEnd,
      segmentCursor' = segmentCursor,
      stagedState' = stagedState,
      segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
      segmentResponses' = segmentResponses,
      crashDurableState' = crashDurableState,
      crashCheckpointState' = crashCheckpointState,
      crashWalHistoryCleared' = crashWalHistoryCleared,
      crashNextCommitOrdinal' = crashNextCommitOrdinal,
      crashFromVolatileStaging' = crashFromVolatileStaging,
      recoveredState' = recoveredState,
      recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
      readBarrierObserved' = readBarrierObserved,
      sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    }
  }

  action StartMutationSegment = all {
    mode == ProcessingGroup,
    hasPhysicalGroup,
    requestIndex < groupEnd,
    requestIndex < 4,
    segmentStart' = requestIndex,
    segmentEnd' = if (groupEnd < 4) groupEnd else 4,
    segmentCursor' = requestIndex,
    stagedState' = committedState,
    segmentNextCommitOrdinal' = nextCommitOrdinal,
    segmentResponses' = List(),
    mode' = StagingMutationSegment,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
  }

  action ProcessPutA = all {
    mode == StagingMutationSegment,
    segmentCursor == 0,
    segmentCursor < segmentEnd,
    stagedState' = { aRevision: segmentNextCommitOrdinal, bRevision: stagedState.bRevision },
    segmentResponses' = segmentResponses.append(ACommitted),
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal + 1,
    segmentCursor' = segmentCursor + 1,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    mode' = mode,
  }

  action ProcessConditionOnlyA = all {
    mode == StagingMutationSegment,
    segmentCursor == 1,
    segmentCursor < segmentEnd,
    stagedState.aRevision == 1,
    segmentResponses' = segmentResponses.append(ConditionSatisfied),
    segmentCursor' = segmentCursor + 1,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    stagedState' = stagedState,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    mode' = mode,
  }

  action ProcessConflictingPutB = all {
    mode == StagingMutationSegment,
    segmentCursor == 2,
    segmentCursor < segmentEnd,
    stagedState.aRevision != 0,
    segmentResponses' = segmentResponses.append(ConflictResult),
    segmentCursor' = segmentCursor + 1,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    stagedState' = stagedState,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    mode' = mode,
  }

  action ProcessPutB = all {
    mode == StagingMutationSegment,
    segmentCursor == 3,
    segmentCursor < segmentEnd,
    stagedState' = { aRevision: stagedState.aRevision, bRevision: segmentNextCommitOrdinal },
    segmentResponses' = segmentResponses.append(BCommitted),
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal + 1,
    segmentCursor' = segmentCursor + 1,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    mode' = mode,
  }

  action DurablyPublishMutationSegment = {
    val hasSuccessfulMutation = segmentNextCommitOrdinal > nextCommitOrdinal
    all {
      mode == StagingMutationSegment,
      segmentCursor == segmentEnd,
      durableCommittedState' = stagedState,
      committedState' = stagedState,
      readViewState' = stagedState,
      durableMutationPrefix' = segmentEnd,
      nextCommitOrdinal' = segmentNextCommitOrdinal,
      walSyncCount' = if (hasSuccessfulMutation) walSyncCount + 1 else walSyncCount,
      mode' = SegmentDurableAwaitingResponses,
      requestIndex' = requestIndex,
      groupEnd' = groupEnd,
    groupStart' = groupStart,
      hasPhysicalGroup' = hasPhysicalGroup,
      responses' = responses,
      checkpointState' = checkpointState,
      checkpointOrdinal' = checkpointOrdinal,
      walHistoryCleared' = walHistoryCleared,
      historyStartOrdinal' = historyStartOrdinal,
      segmentStart' = segmentStart,
      segmentEnd' = segmentEnd,
      segmentCursor' = segmentCursor,
      stagedState' = stagedState,
      segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
      segmentResponses' = segmentResponses,
      crashDurableState' = crashDurableState,
      crashCheckpointState' = crashCheckpointState,
      crashWalHistoryCleared' = crashWalHistoryCleared,
      crashNextCommitOrdinal' = crashNextCommitOrdinal,
      crashFromVolatileStaging' = crashFromVolatileStaging,
      recoveredState' = recoveredState,
      recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
      readBarrierObserved' = readBarrierObserved,
      sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    }
  }

  action EmitMutationSegmentResponses = all {
    mode == SegmentDurableAwaitingResponses,
    responses' = responses.concat(segmentResponses),
    requestIndex' = segmentEnd,
    mode' = ProcessingGroup,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    segmentCursor' = segmentCursor,
    stagedState' = stagedState,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    segmentResponses' = segmentResponses,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
  }

  action ReadBarrier = all {
    mode == ProcessingGroup,
    hasPhysicalGroup,
    requestIndex == 4,
    requestIndex < groupEnd,
    readViewState == AfterAB,
    readBarrierObserved' = readViewState,
    responses' = responses.append(ReadBoth),
    requestIndex' = requestIndex + 1,
    sawMutationBarrierSharedPhysicalGroup' = if (groupStart <= 3 and groupEnd > 4) true else sawMutationBarrierSharedPhysicalGroup,
    mode' = mode,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    segmentCursor' = segmentCursor,
    stagedState' = stagedState,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    segmentResponses' = segmentResponses,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
  }

  action CheckpointBarrier = all {
    mode == ProcessingGroup,
    hasPhysicalGroup,
    requestIndex == 5,
    requestIndex < groupEnd,
    committedState == AfterAB,
    readViewState == AfterAB,
    durableCommittedState == AfterAB,
    nextCommitOrdinal == 3,
    checkpointState' = AfterAB,
    checkpointOrdinal' = 2,
    walHistoryCleared' = true,
    historyStartOrdinal' = 2,
    responses' = responses.append(CheckpointSucceeded),
    requestIndex' = 6,
    mode' = mode,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    walSyncCount' = walSyncCount,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    segmentCursor' = segmentCursor,
    stagedState' = stagedState,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    segmentResponses' = segmentResponses,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
  }

  action FinishPhysicalGroup = all {
    mode == ProcessingGroup,
    hasPhysicalGroup,
    requestIndex == groupEnd,
    hasPhysicalGroup' = false,
    mode' = if (requestIndex == FixedRequests.length()) Done else NeedGroup,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    requestIndex' = requestIndex,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    segmentCursor' = segmentCursor,
    stagedState' = stagedState,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    segmentResponses' = segmentResponses,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
  }

  action Crash = all {
    mode == NeedGroup or mode == ProcessingGroup or mode == StagingMutationSegment or mode == SegmentDurableAwaitingResponses or mode == Done,
    crashDurableState' = durableCommittedState,
    crashCheckpointState' = checkpointState,
    crashWalHistoryCleared' = walHistoryCleared,
    crashNextCommitOrdinal' = nextCommitOrdinal,
    crashFromVolatileStaging' = mode == StagingMutationSegment,
    mode' = Crashed,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    segmentCursor' = segmentCursor,
    stagedState' = stagedState,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    segmentResponses' = segmentResponses,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
  }

  action Recover = {
    val sourceState = if (crashWalHistoryCleared) crashCheckpointState else crashDurableState
    all {
      mode == Crashed,
      recoveredState' = if (crashWalHistoryCleared) crashCheckpointState else crashDurableState,
      recoveredNextCommitOrdinal' = referenceNextCommitOrdinal(sourceState),
      mode' = Recovered,
      requestIndex' = requestIndex,
      groupEnd' = groupEnd,
    groupStart' = groupStart,
      hasPhysicalGroup' = hasPhysicalGroup,
      committedState' = committedState,
      readViewState' = readViewState,
      durableCommittedState' = durableCommittedState,
      durableMutationPrefix' = durableMutationPrefix,
      nextCommitOrdinal' = nextCommitOrdinal,
      responses' = responses,
      walSyncCount' = walSyncCount,
      checkpointState' = checkpointState,
      checkpointOrdinal' = checkpointOrdinal,
      walHistoryCleared' = walHistoryCleared,
      historyStartOrdinal' = historyStartOrdinal,
      segmentStart' = segmentStart,
      segmentEnd' = segmentEnd,
      segmentCursor' = segmentCursor,
      stagedState' = stagedState,
      segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
      segmentResponses' = segmentResponses,
      crashDurableState' = crashDurableState,
      crashCheckpointState' = crashCheckpointState,
      crashWalHistoryCleared' = crashWalHistoryCleared,
      crashNextCommitOrdinal' = crashNextCommitOrdinal,
      crashFromVolatileStaging' = crashFromVolatileStaging,
      readBarrierObserved' = readBarrierObserved,
      sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
    }
  }

  action terminalStutter = all {
    mode == Done or mode == Recovered,
    mode' = mode,
    requestIndex' = requestIndex,
    groupEnd' = groupEnd,
    groupStart' = groupStart,
    hasPhysicalGroup' = hasPhysicalGroup,
    committedState' = committedState,
    readViewState' = readViewState,
    durableCommittedState' = durableCommittedState,
    durableMutationPrefix' = durableMutationPrefix,
    nextCommitOrdinal' = nextCommitOrdinal,
    responses' = responses,
    walSyncCount' = walSyncCount,
    checkpointState' = checkpointState,
    checkpointOrdinal' = checkpointOrdinal,
    walHistoryCleared' = walHistoryCleared,
    historyStartOrdinal' = historyStartOrdinal,
    segmentStart' = segmentStart,
    segmentEnd' = segmentEnd,
    segmentCursor' = segmentCursor,
    stagedState' = stagedState,
    segmentNextCommitOrdinal' = segmentNextCommitOrdinal,
    segmentResponses' = segmentResponses,
    crashDurableState' = crashDurableState,
    crashCheckpointState' = crashCheckpointState,
    crashWalHistoryCleared' = crashWalHistoryCleared,
    crashNextCommitOrdinal' = crashNextCommitOrdinal,
    crashFromVolatileStaging' = crashFromVolatileStaging,
    recoveredState' = recoveredState,
    recoveredNextCommitOrdinal' = recoveredNextCommitOrdinal,
    readBarrierObserved' = readBarrierObserved,
    sawMutationBarrierSharedPhysicalGroup' = sawMutationBarrierSharedPhysicalGroup,
  }

  action step = any {
    CollectPhysicalGroup,
    StartMutationSegment,
    ProcessPutA,
    ProcessConditionOnlyA,
    ProcessConflictingPutB,
    ProcessPutB,
    DurablyPublishMutationSegment,
    EmitMutationSegmentResponses,
    ReadBarrier,
    CheckpointBarrier,
    FinishPhysicalGroup,
    Crash,
    Recover,
    terminalStutter,
  }

  action stepWithoutCrash = any {
    CollectPhysicalGroup,
    StartMutationSegment,
    ProcessPutA,
    ProcessConditionOnlyA,
    ProcessConflictingPutB,
    ProcessPutB,
    DurablyPublishMutationSegment,
    EmitMutationSegmentResponses,
    ReadBarrier,
    CheckpointBarrier,
    FinishPhysicalGroup,
    terminalStutter,
  }

  action stepCrashAfterCheckpoint = if (mode == Crashed) {
    Recover
  } else if (responses.length() == 6 and mode != Recovered) {
    Crash
  } else {
    stepWithoutCrash
  }

  val TypeOK = all {
    requestIndex >= 0 and requestIndex <= 6,
    groupStart >= 0 and groupStart <= 6,
    groupEnd >= 0 and groupEnd <= 6,
    not (hasPhysicalGroup) or (groupEnd >= requestIndex and groupEnd <= requestIndex + MaxPhysicalGroupRequests),
    durableMutationPrefix >= 0 and durableMutationPrefix <= 4,
    nextCommitOrdinal >= 1 and nextCommitOrdinal <= 3,
    crashNextCommitOrdinal >= 1 and crashNextCommitOrdinal <= 3,
    recoveredNextCommitOrdinal >= 1 and recoveredNextCommitOrdinal <= 3,
    walSyncCount >= 0 and walSyncCount <= 2,
    checkpointOrdinal >= 0 and checkpointOrdinal <= 2,
    historyStartOrdinal >= 0 and historyStartOrdinal <= 2,
    segmentStart >= 0 and segmentStart <= 4,
    segmentEnd >= segmentStart and segmentEnd <= 4,
    segmentCursor >= segmentStart and segmentCursor <= segmentEnd,
    segmentNextCommitOrdinal >= 1 and segmentNextCommitOrdinal <= 3,
    responses.length() <= 6,
    mode == NeedGroup or mode == ProcessingGroup or mode == StagingMutationSegment or mode == SegmentDurableAwaitingResponses or mode == Crashed or mode == Recovered or mode == Done,
    committedState == Initial or committedState == AfterA or committedState == AfterAB,
    readViewState == Initial or readViewState == AfterA or readViewState == AfterAB,
    durableCommittedState == Initial or durableCommittedState == AfterA or durableCommittedState == AfterAB,
    checkpointState == Initial or checkpointState == AfterA or checkpointState == AfterAB,
    stagedState == Initial or stagedState == AfterA or stagedState == AfterAB,
    crashDurableState == Initial or crashDurableState == AfterA or crashDurableState == AfterAB,
    crashCheckpointState == Initial or crashCheckpointState == AfterA or crashCheckpointState == AfterAB,
    recoveredState == Initial or recoveredState == AfterA or recoveredState == AfterAB,
  }

  val ResponsesAreSerialPrefix = responses == ExpectedResponses.slice(0, responses.length())
  val CommittedStateIsDurablePublishedState = committedState == durableCommittedState and readViewState == committedState
  val DurableStateMatchesSerialReference = durableCommittedState == referenceStateAfterPrefix(durableMutationPrefix)
  val ConditionOnlyDoesNotConsumeCommit = nextCommitOrdinal == referenceNextCommitOrdinal(durableCommittedState)
  val MutationSuccessResponseRequiresDurability =
    (not (responses.length() >= 1) or durableCommittedState.aRevision == 1) and
    (not (responses.length() >= 4) or durableCommittedState == AfterAB)
  val ConflictAndConditionDoNotCreateCommit =
    nextCommitOrdinal == referenceNextCommitOrdinal(durableCommittedState)
  val ReadBarrierSeesSerialCommittedState =
    (not (responses.length() >= 5) or (readBarrierObserved == AfterAB and readViewState == AfterAB and durableCommittedState == AfterAB)) and
    (requestIndex <= 4 or readBarrierObserved == AfterAB)
  val ResponsesPreserveBarrierOrder =
    (not (responses.length() >= 5) or responses.slice(0, 4) == ExpectedResponses.slice(0, 4)) and
    (not (responses.length() >= 6) or responses.slice(0, 5) == ExpectedResponses.slice(0, 5))
  val PhysicalGroupingDoesNotChangeFinalSemantics = mode != Done or all {
    responses == ExpectedResponses,
    committedState == AfterAB,
    readViewState == AfterAB,
    durableCommittedState == AfterAB,
    nextCommitOrdinal == 3,
    checkpointState == AfterAB,
    checkpointOrdinal == 2,
    walHistoryCleared,
    historyStartOrdinal == 2,
    walSyncCount == 1 or walSyncCount == 2,
  }
  val CheckpointCoversPublishedState = not (walHistoryCleared) or all {
    checkpointState == durableCommittedState,
    checkpointState == AfterAB,
    checkpointOrdinal == 2,
    historyStartOrdinal == 2,
  }
  val RecoveryMatchesDurableAuthority = mode != Recovered or all {
    recoveredState == referenceRecoveredState(crashDurableState, crashCheckpointState, crashWalHistoryCleared),
    recoveredNextCommitOrdinal == referenceNextCommitOrdinal(recoveredState),
    recoveredNextCommitOrdinal == referenceNextCommitOrdinal(
      referenceRecoveredState(crashDurableState, crashCheckpointState, crashWalHistoryCleared)),
  }
  val SuccessfulResponsesSurviveRecovery = mode != Recovered or all {
    not (responses.length() >= 1) or recoveredState.aRevision == 1,
    not (responses.length() >= 4) or recoveredState == AfterAB,
    not (responses.length() >= 6) or (recoveredState == AfterAB and recoveredNextCommitOrdinal == 3),
  }
  val VolatileStagingIsNeverRecoveryAuthority = mode != Recovered or
    (not (crashFromVolatileStaging) or recoveredState == referenceRecoveredState(crashDurableState, crashCheckpointState, crashWalHistoryCleared))

  val state_invariants = all {
    TypeOK,
    ResponsesAreSerialPrefix,
    CommittedStateIsDurablePublishedState,
    DurableStateMatchesSerialReference,
    ConditionOnlyDoesNotConsumeCommit,
    MutationSuccessResponseRequiresDurability,
    ConflictAndConditionDoNotCreateCommit,
    ReadBarrierSeesSerialCommittedState,
    ResponsesPreserveBarrierOrder,
    PhysicalGroupingDoesNotChangeFinalSemantics,
    CheckpointCoversPublishedState,
    RecoveryMatchesDurableAuthority,
    SuccessfulResponsesSurviveRecovery,
    VolatileStagingIsNeverRecoveryAuthority,
  }

  pure val ReferencePrefixTest = all {
    referenceStateAfterPrefix(0) == Initial,
    referenceStateAfterPrefix(1) == AfterA,
    referenceStateAfterPrefix(2) == AfterA,
    referenceStateAfterPrefix(3) == AfterA,
    referenceStateAfterPrefix(4) == AfterAB,
    referenceStateAfterPrefix(5) == AfterAB,
    referenceStateAfterPrefix(6) == AfterAB,
    referenceNextCommitOrdinal(Initial) == 1,
    referenceNextCommitOrdinal(AfterA) == 2,
    referenceNextCommitOrdinal(AfterAB) == 3,
  }

  pure val RecoveryReferenceTest = all {
    referenceRecoveredState(AfterA, Initial, false) == AfterA,
    referenceRecoveredState(AfterAB, Initial, false) == AfterAB,
    referenceRecoveredState(AfterAB, AfterAB, true) == AfterAB,
  }

  val pathAReached = mode == Done and walSyncCount == 1 and responses == ExpectedResponses and durableCommittedState == AfterAB
  val pathBReached = mode == Done and walSyncCount == 2 and responses == ExpectedResponses and durableCommittedState == AfterAB
  val pathCReached = sawMutationBarrierSharedPhysicalGroup and responses.length() >= 5 and readBarrierObserved == AfterAB
  val pathDReached = mode == Recovered and crashDurableState == AfterA and recoveredState == AfterA and recoveredNextCommitOrdinal == 2
  val pathEReached = mode == Recovered and crashDurableState == AfterAB and responses.length() < 4 and recoveredState == AfterAB and recoveredNextCommitOrdinal == 3
  val pathFReached = mode == Recovered and crashFromVolatileStaging and crashDurableState == AfterA and stagedState == AfterAB and recoveredState == AfterA
  val pathGReached = responses.length() >= 5 and readBarrierObserved == AfterAB
  val pathHReached = mode == Recovered and responses.length() >= 6 and crashWalHistoryCleared and crashCheckpointState == AfterAB and recoveredState == AfterAB and recoveredNextCommitOrdinal == 3
}
