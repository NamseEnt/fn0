# Filesystem setup commands

All filesystems use `/dev/sda4`, mounted at `/bench/btrfs/db`, with benchmark data in `/bench/btrfs/db/fsperf-working`. The partition is unmounted before each `mkfs` or `zpool create` operation. The unchanged boot, root, EFI, and LVM partitions are not formatted.

## ext4

```sh
mkfs.ext4 -F -b 4096 -L dodbbench /dev/sda4
mount -o noatime /dev/sda4 /bench/btrfs/db
```

The ext4 journal remains enabled with the default ordered mode and normal barrier behavior.

## XFS

```sh
mkfs.xfs -f -b size=4096 -L dodbbench /dev/sda4
mount -o noatime /dev/sda4 /bench/btrfs/db
```

## Btrfs

The Btrfs-pre filesystem already existed before this experiment. Its exact original mkfs invocation was not captured. The recorded fstab entry used `defaults,noatime`; its effective mount options included `noatime`, `discard=async`, and `space_cache=v2`. The checked working directory and fio file had no NOCOW `C` flag, and no compression mount option was present.

Btrfs-post was freshly formatted and mounted with:

```sh
mkfs.btrfs -f -s 4096 -L dodbbench /dev/sda4
mount -o noatime /dev/sda4 /bench/btrfs/db
```

Compression is disabled and CoW remains enabled in both observed states. No `chattr +C` or NOCOW setting is applied. Btrfs-post effective options are recorded in `raw/environment-btrfs-post-final.txt`.

## ZFS

```sh
zpool create -f -o ashift=12 dodbbench /dev/sda4
zfs create -o recordsize=4K -o compression=off -o atime=off -o sync=standard -o mountpoint=/bench/btrfs/db dodbbench/db
```

ARC and `primarycache` are left at defaults. No pool or dataset sync weakening is used.

## Temporary fstab handling

The original Btrfs UUID line is removed after the Btrfs-pre run and before formatting. This prevents a stale UUID from making a boot fail during the filesystem matrix. After Btrfs-post, the final Btrfs UUID and mount are recorded in `/etc/fstab` so the benchmark mount remains available after reboot.
