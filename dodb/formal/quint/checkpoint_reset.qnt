module checkpoint_reset {
  type WalForm = OldWal | EmptyWal | TornResetInit | NewWal
  type Phase = Ready | DataWritten | DataSynced | MetadataWritten | MetadataSynced | InvariantChecked | WalTruncateIssued | WalTruncateSynced | WalInitWritten | WalResetSynced | Completing | Failed | Crashed | Recovered | Done
  type CrashOrigin = NoCrash | BeforeDataSync | AfterDataSync | DuringMetadataWrite | AfterMetadataSync | DuringWalTruncate | AfterWalTruncateSync | DuringWalInitWrite | AfterWalResetSync

  pure val OldState = 0
  pure val LatestState = 2
  pure val OldCheckpointLsn = 0
  pure val TargetCheckpointLsn = 5
  pure val ExpectedNextLsn = 6
  pure val DataStates: Set[int] = Set(OldState, LatestState)
  pure val CheckpointLsns: Set[int] = Set(OldCheckpointLsn, TargetCheckpointLsn)
  pure val WalForms: Set[WalForm] = Set(OldWal, EmptyWal, TornResetInit, NewWal)
  pure val Phases: Set[Phase] = Set(Ready, DataWritten, DataSynced, MetadataWritten, MetadataSynced, InvariantChecked, WalTruncateIssued, WalTruncateSynced, WalInitWritten, WalResetSynced, Completing, Failed, Crashed, Recovered, Done)
  pure val CrashOrigins: Set[CrashOrigin] = Set(NoCrash, BeforeDataSync, AfterDataSync, DuringMetadataWrite, AfterMetadataSync, DuringWalTruncate, AfterWalTruncateSync, DuringWalInitWrite, AfterWalResetSync)

  pure def referenceRecoveredState(dataState: int, checkpointLsn: int, form: WalForm): int =
    match (form) {
      | OldWal => if (checkpointLsn == OldCheckpointLsn) LatestState else dataState
      | EmptyWal => dataState
      | TornResetInit => dataState
      | NewWal => dataState
    }

  pure def referenceRecoveredNextLsn(checkpointLsn: int, form: WalForm): int =
    match (form) {
      | OldWal => ExpectedNextLsn
      | EmptyWal => checkpointLsn + 1
      | TornResetInit => checkpointLsn + 1
      | NewWal => ExpectedNextLsn
    }

  pure def referenceRecoveredHistoryStart(checkpointLsn: int, form: WalForm): int =
    match (form) {
      | OldWal => OldCheckpointLsn
      | EmptyWal => checkpointLsn
      | TornResetInit => checkpointLsn
      | NewWal => TargetCheckpointLsn
    }

  var dataSyncSucceeded: bool
  var durableDataState: int
  var metadataWriteIssued: bool
  var metadataSyncSucceeded: bool
  var durableCheckpointLsn: int
  var invariantCheckPassed: bool
  var walForm: WalForm
  var walTruncateIssued: bool
  var walTruncateSyncSucceeded: bool
  var walInitWriteIssued: bool
  var walInitSyncSucceeded: bool
  var checkpointReturnedSuccess: bool
  var checkpointFailed: bool
  var crashed: bool
  var recovered: bool
  var crashDataState: int
  var crashCheckpointLsn: int
  var crashWalForm: WalForm
  var recoveredState: int
  var recoveredNextLsn: int
  var recoveredHistoryStart: int
  var crashOrigin: CrashOrigin
  var crashDirtyDataWritten: bool
  var phase: Phase

  action init = all {
    dataSyncSucceeded' = false,
    durableDataState' = OldState,
    metadataWriteIssued' = false,
    metadataSyncSucceeded' = false,
    durableCheckpointLsn' = OldCheckpointLsn,
    invariantCheckPassed' = false,
    walForm' = OldWal,
    walTruncateIssued' = false,
    walTruncateSyncSucceeded' = false,
    walInitWriteIssued' = false,
    walInitSyncSucceeded' = false,
    checkpointReturnedSuccess' = false,
    checkpointFailed' = false,
    crashed' = false,
    recovered' = false,
    crashDataState' = OldState,
    crashCheckpointLsn' = OldCheckpointLsn,
    crashWalForm' = OldWal,
    recoveredState' = OldState,
    recoveredNextLsn' = 1,
    recoveredHistoryStart' = OldCheckpointLsn,
    crashOrigin' = NoCrash,
    crashDirtyDataWritten' = false,
    phase' = Ready
  }

  action WriteData = all {
    phase == Ready,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = DataWritten
  }
  action DataSyncSuccess = all {
    phase == DataWritten,
    dataSyncSucceeded' = true,
    durableDataState' = LatestState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = DataSynced
  }
  action WriteCheckpointMetadata = all {
    phase == DataSynced,
    dataSyncSucceeded,
    durableDataState == LatestState,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = true,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = MetadataWritten
  }
  action MetadataSyncSuccess = all {
    phase == MetadataWritten,
    metadataWriteIssued,
    durableDataState == LatestState,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = true,
    durableCheckpointLsn' = TargetCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = MetadataSynced
  }
  action InvariantCheckSuccess = all {
    phase == MetadataSynced,
    metadataSyncSucceeded,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = true,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = InvariantChecked
  }
  action BeginWalTruncate = all {
    phase == InvariantChecked,
    dataSyncSucceeded,
    metadataSyncSucceeded,
    durableCheckpointLsn == TargetCheckpointLsn,
    invariantCheckPassed,
    walForm == OldWal,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = true,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = WalTruncateIssued
  }
  action WalTruncateSyncSuccess = all {
    phase == WalTruncateIssued,
    walTruncateIssued,
    durableDataState == LatestState,
    durableCheckpointLsn == TargetCheckpointLsn,
    invariantCheckPassed,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = EmptyWal,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = true,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = WalTruncateSynced
  }
  action WriteNewWalInit = all {
    phase == WalTruncateSynced,
    walTruncateSyncSucceeded,
    walForm == EmptyWal,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = true,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = WalInitWritten
  }
  action WalInitSyncSuccess = all {
    phase == WalInitWritten,
    walInitWriteIssued,
    walForm == EmptyWal,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = NewWal,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = true,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = WalResetSynced
  }
  action BeginCompletion = all {
    phase == WalResetSynced,
    walInitSyncSucceeded,
    walForm == NewWal,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = Completing
  }
  action CheckpointSuccess = all {
    phase == Completing,
    durableDataState == LatestState,
    durableCheckpointLsn == TargetCheckpointLsn,
    walForm == NewWal,
    walInitSyncSucceeded,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = true,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = Done
  }
  action CompletionFailure = all {
    phase == Completing,
    durableDataState == LatestState,
    durableCheckpointLsn == TargetCheckpointLsn,
    walForm == NewWal,
    walInitSyncSucceeded,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = true,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = Failed
  }
  action CrashBeforeDataSync = {
nondet snapshotDataState = Set(OldState, LatestState).oneOf()
  all {
    phase == Ready or phase == DataWritten,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = snapshotDataState,
    crashCheckpointLsn' = OldCheckpointLsn,
    crashWalForm' = OldWal,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = BeforeDataSync,
    crashDirtyDataWritten' = phase == DataWritten,
    phase' = Crashed
  }
  }
  action CrashAfterDataSync = all {
    phase == DataSynced,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = OldCheckpointLsn,
    crashWalForm' = OldWal,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = AfterDataSync,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  action CrashDuringMetadataWrite = {
nondet selectedCheckpointLsn = Set(OldCheckpointLsn, TargetCheckpointLsn).oneOf()
  all {
    phase == MetadataWritten,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = selectedCheckpointLsn,
    crashWalForm' = OldWal,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = DuringMetadataWrite,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  }
  action CrashAfterMetadataSync = all {
    phase == MetadataSynced or phase == InvariantChecked,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = TargetCheckpointLsn,
    crashWalForm' = OldWal,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = AfterMetadataSync,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  action CrashDuringWalTruncate = {
nondet persistedWalForm = Set(OldWal, EmptyWal).oneOf()
  all {
    phase == WalTruncateIssued,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = TargetCheckpointLsn,
    crashWalForm' = persistedWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = DuringWalTruncate,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  }
  action CrashAfterWalTruncateSync = all {
    phase == WalTruncateSynced,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = TargetCheckpointLsn,
    crashWalForm' = EmptyWal,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = AfterWalTruncateSync,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  action CrashDuringWalInitWrite = {
nondet persistedInitForm = Set(EmptyWal, TornResetInit, NewWal).oneOf()
  all {
    phase == WalInitWritten,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = TargetCheckpointLsn,
    crashWalForm' = persistedInitForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = DuringWalInitWrite,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  }
  action CrashAfterWalResetSync = all {
    phase == WalResetSynced or phase == Completing or phase == Failed or phase == Done,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = true,
    recovered' = recovered,
    crashDataState' = LatestState,
    crashCheckpointLsn' = TargetCheckpointLsn,
    crashWalForm' = NewWal,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = AfterWalResetSync,
    crashDirtyDataWritten' = true,
    phase' = Crashed
  }
  action Recover = all {
    crashed,
    phase == Crashed,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = false,
    recovered' = true,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = match (crashWalForm) { | OldWal => if (crashCheckpointLsn == OldCheckpointLsn) LatestState else crashDataState | EmptyWal => crashDataState | TornResetInit => crashDataState | NewWal => crashDataState },
    recoveredNextLsn' = match (crashWalForm) { | OldWal => ExpectedNextLsn | EmptyWal => crashCheckpointLsn + 1 | TornResetInit => crashCheckpointLsn + 1 | NewWal => ExpectedNextLsn },
    recoveredHistoryStart' = match (crashWalForm) { | OldWal => OldCheckpointLsn | EmptyWal => crashCheckpointLsn | TornResetInit => crashCheckpointLsn | NewWal => TargetCheckpointLsn },
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = Recovered
  }
  action terminalStutter = all {
    phase == Done or phase == Recovered,
    dataSyncSucceeded' = dataSyncSucceeded,
    durableDataState' = durableDataState,
    metadataWriteIssued' = metadataWriteIssued,
    metadataSyncSucceeded' = metadataSyncSucceeded,
    durableCheckpointLsn' = durableCheckpointLsn,
    invariantCheckPassed' = invariantCheckPassed,
    walForm' = walForm,
    walTruncateIssued' = walTruncateIssued,
    walTruncateSyncSucceeded' = walTruncateSyncSucceeded,
    walInitWriteIssued' = walInitWriteIssued,
    walInitSyncSucceeded' = walInitSyncSucceeded,
    checkpointReturnedSuccess' = checkpointReturnedSuccess,
    checkpointFailed' = checkpointFailed,
    crashed' = crashed,
    recovered' = recovered,
    crashDataState' = crashDataState,
    crashCheckpointLsn' = crashCheckpointLsn,
    crashWalForm' = crashWalForm,
    recoveredState' = recoveredState,
    recoveredNextLsn' = recoveredNextLsn,
    recoveredHistoryStart' = recoveredHistoryStart,
    crashOrigin' = crashOrigin,
    crashDirtyDataWritten' = crashDirtyDataWritten,
    phase' = phase
  }

  action step = any {
    CrashBeforeDataSync,
    CrashAfterDataSync,
    CrashDuringMetadataWrite,
    CrashAfterMetadataSync,
    CrashDuringWalTruncate,
    CrashAfterWalTruncateSync,
    CrashDuringWalInitWrite,
    CrashAfterWalResetSync,
    WriteData,
    DataSyncSuccess,
    WriteCheckpointMetadata,
    MetadataSyncSuccess,
    InvariantCheckSuccess,
    BeginWalTruncate,
    WalTruncateSyncSuccess,
    WriteNewWalInit,
    WalInitSyncSuccess,
    BeginCompletion,
    CheckpointSuccess,
    CompletionFailure,
    Recover,
    terminalStutter
  }

  val TypeOK = all {
    DataStates.contains(durableDataState),
    CheckpointLsns.contains(durableCheckpointLsn),
    WalForms.contains(walForm),
    DataStates.contains(crashDataState),
    CheckpointLsns.contains(crashCheckpointLsn),
    WalForms.contains(crashWalForm),
    DataStates.contains(recoveredState),
    recoveredNextLsn >= 1 and recoveredNextLsn <= ExpectedNextLsn,
    CheckpointLsns.contains(recoveredHistoryStart),
    CrashOrigins.contains(crashOrigin),
    Phases.contains(phase),
    not (checkpointReturnedSuccess and checkpointFailed),
    crashed implies phase == Crashed,
    recovered implies phase == Recovered or phase == Done,
    checkpointReturnedSuccess implies phase == Done or phase == Crashed or phase == Recovered
  }

  val DurableCheckpointNeverExceedsDurableData =
    (durableCheckpointLsn != TargetCheckpointLsn or durableDataState == LatestState) and
    (crashCheckpointLsn != TargetCheckpointLsn or crashDataState == LatestState)

  val WalReclamationRequiresDurableCheckpoint =
    (walForm == OldWal or (durableCheckpointLsn == TargetCheckpointLsn and durableDataState == LatestState)) and
    (crashWalForm == OldWal or (crashCheckpointLsn == TargetCheckpointLsn and crashDataState == LatestState))

  val WalResetRequiresInvariantGate = not (walTruncateIssued) or invariantCheckPassed

  val NewWalStartsAtCheckpoint =
    (walForm != NewWal or durableCheckpointLsn == TargetCheckpointLsn) and
    (crashWalForm != NewWal or crashCheckpointLsn == TargetCheckpointLsn)

  val CheckpointSuccessRequiresStableReset = not (checkpointReturnedSuccess) or all {
    durableDataState == LatestState,
    durableCheckpointLsn == TargetCheckpointLsn,
    walForm == NewWal,
    walInitSyncSucceeded
  }

  val RecoveryMatchesReference = not (recovered) or all {
    recoveredState == referenceRecoveredState(crashDataState, crashCheckpointLsn, crashWalForm),
    recoveredNextLsn == referenceRecoveredNextLsn(crashCheckpointLsn, crashWalForm),
    recoveredHistoryStart == referenceRecoveredHistoryStart(crashCheckpointLsn, crashWalForm)
  }

  val RecoveryPreservesLatestCommittedState = not (recovered) or recoveredState == LatestState
  val RecoveryPreservesLsnFloor = not (recovered) or recoveredNextLsn == ExpectedNextLsn

  val CheckpointFailureDoesNotImplyRollback =
    not (checkpointFailed) or (not (checkpointReturnedSuccess) and durableDataState == LatestState and durableCheckpointLsn == TargetCheckpointLsn and walForm == NewWal and walInitSyncSucceeded)

  val MetadataRequiresDurableData = not (metadataWriteIssued) or dataSyncSucceeded
  val TruncateRequiresDurableCheckpoint = not (walTruncateIssued) or (metadataSyncSucceeded and durableCheckpointLsn == TargetCheckpointLsn and durableDataState == LatestState)
  val WalInitRequiresTruncateSync = not (walInitWriteIssued) or walTruncateSyncSucceeded

  val state_invariants = all {
    TypeOK,
    DurableCheckpointNeverExceedsDurableData,
    WalReclamationRequiresDurableCheckpoint,
    WalResetRequiresInvariantGate,
    NewWalStartsAtCheckpoint,
    CheckpointSuccessRequiresStableReset,
    RecoveryMatchesReference,
    RecoveryPreservesLatestCommittedState,
    RecoveryPreservesLsnFloor,
    CheckpointFailureDoesNotImplyRollback,
    MetadataRequiresDurableData,
    TruncateRequiresDurableCheckpoint,
    WalInitRequiresTruncateSync
  }

  pure val RecoveryReferenceTest = all {
    referenceRecoveredState(OldState, OldCheckpointLsn, OldWal) == LatestState,
    referenceRecoveredNextLsn(OldCheckpointLsn, OldWal) == ExpectedNextLsn,
    referenceRecoveredState(LatestState, TargetCheckpointLsn, OldWal) == LatestState,
    referenceRecoveredNextLsn(TargetCheckpointLsn, OldWal) == ExpectedNextLsn,
    referenceRecoveredState(LatestState, TargetCheckpointLsn, EmptyWal) == LatestState,
    referenceRecoveredNextLsn(TargetCheckpointLsn, EmptyWal) == ExpectedNextLsn,
    referenceRecoveredState(LatestState, TargetCheckpointLsn, TornResetInit) == LatestState,
    referenceRecoveredNextLsn(TargetCheckpointLsn, TornResetInit) == ExpectedNextLsn,
    referenceRecoveredState(LatestState, TargetCheckpointLsn, NewWal) == LatestState,
    referenceRecoveredNextLsn(TargetCheckpointLsn, NewWal) == ExpectedNextLsn,
    referenceRecoveredHistoryStart(TargetCheckpointLsn, EmptyWal) == TargetCheckpointLsn,
    referenceRecoveredHistoryStart(TargetCheckpointLsn, TornResetInit) == TargetCheckpointLsn,
    referenceRecoveredHistoryStart(TargetCheckpointLsn, NewWal) == TargetCheckpointLsn,
    referenceRecoveredNextLsn(OldCheckpointLsn, EmptyWal) == 1
  }

  val checkpointPathAReached = recovered and crashOrigin == BeforeDataSync and crashDirtyDataWritten and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathBReached = recovered and crashOrigin == AfterDataSync and crashCheckpointLsn == OldCheckpointLsn and crashWalForm == OldWal
  val checkpointPathCZeroReached = recovered and crashOrigin == DuringMetadataWrite and crashCheckpointLsn == OldCheckpointLsn and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathCFiveReached = recovered and crashOrigin == DuringMetadataWrite and crashCheckpointLsn == TargetCheckpointLsn and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathDOldWalReached = recovered and crashOrigin == DuringWalTruncate and crashWalForm == OldWal and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathDEmptyWalReached = recovered and crashOrigin == DuringWalTruncate and crashWalForm == EmptyWal and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathEReached = recovered and crashOrigin == AfterWalTruncateSync and crashWalForm == EmptyWal and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathFReached = recovered and crashOrigin == DuringWalInitWrite and crashWalForm == TornResetInit and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
  val checkpointPathGReached = checkpointReturnedSuccess and phase == Done and durableDataState == LatestState and durableCheckpointLsn == TargetCheckpointLsn and walForm == NewWal
  val checkpointPathHReached = recovered and checkpointFailed and not (checkpointReturnedSuccess) and crashOrigin == AfterWalResetSync and recoveredState == LatestState and recoveredNextLsn == ExpectedNextLsn
}
