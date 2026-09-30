# Filesystem Setup

All three stages use `/dev/sda4` mounted at `/bench/btrfs/db`. The benchmark working directory is `/bench/btrfs/db/xfs-zfs-crossover-working`. The prior Btrfs UUID entry was removed from `/etc/fstab` before the first reformat. The final XFS UUID entry was restored after XFS-post.

## XFS-pre and XFS-post

The XFS command lines match the four-filesystem experiment:

```sh
mkfs.xfs -f -b size=4096 -L dodbbench /dev/sda4
mount -o noatime /dev/sda4 /bench/btrfs/db
```

The default XFS log and write-barrier behavior are retained. No durability-weakening options are set. XFS-pre mounted directly after mkfs. XFS-post followed ZFS; the first mount attempt detected residual ZFS signatures after `zpool destroy`, so `wipefs -a /dev/sda4` was run on the partition only, then the same `mkfs.xfs` and mount commands were repeated. The actual commands, output, `xfs_info`, and effective mount options are captured in `raw/setup-XFS-pre.txt` and `raw/setup-XFS-post.txt`.

## ZFS

The ZFS command lines match the four-filesystem experiment:

```sh
zpool create -f -o ashift=12 dodbbench /dev/sda4
zfs create -o recordsize=4K -o compression=off -o atime=off -o sync=standard -o mountpoint=/bench/btrfs/db dodbbench/db
```

ARC and `primarycache` remain at their defaults. The actual pool and dataset properties, mount options, and `zpool status` are captured in `raw/setup-ZFS.txt`.

After ZFS results have been copied out of the stage, the pool is destroyed before formatting XFS-post. The final XFS mount UUID and `noatime` fstab entry are verified and recorded after all runs.
