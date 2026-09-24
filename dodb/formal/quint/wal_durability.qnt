module wal_durability {
  type Frame = APage | ACommit | BPage1 | BPage2 | BCommit
  type LogicalCommit = CommitA | CommitB
  type Phase = Writing | ReadyToSync | Syncing | Synced | Failed | Crashed | Recovered | Returned

  pure val GroupFrames: List[Frame] = List(APage, ACommit, BPage1, BPage2, BCommit)
  pure val AllCommits: Set[LogicalCommit] = Set(CommitA, CommitB)
  pure val AllDurableCounts: Set[int] = Set(0, 1, 2, 3, 4, 5)

  pure def frameLsn(frame: Frame): int =
    match (frame) {
      | APage => 1
      | ACommit => 2
      | BPage1 => 3
      | BPage2 => 4
      | BCommit => 5
    }

  pure val CommitALsn = frameLsn(ACommit)
  pure val CommitBLsn = frameLsn(BCommit)

  pure def recoveredFromPrefix(durableFrameCount: int): Set[LogicalCommit] =
    Set().union(if (durableFrameCount >= CommitALsn) Set(CommitA) else Set())
      .union(if (durableFrameCount >= CommitBLsn) Set(CommitB) else Set())

  pure def replayedCommits(durableFrameCount: int): Set[LogicalCommit] =
    Set().union(if (durableFrameCount >= 2) Set(CommitA) else Set())
      .union(if (durableFrameCount >= 5) Set(CommitB) else Set())

  pure def nextLsnFromPrefix(durableFrameCount: int): int = durableFrameCount + 1

  var writeIndex: int
  var writtenFrames: List[Frame]
  var durableCount: int
  var tornTail: bool
  var syncStarted: bool
  var syncSucceeded: bool
  var appendReturnedSuccess: bool
  var broken: bool
  var recoveredCommits: Set[LogicalCommit]
  var recoveredNextLsn: int
  var phase: Phase

  action init = all {
    writeIndex' = 0,
    writtenFrames' = List(),
    durableCount' = 0,
    tornTail' = false,
    syncStarted' = false,
    syncSucceeded' = false,
    appendReturnedSuccess' = false,
    broken' = false,
    recoveredCommits' = Set(),
    recoveredNextLsn' = 1,
    phase' = Writing,
  }

  action WriteNextFrame = {
    val nextFrames = writtenFrames.append(GroupFrames.nth(writeIndex))
    all {
      phase == Writing,
      writeIndex < GroupFrames.length(),
      writtenFrames == GroupFrames.slice(0, writeIndex),
      durableCount <= writtenFrames.length(),
      writtenFrames' = nextFrames,
      writeIndex' = writeIndex + 1,
      durableCount' = durableCount,
      tornTail' = tornTail,
      syncStarted' = syncStarted,
      syncSucceeded' = syncSucceeded,
      appendReturnedSuccess' = appendReturnedSuccess,
      broken' = broken,
      recoveredCommits' = recoveredCommits,
      recoveredNextLsn' = recoveredNextLsn,
      phase' = if (writeIndex + 1 == GroupFrames.length()) ReadyToSync else Writing,
    }
  }

  action StartSync = all {
    phase == ReadyToSync,
    not (syncStarted),
    writeIndex == GroupFrames.length(),
    phase' = Syncing,
    syncStarted' = true,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    durableCount' = durableCount,
    tornTail' = tornTail,
    syncSucceeded' = syncSucceeded,
    appendReturnedSuccess' = appendReturnedSuccess,
    broken' = broken,
    recoveredCommits' = recoveredCommits,
    recoveredNextLsn' = recoveredNextLsn,
  }

  action SyncSuccess = all {
    phase == Syncing,
    syncStarted,
    not (syncSucceeded),
    durableCount' = GroupFrames.length(),
    syncSucceeded' = true,
    phase' = Synced,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    tornTail' = false,
    syncStarted' = syncStarted,
    appendReturnedSuccess' = appendReturnedSuccess,
    broken' = broken,
    recoveredCommits' = recoveredCommits,
    recoveredNextLsn' = recoveredNextLsn,
  }

  action SyncFailure = {
    nondet persistedCount = AllDurableCounts.oneOf()
    nondet hasTornTail = Set(true, false).oneOf()
    all {
      phase == Syncing,
      syncStarted,
      not (syncSucceeded),
      persistedCount >= durableCount,
      persistedCount <= GroupFrames.length(),
      durableCount' = persistedCount,
      syncSucceeded' = false,
      phase' = Failed,
      broken' = true,
      tornTail' = hasTornTail,
      writeIndex' = writeIndex,
      writtenFrames' = writtenFrames,
      syncStarted' = syncStarted,
      appendReturnedSuccess' = false,
      recoveredCommits' = recoveredCommits,
      recoveredNextLsn' = recoveredNextLsn,
    }
  }

  action PostSyncFailure = all {
    phase == Synced,
    syncSucceeded,
    not (appendReturnedSuccess),
    durableCount == GroupFrames.length(),
    phase' = Failed,
    broken' = true,
    appendReturnedSuccess' = false,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    durableCount' = durableCount,
    tornTail' = tornTail,
    syncStarted' = syncStarted,
    syncSucceeded' = syncSucceeded,
    recoveredCommits' = recoveredCommits,
    recoveredNextLsn' = recoveredNextLsn,
  }

  action AppendSuccess = all {
    phase == Synced,
    syncSucceeded,
    not (broken),
    durableCount == GroupFrames.length(),
    appendReturnedSuccess' = true,
    phase' = Returned,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    durableCount' = durableCount,
    tornTail' = tornTail,
    syncStarted' = syncStarted,
    syncSucceeded' = syncSucceeded,
    broken' = broken,
    recoveredCommits' = recoveredCommits,
    recoveredNextLsn' = recoveredNextLsn,
  }

  action Crash = all {
    phase == Synced or phase == Failed or phase == Returned,
    phase' = Crashed,
    broken' = if (appendReturnedSuccess) false else true,
    appendReturnedSuccess' = appendReturnedSuccess,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    durableCount' = durableCount,
    tornTail' = tornTail,
    syncStarted' = syncStarted,
    syncSucceeded' = syncSucceeded,
    recoveredCommits' = recoveredCommits,
    recoveredNextLsn' = recoveredNextLsn,
  }

  action CrashWithPersistence = {
    nondet persistedCount = AllDurableCounts.oneOf()
    nondet hasTornTail = Set(true, false).oneOf()
    all {
      phase == Writing or phase == ReadyToSync or phase == Syncing,
      not (syncSucceeded),
      persistedCount >= durableCount,
      persistedCount <= writtenFrames.length(),
      phase' = Crashed,
      broken' = true,
      durableCount' = persistedCount,
      tornTail' = hasTornTail,
      appendReturnedSuccess' = false,
      writeIndex' = writeIndex,
      writtenFrames' = writtenFrames,
      syncStarted' = syncStarted,
      syncSucceeded' = syncSucceeded,
      recoveredCommits' = recoveredCommits,
      recoveredNextLsn' = recoveredNextLsn,
    }
  }

  action Recover = all {
    phase == Crashed,
    recoveredCommits' = replayedCommits(durableCount),
    recoveredNextLsn' = nextLsnFromPrefix(durableCount),
    phase' = Recovered,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    durableCount' = durableCount,
    tornTail' = tornTail,
    syncStarted' = syncStarted,
    syncSucceeded' = syncSucceeded,
    appendReturnedSuccess' = appendReturnedSuccess,
    broken' = broken,
  }

  action terminalStutter = all {
    phase == Recovered,
    writeIndex' = writeIndex,
    writtenFrames' = writtenFrames,
    durableCount' = durableCount,
    tornTail' = tornTail,
    syncStarted' = syncStarted,
    syncSucceeded' = syncSucceeded,
    appendReturnedSuccess' = appendReturnedSuccess,
    broken' = broken,
    recoveredCommits' = recoveredCommits,
    recoveredNextLsn' = recoveredNextLsn,
    phase' = phase,
  }

  action step = any {
    WriteNextFrame,
    StartSync,
    SyncSuccess,
    SyncFailure,
    PostSyncFailure,
    AppendSuccess,
    Crash,
    CrashWithPersistence,
    Recover,
    terminalStutter,
  }

  val TypeOK = all {
    writeIndex >= 0,
    writeIndex <= GroupFrames.length(),
    writtenFrames == GroupFrames.slice(0, writeIndex),
    durableCount >= 0,
    durableCount <= writtenFrames.length(),
    phase == Writing or phase == ReadyToSync or phase == Syncing or phase == Synced or phase == Failed or phase == Crashed or phase == Recovered or phase == Returned,
    recoveredCommits.forall(commit => AllCommits.contains(commit)),
    recoveredNextLsn >= 1,
  }

  val WrittenFramesAreOrderedPrefix = writtenFrames == GroupFrames.slice(0, writeIndex)
  val DurableFramesAreWrittenPrefix = durableCount >= 0 and durableCount <= writtenFrames.length()
  val CommitRecoveryRequiresCommitMarker =
    (phase != Recovered or recoveredCommits == recoveredFromPrefix(durableCount)) and
    (not (recoveredCommits.contains(CommitA)) or durableCount >= CommitALsn) and
    (not (recoveredCommits.contains(CommitB)) or durableCount >= CommitBLsn)
  val RecoveredCommitsAreLogicalPrefix =
    recoveredCommits == Set() or recoveredCommits == Set(CommitA) or recoveredCommits == AllCommits
  val SyncSuccessMakesWholeGroupDurable = not (syncSucceeded) or durableCount == GroupFrames.length()
  val AppendSuccessRequiresSyncSuccess = not (appendReturnedSuccess) or
    (syncSucceeded and durableCount == GroupFrames.length())
  val SyncStartsAfterCompleteWrite = not (syncStarted) or writeIndex == GroupFrames.length()
  val FailedAppendNeverReportsSuccess = not (broken) or not (appendReturnedSuccess)
  val RecoveryMatchesDurablePrefixReference = phase != Recovered or
    (recoveredCommits == recoveredFromPrefix(durableCount) and
      recoveredNextLsn == nextLsnFromPrefix(durableCount))
  val TornTailDoesNotCreateCommit =
    (not (tornTail) or phase != Recovered or recoveredCommits == recoveredFromPrefix(durableCount)) and
    (not (tornTail) or not (recoveredCommits.contains(CommitA)) or durableCount >= CommitALsn) and
    (not (tornTail) or not (recoveredCommits.contains(CommitB)) or durableCount >= CommitBLsn)
  val AppendSuccessRecoversWholeGroup = not (appendReturnedSuccess) or recoveredCommits == AllCommits or phase != Recovered

  val state_invariants = all {
    TypeOK,
    WrittenFramesAreOrderedPrefix,
    DurableFramesAreWrittenPrefix,
    CommitRecoveryRequiresCommitMarker,
    RecoveredCommitsAreLogicalPrefix,
    SyncSuccessMakesWholeGroupDurable,
    AppendSuccessRequiresSyncSuccess,
    SyncStartsAfterCompleteWrite,
    FailedAppendNeverReportsSuccess,
    RecoveryMatchesDurablePrefixReference,
    TornTailDoesNotCreateCommit,
    AppendSuccessRecoversWholeGroup,
  }

  pure val RecoveryReferenceTest = all {
    recoveredFromPrefix(0) == Set(),
    recoveredFromPrefix(1) == Set(),
    recoveredFromPrefix(2) == Set(CommitA),
    recoveredFromPrefix(3) == Set(CommitA),
    recoveredFromPrefix(4) == Set(CommitA),
    recoveredFromPrefix(5) == Set(CommitA, CommitB),
    nextLsnFromPrefix(0) == 1,
    nextLsnFromPrefix(2) == 3,
    nextLsnFromPrefix(5) == 6,
  }

  val walPathAReached = phase == Recovered and durableCount == 0 and recoveredCommits == Set()
  val walPathBReached = phase == Recovered and durableCount == 2 and recoveredCommits == Set(CommitA)
  val walPathCReached = phase == Recovered and durableCount == 5 and not (syncSucceeded) and recoveredCommits == AllCommits
  val walPathDReached = phase == Recovered and durableCount == 5 and syncSucceeded and broken and not (appendReturnedSuccess) and recoveredCommits == AllCommits
  val walPathEReached = phase == Recovered and appendReturnedSuccess and recoveredCommits == AllCommits
}
