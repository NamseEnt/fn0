# Project telemetry policy rollout runbook

`ProjectDoc.telemetry_policy` is the only source of a project tenant's
policy. It stores the policy meaning selected for that project:

- the tenant base retention;
- whether each of log, trace, and metric has an override, and the value when
  it does;
- the storage limit, including an explicitly selected `infinite` value; and
- the monotonic policy revision.

An absent signal override is not an effective value. It means that the signal
inherits the base retention. The migration and semantic-diff checks must keep
that distinction. If Signy treats an omitted storage limit and the explicit
`unlimited` representation as equivalent, the migration canonicalizes them to
the one schema representation and tests that equivalence; no other policy
values are defaulted.

The new-project control path selects and records `30d` base retention,
`None` for each signal override, and `512MiB` storage. This is a concrete
policy for that project, not a Signy fallback, platform policy, or installer
default. A future subscription-tier selector may choose different concrete
values, but the selected values must be written to `ProjectDoc` before the
outbox is created.

## State machine

The rollout uses these operational states. The state and its evidence must be
recorded before moving to the next state:

`WritersRunning` -> `Frozen` -> `TargetCaptured` -> `Migrating` ->
`ExceptionsPending` -> `SemanticVerified` -> `SchemaReady` ->
`NewControlDeployed` -> `WritersResumed`.

`ExceptionsPending` is a holding state, not a successful migration result. A
project may leave it only after an operator has selected a concrete policy and
the normal revision/outbox path has made the ProjectDoc and Signy meanings
equal.

## Preflight and freeze

1. Keep production, dry-run deployment, migration execution, and R2 deletion
   disabled until the compaction crash-window change has passed its
   process-level fault-injection test suite.
2. Record the exact old control image, database endpoint, Signy endpoint, and
   migration control build. Do not use an installer or a seed script to create
   a platform tenant policy.
3. Stop the old control writer, queue consumers, scheduled reconciliation, and
   any administrative writer of `ProjectDoc`. Confirm from process and queue
   telemetry that no old writer can continue to commit during migration.
4. Keep an authenticated read-only observation path available for the
   database, Signy, outbox, deletion tombstones, and migration exceptions.
   Do not modify data during this check.

If the freeze cannot be proven, remain in `WritersRunning`; do not attempt a
best-effort migration against live writers.

## Target capture

With writers frozen, scan the raw `ProjectDoc` collection and record the
target inventory, scan start/end evidence, and the database revision or
equivalent consistency evidence available from the control database. The
target is the set of existing project documents at this freeze boundary. If a
writer or an out-of-scope document is observed, abort the run and return to
`Frozen` before restarting the scan.

The migration task deliberately reads legacy documents as raw JSON before the
final typed required-field reader is enabled. It must not use the new-project
values to fill an old document.

## Migration execution

For every target project:

1. If the document already has a valid policy, compare its semantic meaning
   with the actual Signy policy. Do not compare only effective per-signal
   values: base retention and override presence must match.
2. If the old document has no policy, read the project's actual Signy tenant
   policy. Copy the base retention, every override's presence and value, the
   storage limit, explicit `infinite`, and the existing revision meaning.
3. If Signy has no explicit policy, write only a
   `TelemetryPolicyMigrationExceptionDoc` containing the project and reason.
   Do not write `MigrationRequired`, a common period, `infinite`, or any other
   placeholder into `ProjectDoc`.
4. Claim the unchanged Signy values under the project-managed policy path at
   the current revision. This metadata claim changes ownership semantics, not
   retention or storage values. It is protected by Signy's CAS and is safe to
   repeat.
5. Write the raw ProjectDoc policy and its outbox record in an optimistic
   transaction. The transaction re-reads the document and refuses to replace
   a policy or outbox revision that a newer control writer has already
   committed.
6. Ensure the outbox record exists and points at the ProjectDoc revision.
   If document persistence succeeded but queue submission failed, the
   reconciliation scan recreates the submission. A missing or failed outbox
   record remains observable and blocks completion.

The migration is idempotent. A rerun may re-read Signy and ProjectDoc, but it
must not change policy values, materialize missing overrides, or advance a
revision merely because the previous run crashed.

## Exception resolution

After the first pass, export the exception list with project identifier and
reason. For each exception, an operator must explicitly choose the concrete
tenant policy. The resolution must use the normal control policy-change path:

1. write the chosen base, optional overrides, storage value, and next revision
   to `ProjectDoc`;
2. create or update the durable outbox record; and
3. let the queue apply the same revision and body to Signy.

The operator's choice is not inferred from the policy for a new project. The
exception is cleared only after a semantic comparison confirms that
ProjectDoc and Signy agree. Until then, the project remains an exception and
the required-field rollout cannot complete.

## Revision and deletion fences

Signy project policy writes use the following comparison against the current
project-managed revision:

- request revision lower than current: stale no-op;
- equal revision with equal policy body: idempotent success;
- equal revision with a different body: conflict;
- higher revision: apply, even when one or more revisions were skipped;
- any revision from a revoked generation: reject.

