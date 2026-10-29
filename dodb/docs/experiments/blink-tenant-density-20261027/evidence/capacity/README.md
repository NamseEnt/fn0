# Capacity evidence

Runtime source: `aaa40565529ca92150df5d94e19b02ca93859f71`. Read [the result](../../DENSITY-CAPACITY-RESULTS.md) before interpreting the counters.

Run `PYTHONDONTWRITEBYTECODE=1 python3 review_all.py` from this directory. It verifies the two raw archive hashes, regenerates each completed view's independent reduction, checks original report/status immutability, and writes `final-review.json`. It does not launch workloads or contact OCI.

- `runtime/capacity-aaa405655-r1` is the stopped original namespace: 36 actual executions and an original classifier failure after a setup-worker shutdown deadline. It has no whole-sequence completion receipt.
- `runtime/capacity-aaa405655-r2` is the complete composite view: the same 36 raw reports plus sixteen new executions. Reused reports are byte-identical; the deadline classification is a documented derived status. Its `completion.json` distinguishes reused and newly executed cases.
- `runtime/capacity-supplement-aaa405655-r1` contains five additional actual executions: three independent shared-reference E3 confirmations and one paired actual-boundary maintenance observation.

Do not sum all three directories' case counts. There are 57 distinct executions, 55 operational completions, 52 experimental SLO passes, three completed SLO misses, and two pre-offer operational failures. Only complete reports contribute to the 2,304,000 successful offered-request count. Neither failed setup has a measured zero-throughput interval.

`collection/` contains launcher/PID receipts and postflight checks of source, frozen binaries, exited process identities, preserved databases, cgroups, and host capacity. The original/composite database records overlap; the combined unique set is 57 paths and 279 process identities. The raw archives omit binaries and databases, which remain on the designated OCI host.

`validation/`, `local-validation-review.json`, and `clippy-review.json` preserve the 68 clean-source tests, feature links, and unchanged seventeen advisory Clippy findings. `continuation-preflight.json` preserves all-report and negative classifier checks. Controller/reducer source files beside this README reproduce the executed protocols; their hashes are recorded in runtime provenance.

Offer-boundary kernel peaks and final epoch peaks have separate fields in `frontier.json`. The latter can include substantial diagnostic serialization after backend shutdown. The serialized observation limit is not a heap bound. Driver memory and host ARC are outside the backend cgroup ceiling. Durations, cadences, all-active/sparse populations, and operational failure kinds must remain separate.
