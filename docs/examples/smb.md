# SMB/CIFS examples

SMB mounts require system scope (`--system` flag) since kernel mounts need root.

## Basic mount with credentials file

```bash
mntctl add office-share --system -t smb \
  -s //fileserver/share \
  -T /mnt/office \
  -o credentials=/etc/samba/creds
mntctl start office-share
```

The credentials file (`/etc/samba/creds`) should have 0600 permissions:

```
username=myuser
password=mypassword
domain=WORKGROUP
```

## Mount with UID/GID mapping

```bash
mntctl add nas-docs --system -t smb \
  -s //nas/documents \
  -T /mnt/nas-docs \
  -o "credentials=/etc/samba/creds,uid=1000,gid=1000,file_mode=0644,dir_mode=0755"
```

Config at `/etc/mntctl/mounts/nas-docs.toml`:

```toml
[mount]
name = "nas-docs"
type = "smb"
source = "//nas/documents"
target = "/mnt/nas-docs"
scope = "system"

[options]
credentials = "/etc/samba/creds"
uid = 1000
gid = 1000
file_mode = "0644"
dir_mode = "0755"
```

## Persistent mount

```bash
mntctl enable nas-docs --system
```

Generates a `.mount` unit at `/etc/systemd/system/mnt-nas\x2ddocs.mount`:

```ini
[Unit]
Description=mntctl mount: nas-docs (smb)
After=network-online.target
Wants=network-online.target

[Mount]
What=//nas/documents
Where=/mnt/nas-docs
Type=cifs
Options=credentials=/etc/samba/creds,uid=1000,gid=1000,file_mode=0644,dir_mode=0755

[Install]
WantedBy=default.target
```

## Guest access

```bash
mntctl add public --system -t smb \
  -s //server/public \
  -T /mnt/public \
  -o guest,uid=1000
```
