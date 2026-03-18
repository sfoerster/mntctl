# encfs examples

## Basic mount (interactive passphrase)

```bash
mntctl add private -t encfs -s /home/user/.encrypted/private -T ~/mnt/private
mntctl start private  # prompts for passphrase
```

## Mount with password file

```bash
mntctl add private -t encfs \
  -s /home/user/.encrypted/private \
  -T ~/mnt/private \
  -o password_file=/home/user/.secrets/encfs.pass
```

## Mount with password command

```bash
mntctl add private -t encfs \
  -s /home/user/.encrypted/private \
  -T ~/mnt/private \
  -o "password_cmd=pass show encfs-private"
```

## Persistent mount via systemd

Requires `password_file` or `password_cmd` (systemd cannot prompt interactively):

```bash
mntctl add private -t encfs \
  -s /home/user/.encrypted/private \
  -T ~/mnt/private \
  -o "password_cmd=pass show encfs-private"
mntctl enable private
```

This creates `~/.config/systemd/user/mntctl-private.service`:

```ini
[Unit]
Description=mntctl mount: private (encfs)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/encfs "--extpass=pass show encfs-private" -f /home/user/.encrypted/private /home/user/mnt/private
ExecStop=/usr/bin/fusermount -u /home/user/mnt/private
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

## Full lifecycle

```bash
mntctl add private -t encfs -s ~/.encrypted/private -T ~/mnt/private -o "password_cmd=pass show encfs-private"
mntctl start private
mntctl status private
mntctl stop private
mntctl enable private
mntctl disable private
mntctl remove private
```
