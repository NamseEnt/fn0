# Environment and provenance

## OCI instance

- OCI query at the end of the matrix reported instance `instance-20260923-1013`, lifecycle `RUNNING`, shape `VM.Standard.A1.Flex`, 2 OCPUs, and 12.0 GB RAM in `ap-osaka-1`, availability domain `IsRG:AP-OSAKA-1-AD-1`.
- The boot-volume attachment query reported the same attached boot volume used by the earlier A1 benchmark: `ocid1.bootvolume.oc1.ap-osaka-1.abvwsljrxzh3px7kxsz7w4jzqiqaytbke42huk7rvadeu6y3r4mynxlr3aiq`.
- The guest host name was `instance-20260923-1013`; architecture was AArch64, CPU model Ampere Altra Neoverse-N1, with 2 online logical CPUs. Guest kernel was `6.12.0-206.104.4.4.el9uek.aarch64`.
- `/dev/sda` was the 200 GiB boot-volume device. `/dev/sda4` was the same 153.40 GiB GPT partition in every stage. The EFI, `/boot`, LVM root, `/var/oled`, and swap devices are separate. Partition table and per-stage `lsblk`, `findmnt`, `blkid`, and `/etc/fstab` captures are in `raw/environment-*.txt` and `raw/setup-*.txt`.
- At the start of Btrfs-pre, `/dev/sda4` was mounted at `/bench/btrfs/db` and labeled `dodbbench`. The raw environment capture shows its then-current UUID and GPT partition UUID.

## Source and binary

- Source commit: `08e82ec484bd62aeb659471a39f317b86cfb488f`.
- Engine: `parallel-blink`.
- Release binary SHA-256: `765f6b3b33f7a85dfaebdeb5582eb10c64ad6a3222cf354146fc8fc13fe08a4e`.
- That release binary had already been built from the pinned commit during the earlier same-day OCI diagnostic. Its SHA-256 was verified on the host and copied to `/home/opc/phase0-bench-filesystem-matrix`; the identical executable was used for all five stages. No filesystem-specific rebuild occurred.
- The dodb checkout used for raw `git_commit` reporting was a clean detached checkout at the same commit. It was copied outside the benchmark partition before the first reformat so the raw provenance remained available through all stages.
- Rust version was `1.98.1`; fio version was `3.35`.

## Filesystem and telemetry notes

- ext4 used 4 KiB blocks, an enabled journal, default ordered data mode and default barrier behavior, with `noatime`.
- XFS used 4 KiB blocks and `noatime`.
- Btrfs used 4 KiB sectorsize for the freshly formatted post stage, no compression, and default CoW. `lsattr` showed no `C` flag on the working directory or fio file. No `chattr +C` or NOCOW option was used.
- ZFS used `ashift=12`, `recordsize=4K`, `compression=off`, `atime=off`, and `sync=standard`. `primarycache=all` remained the default.
- fio CPU time is derived from fio's per-job user and system CPU percentages over its reported runtime. dodb CPU seconds are derived from its measured-window one-core CPU utilization and `duration_ms`.
- `/dev/sda` write bytes and busy time use before/after deltas from `/proc/diskstats` fields for sectors written and milliseconds doing I/O. fio deltas cover the 20-second fio process. dodb deltas cover the full one-repetition invocation, including database seeding and the 2-second warmup; dodb's engine metrics themselves cover the 5-second measurement window.
- The guest was otherwise idle during measurements; no dodb or fio process was running between stages.

## Btrfs pre/post interpretation

Btrfs-pre was the existing Btrfs filesystem before the requested sequence of reformats. Its original `mkfs.btrfs` invocation predates this experiment and was not available in the captured state. The live state was verified as Btrfs with `noatime`, no compression mount option, default CoW, and no NOCOW inode flag. Btrfs-post was freshly formatted with the command in `filesystem-setup.md`. Therefore the pre/post comparison measures elapsed-time drift together with the difference between the existing and newly formatted Btrfs state; it is not a pure OCI-time-only drift estimate.

The original Btrfs UUID line was removed from `/etc/fstab` before changing filesystem type. The final Btrfs UUID and mount entry were restored after Btrfs-post. The final live mount and fstab line are recorded in `raw/environment-btrfs-post-final.txt`.
