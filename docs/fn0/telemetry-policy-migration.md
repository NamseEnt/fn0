# Project telemetry policy migration

`ProjectDoc.telemetry_policy` is the only source of a project tenant's
policy. It contains the Signy base retention, the presence and value of each
signal override, the storage limit, and the revision. An absent signal
override remains absent so that Signy's inheritance behavior survives the
migration. `infinite` is a valid value only when it is explicitly present in
the policy selected for that tenant.

## Rollout order

The required-field rollout uses the writer-free migration sequence:

1. Stop the old control deployment, queue consumers, cron invocation, and any
   other writer of `ProjectDoc`. Confirm that no old writer can continue to
   write during the migration window.
2. Run the one-shot `migrate_telemetry_policies` admin task against the control
   database. It scans raw documents so old documents can be inspected before
   the final required-field reader is enabled.
3. For a document without the new policy shape, read that project's actual
   Signy policy. If it does not exist, write a
   `TelemetryPolicyMigrationExceptionDoc` and leave the project unchanged.
   Never backfill a common duration, `infinite`, or an effective signal value.
4. Claim existing Signy values through the project endpoint without changing
   their retention or storage values, then write the raw base/override shape
   to `ProjectDoc` and create its outbox record. The Signy claim is
   same-revision metadata-only and is protected by the object-store CAS.
5. Rerun the migration task until the exception list is empty. Reruns are
   safe after a crash between the Signy claim, document write, and outbox
   write: the claim is idempotent and the outbox is recreated if needed.
6. Verify every migrated document against Signy by comparing policy meaning:
   base retention, override presence and values, storage limit, and explicit
   `infinite`. Then deploy the final control build whose ProjectDoc field is
   required.

During the freeze, old writers cannot lose the new field because they are
stopped before the first migration write. A policy update made by the final
control build after a document is migrated is serialized by the same revision
and outbox rules. The queue reads the latest ProjectDoc at execution time, so
multiple pending messages converge on that latest value.

## Completion conditions

The rollout is complete only when:

- every existing `ProjectDoc` has a real policy;
- every policy has the same meaning as the actual Signy tenant policy;
- the migration exception count is zero;
- no old writer or queue consumer remains active during migration;
- rerunning the migration makes no policy-value or revision change;
- the policy sync and enqueue-correction metrics show no unresolved backlog;
- deletion tombstones show no unexpected `revoke_pending` backlog.

Projects with no Signy policy remain operator exceptions. Their physical
telemetry is neither automatically deleted nor treated as an implicit normal
infinite policy. New project creation continues to store the concrete
control-selected policy (`30d` base retention and `512MiB` storage) in the
new document; those values are not a Signy fallback or an installer default.
