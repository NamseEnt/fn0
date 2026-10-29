# Frozen shutdown diagnostic matrix

Tracking issue: https://github.com/NamseEnt/fn0/issues/126

Runtime source: `b22638bdcc4a0b22dad0606c3003d193cf41bbed`. Controller/plan commit is recorded separately after review. Run only after all seventy-nine checks, unchanged advisory Clippy diagnostics, and default/A1 links pass, with the default link last. Use namespace `shutdown-b22638bdc-r1`; do not reuse either earlier observation namespace.

## Exact conditions

Retain the designated two-vCPU OCI host, ZFS dataset/policies, pinned native archive, and relocated target symlink. No build, compression, or other workload overlaps these cells. Host ARC/swap policy is unchanged; driver, uncharged ARC, and already elsewhere-charged shared pages remain outside the backend budget.

Use the immutable capacity controller's arguments: backend maximum/high 256/192 MiB across cell and epochs, zero backend swap, 512-byte values, seed 20261027, mixed Get/Query/Write 40/20/40, width 1/4/16 at 80/15/5, fifty arrivals/s per active tenant, synchronized absolute-deadline lanes, global in-flight and queue bounds eight, request timeout five seconds, drain fifteen seconds, worker-internal asynchronous shutdown fifteen seconds, parent worker-stop watchdog twenty seconds, and controller deadline twenty minutes. Keep B-link segmented/coalesced background checkpoints at the original 256 KiB segment / 1 MiB threshold and the reference's shared 64 MiB cache / 32 MiB WBM / 1 MiB write buffer, one flush/compaction thread each, open-file limit sixty-four, disabled stalls. A1 remains disabled.

1. One tiny E2 pair: one registered=active tenant, 2,048 seed rows, 2,000 ms offer, 1,000 ms cadence, B-link first. Expect one hundred scheduled API calls per route; this is accounting/trace correctness, with too few Query observations for the capacity SLO. Require both full-population oracles and complete shutdown. Verify stderr identities, normal phase order, and absence of partial-failure metadata in the success report.
2. Up to three fresh E3 pairs: five registered=active tenants, 32,768 seed rows each, 60,000 ms offer, 1,000 ms cadence. Shared first in repetition one, B-link first in repetition two, shared first in repetition three. Record actual requests and both oracles on completions. These are shutdown-attribution observations, not a performance ranking or a replacement for the stopped source-023 cohort.

Stop immediately at the first operational failure, unexplained report, mismatched identity/configuration/accounting, or reserve breach. Preserve its raw report, original status/review error, phase logs, binaries, database, and groups. Do not repeat an unchanged failure, resume omitted pairs silently, or relax either shutdown deadline. The old controller's valid-completion reducer may reject the failed report and stop; that error must remain preserved.

Keep the twelve-GiB pre-cell and ten-GiB runtime free-space guards and 1.5 GiB host MemAvailable guard. Successful process groups must be empty before the next cell. A completed sequence has eight cells; a stopped sequence has a failure receipt and no synthetic completion.

## Interpretation

Parse the worker's structured stderr events by PID/start-time/source identity. Keep process identity, timestamp, stage order, completed stage intervals, last observed marker, and incomplete stage pairs. In particular distinguish command receipt, final native sampling/properties, asynchronous server shutdown, final checkpoint metric capture, backend destruction, health-report writing, async-run return, and runtime destruction. Missing markers can result from a stalled operation or failed logging; do not fabricate an end event.

Failure counters tagged `available_unvalidated_after_later_failure` are actual returned workload summaries retained outside the backend budget. They are not an accepted cell, successful oracle, or validated SLO. Keep unavailable counters explicit for earlier aborts. The success JSON shape remains unchanged.

If a failure recurs, report the last stage and watchdog interval without attributing an internal engine cause from timing alone. If all pairs finish, report that this bounded new variant did not reproduce the earlier failure. Logging and extra driver retention have costs; neither outcome proves that diagnostics fixed shutdown or that a production tenant maximum was found. Issue #126 stays open.
