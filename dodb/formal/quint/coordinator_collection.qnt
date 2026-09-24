module coordinator_collection {
  type Request = { id: int, size: int }
  type PendingState = NoPending | PendingRequest(Request)
  type Phase = Collecting | Done

  pure val R0: Request = { id: 0, size: 0 }
  pure val R1: Request = { id: 1, size: 3 }
  pure val R2: Request = { id: 2, size: 1 }
  pure val R3: Request = { id: 3, size: 1 }
  pure val R4: Request = { id: 4, size: 1 }
  pure val FixedRequests: List[Request] = List(R0, R1, R2, R3, R4)
  pure val MaxGroupRequests: int = 2
  pure val MaxGroupBytes: int = 2

  pure def canAddSubsequent(groupCount: int, currentBytes: int, requestSize: int): bool =
    groupCount < MaxGroupRequests
      and currentBytes + requestSize <= MaxGroupBytes

  pure def firstRequestMayStart(request: Request): bool = request.size >= 0

  pure def sumSizes(requests: List[Request]): int =
    requests.foldl(0, (total, request) => total + request.size)

  pure def flattenGroups(groups: List[List[Request]]): List[Request] =
    groups.foldl(List(), (requests, group) => requests.concat(group))

  pure def pendingAsList(pendingState: PendingState): List[Request] =
    match pendingState {
      | NoPending => List()
      | PendingRequest(request) => List(request)
    }

  pure def requestIsWellFormed(request: Request): bool =
    request.id >= 0
      and request.id < FixedRequests.length()
      and request.size >= 0

  pure def physicalGroupRespectsBounds(group: List[Request]): bool =
    group.length() >= 1
      and group.length() <= MaxGroupRequests
      and (group.length() == 1 or sumSizes(group) <= MaxGroupBytes)

  pure def everyRequestWellFormed(requests: List[Request]): bool =
    requests.foldl(true, (wellFormed, request) => wellFormed and requestIsWellFormed(request))

  pure def everyGroupRespectsBounds(groups: List[List[Request]]): bool =
    groups.foldl(true, (withinBounds, group) => withinBounds and physicalGroupRespectsBounds(group))

  pure def everyGroupRequestsWellFormed(groups: List[List[Request]]): bool =
    groups.foldl(true, (wellFormed, group) => wellFormed and everyRequestWellFormed(group))

  var nextIndex: int
  var pending: PendingState
  var currentGroup: List[Request]
  var currentBytes: int
  var sealedGroups: List[List[Request]]
  var phase: Phase

  action init = all {
    nextIndex' = 0,
    pending' = NoPending,
    currentGroup' = List(),
    currentBytes' = 0,
    sealedGroups' = List(),
    phase' = Collecting,
  }

  action StartGroup = {
    val firstRequest = match pending {
      | NoPending => FixedRequests.nth(nextIndex)
      | PendingRequest(request) => request
    }
    all {
      phase == Collecting,
      currentGroup.length() == 0,
      pending != NoPending or nextIndex < FixedRequests.length(),
      currentGroup' = List(firstRequest),
      currentBytes' = firstRequest.size,
      pending' = NoPending,
      nextIndex' = if (pending == NoPending) nextIndex + 1 else nextIndex,
      sealedGroups' = sealedGroups,
      phase' = phase,
    }
  }

  action CollectNext = {
    val request = FixedRequests.nth(nextIndex)
    all {
      phase == Collecting,
      currentGroup.length() > 0,
      pending == NoPending,
      nextIndex < FixedRequests.length(),
      canAddSubsequent(currentGroup.length(), currentBytes, request.size),
      currentGroup' = currentGroup.append(request),
      currentBytes' = currentBytes + request.size,
      nextIndex' = nextIndex + 1,
      pending' = pending,
      sealedGroups' = sealedGroups,
      phase' = phase,
    }
  }

  action RejectNextToPending = {
    val request = FixedRequests.nth(nextIndex)
    all {
      phase == Collecting,
      currentGroup.length() > 0,
      pending == NoPending,
      nextIndex < FixedRequests.length(),
      not (canAddSubsequent(currentGroup.length(), currentBytes, request.size)),
      currentGroup' = currentGroup,
      currentBytes' = currentBytes,
      nextIndex' = nextIndex + 1,
      pending' = PendingRequest(request),
      sealedGroups' = sealedGroups,
      phase' = phase,
    }
  }

  action SealGroup = all {
    phase == Collecting,
    currentGroup.length() > 0,
    sealedGroups' = sealedGroups.append(currentGroup),
    currentGroup' = List(),
    currentBytes' = 0,
    nextIndex' = nextIndex,
    pending' = pending,
    phase' = phase,
  }

  action Finish = all {
    phase == Collecting,
    nextIndex == FixedRequests.length(),
    pending == NoPending,
    currentGroup.length() == 0,
    nextIndex' = nextIndex,
    pending' = pending,
    currentGroup' = currentGroup,
    currentBytes' = currentBytes,
    sealedGroups' = sealedGroups,
    phase' = Done,
  }

  action terminalStutter = all {
    phase == Done,
    nextIndex' = nextIndex,
    pending' = pending,
    currentGroup' = currentGroup,
    currentBytes' = currentBytes,
    sealedGroups' = sealedGroups,
    phase' = phase,
  }

  action step = any {
    StartGroup,
    CollectNext,
    RejectNextToPending,
    SealGroup,
    Finish,
    terminalStutter,
  }

  val ConservationOrder: List[Request] =
    flattenGroups(sealedGroups)
      .concat(currentGroup)
      .concat(pendingAsList(pending))
      .concat(FixedRequests.slice(nextIndex, FixedRequests.length()))

  val TypeOK = all {
    nextIndex >= 0,
    nextIndex <= FixedRequests.length(),
    FixedRequests.length() == 5,
    FixedRequests == List(R0, R1, R2, R3, R4),
    everyRequestWellFormed(FixedRequests),
    currentBytes >= 0,
    everyRequestWellFormed(currentGroup),
    everyRequestWellFormed(pendingAsList(pending)),
    everyGroupRequestsWellFormed(sealedGroups),
    phase == Collecting or phase == Done,
  }

  val RequestConservationAndFIFO = ConservationOrder == FixedRequests

  val PhysicalGroupsRespectBounds =
    not (canAddSubsequent(MaxGroupRequests, 0, 0))
      and everyGroupRespectsBounds(sealedGroups)
      and (currentGroup.length() == 0 or physicalGroupRespectsBounds(currentGroup))

  val CurrentByteAccountingExact = currentBytes == sumSizes(currentGroup)

  val PendingWasRejectedByCurrentBounds =
    pending == NoPending
      or currentGroup.length() == 0
      or match pending {
        | NoPending => true
        | PendingRequest(request) =>
          not (canAddSubsequent(currentGroup.length(), currentBytes, request.size))
      }

  val DoneContainsEveryRequestExactlyOnce =
    phase != Done
      or all {
        flattenGroups(sealedGroups) == FixedRequests,
        currentGroup.length() == 0,
        pending == NoPending,
        nextIndex == FixedRequests.length(),
      }

  val state_invariants = all {
    TypeOK,
    RequestConservationAndFIFO,
    PhysicalGroupsRespectBounds,
    CurrentByteAccountingExact,
    PendingWasRejectedByCurrentBounds,
    DoneContainsEveryRequestExactlyOnce,
  }

  pure val ZeroByteFirstCannotBypassByteBoundTest =
    not (canAddSubsequent(1, 0, 3))

  pure val NormalRequestFitsTest =
    canAddSubsequent(1, 1, 1)

  pure val FullRequestCountRejectsTest =
    not (canAddSubsequent(2, 0, 0))

  pure val FullByteBoundRejectsTest =
    not (canAddSubsequent(1, 2, 1))

  pure val OversizedFirstRequestMayStartTest =
    firstRequestMayStart(R1)
      and R1.size == 3
      and R1.size > MaxGroupBytes
}
