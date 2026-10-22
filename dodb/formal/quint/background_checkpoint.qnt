module background_checkpoint {
  type State = {
    latest: int, target: int, checkpoint: int, disk: int, phase: int,
    required: Set[int], files: Set[int], stopped: bool, recovered: int
  }

  var state: State

  action init = state' = {
    latest: 0, target: 0, checkpoint: 0, disk: 0, phase: 0,
    required: Set(), files: Set(), stopped: false, recovered: 0
  }

  action write = all {
    not(state.stopped), state.latest < 4,
    state' = state.with("latest", state.latest + 1)
      .with("required", state.required.union(Set(state.latest + 1)))
      .with("files", state.files.union(Set(state.latest + 1)))
  }

  action snapshot = all {
    not(state.stopped), state.phase == 0, state.latest > state.checkpoint,
    state' = state.with("target", state.latest).with("phase", 1)
  }

  action syncData = all {
    not(state.stopped), state.phase == 1,
    state' = state.with("disk", state.target).with("phase", 2)
  }

  action publishLatestManifest = all {
    not(state.stopped), state.phase == 2,
    state' = state.with("checkpoint", state.target).with("phase", 3)
      .with("required", state.required.filter(commit => commit > state.target))
  }

  action reclaim = all {
    not(state.stopped), state.phase == 3,
    state' = state.with("files", state.files.filter(commit => commit > state.checkpoint))
      .with("phase", 0)
  }

  action crash = all {
    not(state.stopped),
    state' = state.with("stopped", true)
      .with("recovered", state.required.fold(state.checkpoint, (latest, commit) => if (commit > latest) commit else latest))
  }

  action step = any { write, snapshot, syncData, publishLatestManifest, reclaim, crash, all { state.stopped, state' = state } }

  val AcknowledgedCovered = 1.to(state.latest).forall(commit =>
    commit <= state.checkpoint or (state.required.contains(commit) and state.files.contains(commit)))
  val CheckpointHasSyncedData = state.checkpoint <= state.disk and state.disk <= state.latest
  val ManifestPreservesConcurrentWrites = state.required.forall(commit => commit > state.checkpoint)
  val RecoveryPreservesAcknowledged = not(state.stopped) or state.recovered == state.latest
  val state_invariants = AcknowledgedCovered and CheckpointHasSyncedData
    and ManifestPreservesConcurrentWrites and RecoveryPreservesAcknowledged
  val overlapReached = state.phase == 1 and state.latest > state.target

  run concurrentRotationTest = init.then(write).then(snapshot).then(write)
    .then(syncData).then(publishLatestManifest).then(reclaim).then(crash)
    .expect(state_invariants and state.latest == 2 and state.checkpoint == 1 and state.recovered == 2)
}
