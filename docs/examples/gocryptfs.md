# gocryptfs examples

## Basic mount (interactive passphrase)

```bash
mntctl add vault -t gocryptfs -s /home/user/.encrypted/vault -T ~/mnt/vault
mntctl start vault  # prompts for passphrase
```

## Mount with password file

```bash
mntctl add vault -t gocryptfs \
  -s /home/user/.encrypted/vault \
  -T ~/mnt/vault \
  -o password_file=/home/user/.secrets/vault.pass
```

## Mount with password command

```bash
mntctl add vault -t gocryptfs \
  -s /home/user/.encrypted/vault \
  -T ~/mnt/vault \
  -o "password_cmd=pass show vault"
```

## Persistent mount via systemd

Requires `password_file` or `password_cmd` (systemd cannot prompt interactively):

```bash
mntctl add vault -t gocryptfs \
  -s /home/user/.encrypted/vault \
  -T ~/mnt/vault \
  -o password_file=/home/user/.secrets/vault.pass
mntctl enable vault
```

This creates `~/.config/systemd/user/mntctl-vault.service`:

```ini
[Unit]
Description=mntctl mount: vault (gocryptfs)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/gocryptfs -passfile /home/user/.secrets/vault.pass -fg /home/user/.encrypted/vault /home/user/mnt/vault
ExecStop=/usr/bin/fusermount -u /home/user/mnt/vault
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

## Full lifecycle

```bash
mntctl add vault -t gocryptfs -s ~/.encrypted/vault -T ~/mnt/vault -o password_file=~/.secrets/vault.pass
mntctl start vault
mntctl status vault
mntctl stop vault
mntctl enable vault
mntctl disable vault
mntctl remove vault
```