The project generation/access fence is checked with the revision. A generic
operator PUT cannot mutate a project-managed tenant through a path that skips
these checks. Legacy policies are claimed at their existing values before
ProjectDoc is written, so ownership changes without changing the actual
retention or storage policy.

Deletion is independent of retention and purge. The delete action first
records a durable `revoke_pending` tombstone. Until Signy durably confirms the
access revoke, the project is not considered torn down. Revoke errors leave
the tombstone in `revoke_pending` and the queue retries it. After successful
revoke, teardown may continue, but it does not change the Signy retention
policy or delete Signy telemetry.

The tombstone remains as the deletion fence. A redelivered or delayed policy
registration task must observe it and cannot restore access or register a
deleted project. The `revoke_pending` count and oldest age are separate
operational signals because access may remain available during that interval.

## Verification gates

Run the following gates while writers remain frozen:

1. Count all target ProjectDocs that lack the required policy. The count must
   be zero after operator resolution.
2. Count `TelemetryPolicyMigrationExceptionDoc` records. The count must be
   zero; deleting an exception without resolving the policy does not satisfy
   this gate.
3. Compare every ProjectDoc with Signy using a semantic diff of base
   retention, each override's presence and value, storage limit, explicit
   `infinite`, and the accepted omitted-versus-`unlimited` storage
   equivalence. Revision comparisons must obey the revision rules above.
4. Verify every in-scope ProjectDoc has an outbox record at the same or newer
   revision, and that no outbox is `pending`, `failed`, or `conflict`.
5. Run the migration a second time. It must be a no-op with zero exceptions,
   no policy-value change, no revision churn, and no new outbox work.
6. Confirm the old writer, old queue consumer, and old reconciliation process
   cannot resume before the new control is deployed.

Only after all six gates pass is the database in `SchemaReady`.

## Deployment and resume

1. Deploy the control build whose typed `ProjectDoc.telemetry_policy` field is
   required and whose queue/reconciliation code understands the outbox,
   revisions, and deletion fence.
2. Verify the new control can read every migrated ProjectDoc and can preserve
   the policy field on every write. Do not run the old writer against a
   database containing the new required field.
3. Start queue consumers and reconciliation with the new control. Confirm
   pending, failed, and conflict counts remain zero before resuming ordinary
   writers.
4. Resume the new control writer. New projects continue to accept only
   `name` at the API boundary; control records the concrete selected policy
   and queues it for Signy.
5. Do not gate deploy or export on policy registration. During registration
   delay, Signy may drop that tenant's telemetry. Such drops must not be
   counted as successfully stored data. Observe registration delay and
   tenant-not-served drops through aggregate metrics without putting tenant
   IDs in metric labels.
6. Observe registration failures, stale results, conflicts, enqueue
   corrections, `revoke_pending`, and migration exceptions after resume.

## Failure, restart, and rollback

The safe resume point after any migration failure is the same compatibility
migration build or a newer build, with writers still frozen:

- crash before the Signy claim: rerun the claim;
- crash after the Signy claim but before the ProjectDoc transaction: rerun the
  metadata claim and document write;
- crash after the ProjectDoc commit but before queue submission: rerun the
  outbox check and enqueue correction;
- crash after an outbox commit but before delivery: let the queue retry;
- a Signy failure: retain the outbox failure state and retry; do not alter the
  ProjectDoc to make the failure disappear;
- an optimistic transaction conflict: reread the latest ProjectDoc and rerun;
  the migration must never move its revision backward.

Do not resume the old writer after any migration write. A rollback is allowed
only to a control build that can read and preserve the required policy field,
outbox documents, and unknown future fields without dropping them. Never
restore a database snapshot that predates a migrated ProjectDoc, send a lower
Signy revision, lower a deletion fence, or use a generic PUT to undo a project
claim. If the new control image fails after migration, keep the migrated
database frozen and redeploy the same compatible build or a newer one.

## Completion and observation

The rollout is complete only when all of the following are true:

- every existing ProjectDoc has a real, required policy;
- every policy has the same meaning as its actual Signy tenant policy;
- migration exceptions are zero and no exception was cleared without an
  operator-selected policy;
- a rerun is idempotent;
- outbox pending, failed, and conflict counts are zero;
- no project is reported as fully torn down while its Signy revoke is pending;
- aggregate metrics show no unexplained registration delay or drop increase;
- the release and rollback versions are recorded.

At minimum, monitor:

- `fn0.telemetry.policy.pending`;
- `fn0.telemetry.policy.oldest_pending_age_seconds`;
- `fn0.telemetry.policy.sync.attempts`, `.success`, `.failure`, `.stale`, and
  `.conflict`;
- `fn0.telemetry.policy.enqueue_corrections`;
- `fn0.telemetry.policy.revoke_pending` and
  `fn0.telemetry.policy.oldest_revoke_pending_age_seconds`;
- `fn0.telemetry.policy.migration_exceptions`; and
- Signy's aggregate `tenant_not_served` drop counter with signal-level
  aggregation.

Tenant identifiers and detailed error text belong in authenticated diagnostics
or a bounded diagnostic view, not unbounded metric labels.

No step in this runbook automatically changes retention, chooses an implicit
infinite policy, purges telemetry, or deletes R2 data.
