# rclone examples

## Basic mount

```bash
mntctl add gdrive -t rclone -s gdrive:documents -T ~/mnt/gdrive
mntctl start gdrive
```

## Mount with caching

```bash
mntctl add gdrive -t rclone \
  -s gdrive:documents \
  -T ~/mnt/gdrive \
  -o vfs-cache-mode=full,allow-other
```

Config at `~/.config/mntctl/mounts/gdrive.toml`:

```toml
[mount]
name = "gdrive"
type = "rclone"
source = "gdrive:documents"
target = "~/mnt/gdrive"

[options]
vfs-cache-mode = "full"
allow-other = true
```

## Persistent mount

```bash
mntctl enable gdrive
```

Generates `~/.config/systemd/user/mntctl-gdrive.service` with `Type=notify` (rclone supports sd_notify).

## S3 bucket

```bash
mntctl add s3-data -t rclone -s s3remote:my-bucket/prefix -T ~/mnt/s3
```

## SFTP via rclone

```bash
mntctl add sftp-backup -t rclone -s mysftp:backups -T ~/mnt/sftp-backup
```
