# Environment and Provenance

## OCI instance and block device

- OCI instance: `instance-20260923-1013`, OCID `ocid1.instance.oc1.ap-osaka-1.anvwsljrrfkd6xqcuiozrid2bkb7uh6ba3p7zkwkip6kcagbwgd5cp7os2jq`.
- Shape: `VM.Standard.A1.Flex`, 2 OCPUs, 12.0 GB RAM, region `ap-osaka-1`.
- The attached boot volume is `ocid1.bootvolume.oc1.ap-osaka-1.abvwsljrxzh3px7kxsz7w4jzqiqaytbke42huk7rvadeu6y3r4mynxlr3aiq`, matching the prior filesystem experiment. OCI instance and boot-volume attachment responses are in `raw/`.
- The guest sees the 200 GiB boot volume as `/dev/sda`. The 153.4 GiB `/dev/sda4` partition has GPT PARTUUID `83f10ca6-016c-4894-85a0-0b67fadf20aa` and GPT label `dodb-zfs`. It was mounted at `/bench/btrfs/db` before the first reformat and was separate from root, `/boot`, EFI, and swap. It was the dedicated dodb benchmark partition; no benchmark process was active at the pre-format safety gate.
- Initial guest command output for `uname -a`, `lscpu`, `nproc`, `free -h`, `lsblk`, `findmnt`, `blkid`, `/etc/fstab`, and the partition table is in `raw/environment-initial.txt`.
- OCI API configuration and private key contents were not copied into this result tree. Only sanitized instance and attachment fields are retained.

## Source and executable

- Source commit: `08e82ec484bd62aeb659471a39f317b86cfb488f`.
- The previous experiment's requested binary SHA-256, `765f6b3b33f7a85dfaebdeb5582eb10c64ad6a3222cf354146fc8fc13fe08a4e`, was not found in the checked existing binary paths. A detached clone of the current experiment branch was checked out at the pinned source commit on the OCI host.
- Build command: `CARGO_TARGET_DIR=/var/oled/xfs-zfs-crossover/target cargo build --release --locked --package dodb-storage --bin phase0-bench`.
- Toolchain: `rustc 1.98.1 (48a229cea 2026-09-01)`, `cargo 1.98.1 (797e8a9bc 2026-08-05)`, AArch64 host.
- New binary SHA-256: `3756eac0557bbe0d87ba3b2ea46ed088df46aa7e8d79fae56f570d8eb52469b6`.
- The build log and checkout status are in `raw/build.log` and `raw/source-provenance.txt`. This binary is used unchanged across all three stages. Its data is reported separately from the earlier four-filesystem result.

## Telemetry

- fio CPU seconds are calculated from fio job user and system CPU percentages multiplied by the reported runtime.
- `/dev/sda` written bytes and busy time are deltas from `/proc/diskstats` before and after each fio or dodb invocation. fio values cover the 20-second fio run. dodb block deltas cover the complete single-repetition invocation, including database setup and the 2-second warmup; dodb engine counters and CPU utilization cover its 5-second measurement interval.
- Raw files and metadata remain on tmpfs during a stage so result writes do not contribute block writes on the measured device. After each stage, the full stage result directory is copied into this experiment directory outside `/dev/sda4`.
