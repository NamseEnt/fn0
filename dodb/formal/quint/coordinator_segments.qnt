module coordinator_segments {
  type Operation = PutA | PutB | ConditionOnly | TransactGet | Observe | Checkpoint | CheckInvariants
  type Phase = Running | Done
  type LogicalState = { aVersion: int, bVersion: int }
  type BarrierObservation = { kind: Operation, aVersion: int, bVersion: int }
  type Segment = | MutationSegment((int, int)) | Barrier(int)
  type ReferenceEvaluation = { finalState: LogicalState, observations: List[BarrierObservation] }
  pure val FixedRequests: List[Operation] = List(
    PutA,
    ConditionOnly,
    PutB,
    TransactGet,
    PutA,
    Observe,
    Checkpoint,
    PutB,
    CheckInvariants,
  )

  pure val InitialState: LogicalState = { aVersion: 0, bVersion: 0 }

  pure def isMutationLike(operation: Operation): bool =
    operation == PutA or operation == PutB or operation == ConditionOnly

  pure def isBarrier(operation: Operation): bool =
    operation == TransactGet or operation == Observe or operation == Checkpoint or operation == CheckInvariants

  pure def referenceIsMutationLike(operation: Operation): bool =
    match (operation) {
      | PutA => true
      | PutB => true
      | ConditionOnly => true
      | TransactGet => false
      | Observe => false
      | Checkpoint => false
      | CheckInvariants => false
    }

  pure def implementationMutationEffect(state: LogicalState, operation: Operation): LogicalState =
    match (operation) {
      | PutA => { aVersion: state.aVersion + 1, bVersion: state.bVersion }
      | PutB => { aVersion: state.aVersion, bVersion: state.bVersion + 1 }
      | ConditionOnly => state
      | _ => state
    }

  pure def applyMutationSegment(state: LogicalState, segment: List[Operation]): LogicalState =
    foldl(segment, state,
      (currentState, operation) => implementationMutationEffect(currentState, operation))

  pure def maximalMutationPrefixEnd(start: int): int = {
    val scan = List(0, 1, 2, 3, 4, 5, 6, 7, 8).foldl(
      { endExclusive: start, accepting: true },
      (current, requestIndex) => {
        if (current.accepting and requestIndex >= start) {
          if (isMutationLike(FixedRequests.nth(requestIndex))) {
            { endExclusive: requestIndex + 1, accepting: true }
          } else {
            { endExclusive: current.endExclusive, accepting: false }
          }
        } else {
          current
        }
      })
    scan.endExclusive
  }

  pure def referenceMutationEffect(state: LogicalState, operation: Operation): LogicalState =
    if (operation == PutA) {
      { aVersion: state.aVersion + 1, bVersion: state.bVersion }
    } else if (operation == PutB) {
      { aVersion: state.aVersion, bVersion: state.bVersion + 1 }
    } else {
      state
    }

  pure def referenceBarrierObservation(state: LogicalState, operation: Operation): BarrierObservation = {
    kind: operation,
    aVersion: state.aVersion,
    bVersion: state.bVersion,
  }

  pure def referenceRun(requests: List[Operation]): ReferenceEvaluation =
    foldl(requests, { finalState: InitialState, observations: List() },
      (evaluation, operation) => {
        if (referenceIsMutationLike(operation)) {
          {
            finalState: referenceMutationEffect(evaluation.finalState, operation),
            observations: evaluation.observations,
          }
        } else {
          {
            finalState: evaluation.finalState,
            observations: evaluation.observations.append(
              referenceBarrierObservation(evaluation.finalState, operation)),
          }
        }
      })

  pure def referenceSegments(requests: List[Operation]): List[Segment] = {
    val scan = List(0, 1, 2, 3, 4, 5, 6, 7, 8).slice(0, requests.length()).foldl(
      { segments: List(), openStart: -1 },
      (current, requestIndex) => {
        val operation = requests.nth(requestIndex)
        if (referenceIsMutationLike(operation)) {
          if (current.openStart == -1) {
            { segments: current.segments, openStart: requestIndex }
          } else {
            current
          }
        } else {
          val closedSegments = if (current.openStart == -1) {
            current.segments
          } else {
            current.segments.append(MutationSegment((current.openStart, requestIndex)))
          }
          {
            segments: closedSegments.append(Barrier(requestIndex)),
            openStart: -1,
          }
        }
      })
    if (scan.openStart == -1) {
      scan.segments
    } else {
      scan.segments.append(MutationSegment((scan.openStart, requests.length())))
    }
  }

  var requests: List[Operation]
  var cursor: int
  var logicalState: LogicalState
  var segmentLog: List[Segment]
  var barrierObservations: List[BarrierObservation]
  var phase: Phase

  action init = all {
    requests' = FixedRequests,
    cursor' = 0,
    logicalState' = InitialState,
    segmentLog' = List(),
    barrierObservations' = List(),
    phase' = Running,
  }

  action ProcessMutationSegment = {
    val endExclusive = maximalMutationPrefixEnd(cursor)
    all {
      phase == Running,
      cursor < requests.length(),
      isMutationLike(requests.nth(cursor)),
      endExclusive > cursor,
      logicalState' = applyMutationSegment(logicalState, requests.slice(cursor, endExclusive)),
      segmentLog' = segmentLog.append(MutationSegment((cursor, endExclusive))),
      barrierObservations' = barrierObservations,
      cursor' = endExclusive,
      requests' = requests,
      phase' = phase,
    }
  }

  action ProcessBarrier = {
    val operation = requests.nth(cursor)
    all {
      phase == Running,
      cursor < requests.length(),
      isBarrier(operation),
      logicalState' = logicalState,
      segmentLog' = segmentLog.append(Barrier(cursor)),
      barrierObservations' = barrierObservations.append({
        kind: operation,
        aVersion: logicalState.aVersion,
        bVersion: logicalState.bVersion,
      }),
      cursor' = cursor + 1,
      requests' = requests,
      phase' = phase,
    }
  }

  action Finish = all {
    phase == Running,
    cursor == requests.length(),
    requests' = requests,
    cursor' = cursor,
    logicalState' = logicalState,
    segmentLog' = segmentLog,
    barrierObservations' = barrierObservations,
    phase' = Done,
  }

  action terminalStutter = all {
    phase == Done,
    requests' = requests,
    cursor' = cursor,
    logicalState' = logicalState,
    segmentLog' = segmentLog,
    barrierObservations' = barrierObservations,
    phase' = phase,
  }

  action step = any {
    ProcessMutationSegment,
    ProcessBarrier,
    Finish,
    terminalStutter,
  }

  val TypeOK = all {
    requests == FixedRequests,
    cursor >= 0,
    cursor <= requests.length(),
    logicalState.aVersion >= 0,
    logicalState.bVersion >= 0,
    phase == Running or phase == Done,
    segmentLog == referenceSegments(requests.slice(0, cursor)),
    barrierObservations.length() <= requests.length(),
  }

  val CursorInBounds = cursor >= 0 and cursor <= requests.length()

  val ProcessedPrefixMatchesReference = {
    val reference = referenceRun(requests.slice(0, cursor))
    all {
      logicalState == reference.finalState,
      barrierObservations == reference.observations,
    }
  }

  val SegmentLogMatchesReference =
    segmentLog == referenceSegments(requests.slice(0, cursor))

  val ConditionOnlyDoesNotSplitMutationSegment =
    cursor < 3 or segmentLog.nth(0) == MutationSegment((0, 3))

  val OperationClassificationMatchesContract = all {
    isMutationLike(PutA),
    isMutationLike(PutB),
    isMutationLike(ConditionOnly),
    not (isBarrier(PutA)),
    not (isBarrier(PutB)),
    not (isBarrier(ConditionOnly)),
    isBarrier(TransactGet),
    isBarrier(Observe),
    isBarrier(Checkpoint),
    isBarrier(CheckInvariants),
    not (isMutationLike(TransactGet)),
    not (isMutationLike(Observe)),
    not (isMutationLike(Checkpoint)),
    not (isMutationLike(CheckInvariants)),
  }

  val DoneMatchesReference =
    phase != Done or {
      val reference = referenceRun(requests)
      all {
        cursor == requests.length(),
        logicalState == reference.finalState,
        barrierObservations == reference.observations,
      }
    }

  val state_invariants = all {
    TypeOK,
    CursorInBounds,
    ProcessedPrefixMatchesReference,
    SegmentLogMatchesReference,
    ConditionOnlyDoesNotSplitMutationSegment,
    OperationClassificationMatchesContract,
    DoneMatchesReference,
  }

  pure val ExpectedSegmentsTest =
    referenceSegments(FixedRequests) == List(
      MutationSegment((0, 3)),
      Barrier(3),
      MutationSegment((4, 5)),
      Barrier(5),
      Barrier(6),
      MutationSegment((7, 8)),
      Barrier(8),
    )

  pure val ExpectedBarrierObservationsTest = {
    val result = referenceRun(FixedRequests)
    result.observations == List(
      { kind: TransactGet, aVersion: 1, bVersion: 1 },
      { kind: Observe, aVersion: 2, bVersion: 1 },
      { kind: Checkpoint, aVersion: 2, bVersion: 1 },
      { kind: CheckInvariants, aVersion: 2, bVersion: 2 },
    )
  }

  pure val ExpectedFinalStateTest =
    referenceRun(FixedRequests).finalState == { aVersion: 2, bVersion: 2 }
}
