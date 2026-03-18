# cryfs examples

## Basic mount (interactive passphrase)

```bash
mntctl add secure -t cryfs -s /home/user/.encrypted/secure -T ~/mnt/secure
mntctl start secure  # prompts for passphrase
```

## Mount with password file

```bash
mntctl add secure -t cryfs \
  -s /home/user/.encrypted/secure \
  -T ~/mnt/secure \
  -o password_file=/home/user/.secrets/cryfs.pass
```

## Mount with password command

```bash
mntctl add secure -t cryfs \
  -s /home/user/.encrypted/secure \
  -T ~/mnt/secure \
  -o "password_cmd=pass show cryfs-secure"
```

## Persistent mount via systemd

Requires `password_file` or `password_cmd` (systemd cannot prompt interactively):

```bash
mntctl add secure -t cryfs \
  -s /home/user/.encrypted/secure \
  -T ~/mnt/secure \
  -o password_file=/home/user/.secrets/cryfs.pass
mntctl enable secure
```

This creates `~/.config/systemd/user/mntctl-secure.service`:

```ini
[Unit]
Description=mntctl mount: secure (cryfs)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/cryfs --passphrase-file /home/user/.secrets/cryfs.pass -f /home/user/.encrypted/secure /home/user/mnt/secure
ExecStop=/usr/bin/fusermount -u /home/user/mnt/secure
Restart=on-failure
RestartSec=5
Environment=CRYFS_FRONTEND=noninteractive

[Install]
WantedBy=default.target
```

## Full lifecycle

```bash
mntctl add secure -t cryfs -s ~/.encrypted/secure -T ~/mnt/secure -o password_file=~/.secrets/cryfs.pass
mntctl start secure
mntctl status secure
mntctl stop secure
mntctl enable secure
mntctl disable secure
mntctl remove secure
```
