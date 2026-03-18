# NFS examples

NFS mounts require system scope (`--system` flag) since kernel mounts need root.

## Basic mount

```bash
mntctl add nas-data --system -t nfs -s fileserver:/export/data -T /mnt/nas-data
mntctl start nas-data
```

## Mount with options

```bash
mntctl add nas-data --system -t nfs \
  -s fileserver:/export/data \
  -T /mnt/nas-data \
  -o rw,soft,timeo=30
```

Config at `/etc/mntctl/mounts/nas-data.toml`:

```toml
[mount]
name = "nas-data"
type = "nfs"
source = "fileserver:/export/data"
target = "/mnt/nas-data"
scope = "system"

[options]
rw = true
soft = true
timeo = 30
```

## Persistent mount

```bash
mntctl enable nas-data --system
```

Generates a `.mount` unit at `/etc/systemd/system/mnt-nas\x2ddata.mount` (path-encoded unit name matching systemd conventions):

```ini
[Unit]
Description=mntctl mount: nas-data (nfs)
After=network-online.target
Wants=network-online.target

[Mount]
What=fileserver:/export/data
Where=/mnt/nas-data
Type=nfs
Options=rw,soft,timeo=30

[Install]
WantedBy=default.target
```

## NFSv4 with specific version

```bash
mntctl add nas-v4 --system -t nfs \
  -s fileserver:/export \
  -T /mnt/nas \
  -o nfsvers=4.2,rw
```
