#!/usr/bin/env bash
set -u
set -o pipefail
root=/bench/zfs/db/oci-a1-narrow-baseline24-4bc4d42e4d816f4a11428f5423d89a4c35494f9b
mkdir -p "$root"/{db,tmp,runs,logs}
printf 'run_id\tstarted_utc\tended_utc\texit_status\n' > "$root/execution-status.tsv"
while IFS=$'\t' read -r run_id condition engine external_repetition jsonl_repetition seed pair_order total_client_workers writers readers width duration command; do
  [ "$run_id" = run_id ] && continue
  start=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  printf '%s START %s\n' "$run_id" "$start" | tee -a "$root/logs/runner.log"
  bash -c "$command" > "$root/logs/$run_id.log" 2>&1
  status=$?
  end=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  printf '%s\t%s\t%s\t%s\n' "$run_id" "$start" "$end" "$status" >> "$root/execution-status.tsv"
  printf '%s END %s exit=%s\n' "$run_id" "$end" "$status" | tee -a "$root/logs/runner.log"
done < "$root/manifest.tsv"
