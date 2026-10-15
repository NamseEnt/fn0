# Validation attempts

## Local checks

`DODB_ROCKSDB_DIR=/tmp/dodb-r1a-rocksdb-abeebd9 cargo check --locked
--offline -p dodb-rocksdb --bin service_workload --tests` passed.
`cargo fmt -p dodb-rocksdb -- --check`, Python source compilation, and
`git diff --check` passed. Executable Rust checks are the OCI evidence below;
no local Rust test execution is claimed.

Strict Clippy was attempted, without adding lint suppressions. The command
with dependencies stopped at 11 existing dodb-storage findings, including
argument counts, collapsible conditionals, and needless slicing. A focused
`--no-deps` attempt stopped at nine existing dodb-rocksdb findings, including
collapsible conditionals, matching Some(ok()), argument count in
query_with_metrics, and iterator return-type complexity. These source sites
already existed before the attribution change. This report does not claim a
strict Clippy pass; unrelated lint cleanup was not included.

## OCI focused attempt 1

Source `61535c8ab2d77cfeac6234cd27dc57b2925b158a` compiled successfully.
The external focused runner set DODB_TEST_DATA_DIR without creating that
parent. The library executable ran 21 cases: one passed and 20 failed while
creating their temporary directories, before RocksDB execution. The other
four test executables were not reached. The original log and manifest remain
under `oci/focused-attempt-01-missing-parent/`. The runner was corrected to
create the parent. No workload or performance run had started.

## OCI focused attempt 2

With the prepared ZFS parent, the same source passed 21 library cases and two
read contract cases. Research metrics ran five cases: four passed, including
the new exact role/byte/phase/rejection accounting case, and one failed. The
existing aggregate accounting assertion still expected six read semaphore
samples for three warm reads, from the removed path-existence submission.
Actual count was three, consistent with the current initialized-tenant path.
The driver and transaction-contract executables were not reached.

The failure remains under `oci/focused-attempt-02-stale-read-count/`. Commit
`d56382ef236542846b3bb836570566f835b2a0e3` changes that expectation from six
to three; it does not alter the execution or instrumentation implementation.

## Accepted focused execution

From the final clean source, all five compiled executables passed: 21 library,
two read-contract, five research-metrics, three driver, and one shared
transaction-contract case. Total 32 passed, zero failed, zero not reached.
Both TMPDIR and DODB_TEST_DATA_DIR were on ZFS. Test binary command arrays,
before/after hashes, exit status, and full logs are retained in `oci/focused/`.
The build-only helper used --no-run; the external runner executed the
produced binaries separately.

Process-crash and power-loss tests were not run for this instrumentation-only
change. Performance attempts and their completion counts are recorded in the
final report and the retained matrix log.

## Overwrite follow-up

The first local probe compile found Rust E0507: moving the Scan result vector
inside a pattern guard. The comparison moved into the match arm, and the
focused cargo check then passed before commit `4418f506b`. This was a local
compile attempt, not a workload attempt. The OCI Release probe build passed
at that exact SHA using the same existing RocksDB archive and shared cache.

All nine probe runs passed without retries. The external runner required every
measured key read to find its full 522-byte encoding and 512-byte payload,
matching full request counts. All final values, advancing per-key revisions,
complete-state Scan, prefix fingerprints, and reopen checks passed. GNU time
captured whole-process CPU, elapsed time, and maximum RSS for each probe run.

## Artifact packaging

The default CSV writer produced CRLF line endings, which Git's whitespace
check reported. The derived CSV writers now explicitly use LF and the
summaries were regenerated without changing any measurements. A full staged
whitespace scan then reported 34 findings, all in original OCI tool output:
trailing command spaces, test-progress spaces, or terminal blank lines.
Those raw logs remain byte-for-byte intact. The source/documentation/derived
artifact whitespace check excludes original `oci/` output and passes. The
relative SHA256SUMS manifest covers both raw and derived files. No Git or
Clippy suppression setting was added.
