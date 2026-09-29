set -e
R=/tmp/planned-blink-results/control-scenario0
mkdir -p $R
cd /tmp/dodb-zfs-experiment
for rep in 0 1 2; do
  seed=$((979000000 + rep))
  TMPDIR=/bench/zfs/db /tmp/dodb-zfs-exp-target/release/phase0-bench --engine planned-blink --suite write --writers 16 --widths 1 --distributions uniform --duration 5s --warmup 2s --repetitions 1 --cache-capacity 256 --working-set 100000 --key-size 16 --value-size 64 --group-limit 64 --group-bytes 4194304 --queue-capacity 256 --collection-delay 0us --transaction-mode unconditional --tokio-workers 2 --sync-mode real --seed $seed --output $R/00-control-rep$((rep+1))-planned-blink-w16-width1-uniform.jsonl > $R/00-control-rep$((rep+1)).log 2>&1
done
fio --name=zfs-sync-4k --directory=/bench/zfs/db --rw=randwrite --bs=4k --size=256M --ioengine=sync --iodepth=1 --fsync=1 --time_based --runtime=15 > /tmp/planned-blink-results/fio-sync-after.txt 2>&1
rm -f /bench/zfs/db/zfs-sync-4k*
date -u +%FT%TZ > $R/done
