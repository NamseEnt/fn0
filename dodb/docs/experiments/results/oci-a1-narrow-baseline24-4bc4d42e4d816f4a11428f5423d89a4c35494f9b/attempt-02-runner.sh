#!/usr/bin/env bash
set -u
set -o pipefail
root=/bench/zfs/db/oci-a1-narrow-baseline24-4bc4d42e4d816f4a11428f5423d89a4c35494f9b
printf 'run_id\tstarted_utc\tended_utc\texit_status\n' > "$root/attempt-02-execution-status.tsv"
while IFS=$'\t' read -r attempt run_id correction condition engine external_repetition jsonl_repetition seed pair_order total_client_workers width command; do
  [ "$attempt" = attempt ] && continue
  start=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  printf '%s START %s\n' "$run_id" "$start" | tee -a "$root/logs/attempt-02-runner.log"
  bash -c "$command" > "$root/logs/$run_id.log" 2>&1
  status=$?
  end=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  printf '%s\t%s\t%s\t%s\n' "$run_id" "$start" "$end" "$status" >> "$root/attempt-02-execution-status.tsv"
  printf '%s END %s exit=%s\n' "$run_id" "$end" "$status" | tee -a "$root/logs/attempt-02-runner.log"
done < "$root/attempt-02-manifest.tsv"
