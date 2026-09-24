module publication_visibility {
  type ResponseState = Pending | Success | ErrorResponse
  type FailureKind = NoFailure | WalFailure | BeforePublishFailure | AfterPublishFailure
  type Phase = Prepared | WalCompleted | Publishing | StorePublicationComplete | WaitingForReadViewPublication | ReplyingSuccess | ReplyingError | Crashed | Recovered | Done

  pure val LogicalCounts: Set[int] = Set(0, 1, 2)
  pure val ResponseStates: Set[ResponseState] = Set(Pending, Success, ErrorResponse)
  pure val FailureKinds: Set[FailureKind] = Set(NoFailure, WalFailure, BeforePublishFailure, AfterPublishFailure)
  pure val Phases: Set[Phase] = Set(Prepared, WalCompleted, Publishing, StorePublicationComplete, WaitingForReadViewPublication, ReplyingSuccess, ReplyingError, Crashed, Recovered, Done)

  pure def referenceRecoveredCount(walDurableCount: int): int = walDurableCount

  pure def liveReadViewAdmissible(count: int): bool = count == 0 or count == 2

  var walDurableCount: int
  var walAppendSucceeded: bool
  var storePublishedCount: int
  var readViewCount: int
  var readViewBroken: bool
  var failureKind: FailureKind
  var responseA: ResponseState
  var responseB: ResponseState
  var crashed: bool
  var recovered: bool
  var phase: Phase
  var observedCounts: Set[int]
  var preRecoveryObservedCounts: Set[int]
  var postRecoveryObservedCounts: Set[int]
  var preCrashStorePublishedCount: int
  var preCrashReadViewCount: int
  var sawBrokenReadView: bool

  action init = all {
    walDurableCount' = 0,
    walAppendSucceeded' = false,
    storePublishedCount' = 0,
    readViewCount' = 0,
    readViewBroken' = false,
    failureKind' = NoFailure,
    responseA' = Pending,
    responseB' = Pending,
    crashed' = false,
    recovered' = false,
    phase' = Prepared,
    observedCounts' = Set(),
    preRecoveryObservedCounts' = Set(),
    postRecoveryObservedCounts' = Set(),
    preCrashStorePublishedCount' = 0,
    preCrashReadViewCount' = 0,
    sawBrokenReadView' = false,
  }

  action WalAppendSuccess = all {
    phase == Prepared,
    walDurableCount' = 2,
    walAppendSucceeded' = true,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = WalCompleted,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action WalAppendFailure = {
    nondet persistedCount = LogicalCounts.oneOf()
    all {
      phase == Prepared,
      walDurableCount' = persistedCount,
      walAppendSucceeded' = false,
      storePublishedCount' = 0,
      readViewCount' = 0,
      readViewBroken' = true,
      failureKind' = WalFailure,
      responseA' = responseA,
      responseB' = responseB,
      crashed' = crashed,
      recovered' = recovered,
      phase' = ReplyingError,
      observedCounts' = observedCounts,
      preRecoveryObservedCounts' = preRecoveryObservedCounts,
      postRecoveryObservedCounts' = postRecoveryObservedCounts,
      preCrashStorePublishedCount' = preCrashStorePublishedCount,
      preCrashReadViewCount' = preCrashReadViewCount,
      sawBrokenReadView' = true,
    }
  }

  action PublishA = all {
    phase == WalCompleted or phase == Publishing,
    walAppendSucceeded,
    walDurableCount == 2,
    failureKind == NoFailure,
    storePublishedCount == 0,
    storePublishedCount' = 1,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = Publishing,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action PublishB = all {
    phase == Publishing,
    walDurableCount == 2,
    storePublishedCount == 1,
    storePublishedCount' = 2,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = StorePublicationComplete,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action NormalStorePublicationComplete = all {
    phase == StorePublicationComplete,
    storePublishedCount == 2,
    failureKind == NoFailure,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = WaitingForReadViewPublication,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action InjectBeforePublishFailure = all {
    phase == WalCompleted,
    walAppendSucceeded,
    walDurableCount == 2,
    storePublishedCount == 0,
    failureKind == NoFailure,
    failureKind' = BeforePublishFailure,
    readViewBroken' = true,
    storePublishedCount' = 0,
    readViewCount' = readViewCount,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = ReplyingError,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = true,
  }

  action InjectAfterPublishFailure = all {
    phase == StorePublicationComplete,
    walDurableCount == 2,
    storePublishedCount == 2,
    readViewCount == 0,
    failureKind == NoFailure,
    failureKind' = AfterPublishFailure,
    readViewBroken' = readViewBroken,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = WaitingForReadViewPublication,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action PublishReadView = all {
    phase == WaitingForReadViewPublication,
    storePublishedCount == 2,
    failureKind == NoFailure or failureKind == AfterPublishFailure,
    not (readViewBroken),
    readViewCount' = 2,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = if (failureKind == NoFailure) ReplyingSuccess else ReplyingError,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action SendSuccessA = all {
    phase == ReplyingSuccess,
    walDurableCount == 2,
    storePublishedCount == 2,
    readViewCount == 2,
    failureKind == NoFailure,
    responseA == Pending,
    responseA' = Success,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = ReplyingSuccess,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action SendSuccessB = all {
    phase == ReplyingSuccess,
    responseA == Success,
    responseB == Pending,
    walDurableCount == 2,
    storePublishedCount == 2,
    readViewCount == 2,
    failureKind == NoFailure,
    responseB' = Success,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    crashed' = crashed,
    recovered' = recovered,
    phase' = Done,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action SendErrorA = all {
    phase == ReplyingError,
    failureKind != NoFailure,
    responseA == Pending,
    responseA' = ErrorResponse,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = ReplyingError,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action SendErrorB = all {
    phase == ReplyingError,
    failureKind != NoFailure,
    responseA == ErrorResponse,
    responseB == Pending,
    responseB' = ErrorResponse,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    crashed' = crashed,
    recovered' = recovered,
    phase' = Done,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action OrdinaryRead = all {
    not (crashed),
    phase != Crashed,
    not (readViewBroken),
    (not (recovered) and not (preRecoveryObservedCounts.contains(readViewCount))) or
      (recovered and not (postRecoveryObservedCounts.contains(readViewCount))),
    observedCounts' = observedCounts.union(Set(readViewCount)),
    if (not (recovered)) {
      preRecoveryObservedCounts' = preRecoveryObservedCounts.union(Set(readViewCount))
    } else {
      preRecoveryObservedCounts' = preRecoveryObservedCounts
    },
    if (recovered) {
      postRecoveryObservedCounts' = postRecoveryObservedCounts.union(Set(readViewCount))
    } else {
      postRecoveryObservedCounts' = postRecoveryObservedCounts
    },
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = phase,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action Crash = all {
    not (crashed),
    not (recovered),
    phase != Done,
    crashed' = true,
    phase' = Crashed,
    preCrashStorePublishedCount' = storePublishedCount,
    preCrashReadViewCount' = readViewCount,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = 0,
    readViewCount' = 0,
    readViewBroken' = false,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    recovered' = recovered,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action Recover = all {
    crashed,
    phase == Crashed,
    storePublishedCount' = walDurableCount,
    readViewCount' = walDurableCount,
    readViewBroken' = false,
    recovered' = true,
    crashed' = false,
    phase' = Recovered,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }


  action terminalStutter = all {
    phase == Done or phase == Recovered,
    walDurableCount' = walDurableCount,
    walAppendSucceeded' = walAppendSucceeded,
    storePublishedCount' = storePublishedCount,
    readViewCount' = readViewCount,
    readViewBroken' = readViewBroken,
    failureKind' = failureKind,
    responseA' = responseA,
    responseB' = responseB,
    crashed' = crashed,
    recovered' = recovered,
    phase' = phase,
    observedCounts' = observedCounts,
    preRecoveryObservedCounts' = preRecoveryObservedCounts,
    postRecoveryObservedCounts' = postRecoveryObservedCounts,
    preCrashStorePublishedCount' = preCrashStorePublishedCount,
    preCrashReadViewCount' = preCrashReadViewCount,
    sawBrokenReadView' = sawBrokenReadView,
  }

  action step = any {
    WalAppendSuccess,
    WalAppendFailure,
    PublishA,
    PublishB,
    NormalStorePublicationComplete,
    InjectBeforePublishFailure,
    InjectAfterPublishFailure,
    PublishReadView,
    SendSuccessA,
    SendSuccessB,
    SendErrorA,
    SendErrorB,
    OrdinaryRead,
    Crash,
    Recover,
    terminalStutter,
  }

  val TypeOK = all {
    LogicalCounts.contains(walDurableCount),
    LogicalCounts.contains(storePublishedCount),
    LogicalCounts.contains(readViewCount),
    LogicalCounts.contains(preCrashStorePublishedCount),
    LogicalCounts.contains(preCrashReadViewCount),
    ResponseStates.contains(responseA),
    ResponseStates.contains(responseB),
    FailureKinds.contains(failureKind),
    Phases.contains(phase),
    observedCounts.forall(count => LogicalCounts.contains(count)),
    preRecoveryObservedCounts.forall(count => LogicalCounts.contains(count)),
    postRecoveryObservedCounts.forall(count => LogicalCounts.contains(count)),
    crashed implies phase == Crashed,
    recovered implies phase == Recovered or phase == Done,
  }

  val WalSuccessMeansWholeGroupDurable = not (walAppendSucceeded) or walDurableCount == 2

  val LiveStorePublicationRequiresDurableWal = recovered or storePublishedCount == 0 or
    (walAppendSucceeded and walDurableCount == 2)

  val LiveReadViewNeverShowsPartialGroup = recovered or liveReadViewAdmissible(readViewCount)

  val ReadViewPublicationRequiresFullStorePublication = recovered or readViewCount != 2 or
    (storePublishedCount == 2 and walDurableCount == 2)

  val ObservedNewStateRequiresDurability =
    (not (preRecoveryObservedCounts.contains(1))) and
    (not (preRecoveryObservedCounts.contains(2)) or walDurableCount == 2) and
    (not (postRecoveryObservedCounts.contains(2)) or walDurableCount == 2) and
    (not (observedCounts.contains(2)) or walDurableCount == 2)

  val SuccessfulResponseRequiresDurabilityAndVisibility =
    ((responseA != Success and responseB != Success) or
      (walDurableCount == 2 and failureKind == NoFailure and
        ((not (crashed) and storePublishedCount == 2 and readViewCount == 2) or
          (crashed and preCrashStorePublishedCount == 2 and preCrashReadViewCount == 2))))

  val ResponsesPreserveFIFO =
    (responseB == Pending or responseA != Pending) and
    (responseB != Success or responseA == Success) and
    (responseB != ErrorResponse or responseA == ErrorResponse)

  val WalOrBeforePublishFailureMarksReadViewBroken = crashed or recovered or
    ((failureKind != WalFailure and failureKind != BeforePublishFailure) or
      (readViewBroken and readViewCount == 0 and storePublishedCount == 0))

  val AfterPublishFailureNeverReportsSuccess = failureKind != AfterPublishFailure or
    (responseA != Success and responseB != Success)

  val RecoveryMatchesWalDurability = not (recovered) or
    (storePublishedCount == walDurableCount and readViewCount == walDurableCount and not (readViewBroken))

  val CrashDoesNotRecoverFromVolatileStore = not (recovered) or
    storePublishedCount == referenceRecoveredCount(walDurableCount)

  val state_invariants = all {
    TypeOK,
    WalSuccessMeansWholeGroupDurable,
    LiveStorePublicationRequiresDurableWal,
    LiveReadViewNeverShowsPartialGroup,
    ReadViewPublicationRequiresFullStorePublication,
    ObservedNewStateRequiresDurability,
    SuccessfulResponseRequiresDurabilityAndVisibility,
    ResponsesPreserveFIFO,
    WalOrBeforePublishFailureMarksReadViewBroken,
    AfterPublishFailureNeverReportsSuccess,
    RecoveryMatchesWalDurability,
    CrashDoesNotRecoverFromVolatileStore,
  }

  pure val ReferenceRecoveryTest = all {
    referenceRecoveredCount(0) == 0,
    referenceRecoveredCount(1) == 1,
    referenceRecoveredCount(2) == 2,
  }

  pure val ReadViewAdmissibilityTest = all {
    liveReadViewAdmissible(0),
    not (liveReadViewAdmissible(1)),
    liveReadViewAdmissible(2),
  }

  val publicationPathAReached = phase == Done and responseA == Success and responseB == Success and readViewCount == 2 and failureKind == NoFailure
  val publicationPathBReached = recovered and walDurableCount == 2 and preCrashStorePublishedCount == 1 and preRecoveryObservedCounts.contains(0)
  val publicationPathCReached = recovered and failureKind == BeforePublishFailure and sawBrokenReadView and responseA == ErrorResponse and walDurableCount == 2
  val publicationPathDReached = recovered and failureKind == WalFailure and walDurableCount == 1 and responseA == ErrorResponse
  val publicationPathEReached = phase == Done and failureKind == AfterPublishFailure and readViewCount == 2 and responseA == ErrorResponse and responseB == ErrorResponse
  val publicationPathFReached = recovered and walDurableCount == 2 and responseA == Success and responseB == Pending
}
