# sshfs examples

## Basic mount

```bash
mntctl add my-server -t sshfs -s user@server.example.com:/home/user -T ~/mnt/server
mntctl start my-server
```

Config created at `~/.config/mntctl/mounts/my-server.toml`:

```toml
[mount]
name = "my-server"
type = "sshfs"
source = "user@server.example.com:/home/user"
target = "~/mnt/server"
scope = "user"
```

## Mount with options

```bash
mntctl add my-server -t sshfs \
  -s user@server.example.com:/home/user \
  -T ~/mnt/server \
  -o reconnect,cache=yes,ServerAliveInterval=15
```

## Persistent mount via systemd

```bash
mntctl add my-server -t sshfs -s user@server.example.com:/data -T ~/mnt/data
mntctl enable my-server
```

This creates `~/.config/systemd/user/mntctl-my-server.service`:

```ini
[Unit]
Description=mntctl mount: my-server (sshfs)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/sshfs user@server.example.com:/data /home/user/mnt/data -f
ExecStop=/usr/bin/fusermount -u /home/user/mnt/data
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
```

## Mount with custom sftp-server

Useful when you need to run as a different user on the remote host:

```bash
mntctl add bastion -t sshfs \
  -s admin@bastion.example.com:/opt/app \
  -T ~/mnt/bastion \
  -o "sftp_server=/usr/bin/sudo -u appuser /usr/libexec/openssh/sftp-server"

The generated unit will quote that option as a single `ExecStart=` argument so systemd does not split the embedded spaces.
```

## Full lifecycle

```bash
mntctl add demo -t sshfs -s user@host:/path -T ~/mnt/demo
mntctl list                        # shows demo as stopped/disabled
mntctl start demo                  # mounts
mntctl status demo                 # shows active
mntctl stop demo                   # unmounts
mntctl enable demo                 # installs systemd unit
systemctl --user status mntctl-demo.service
mntctl disable demo                # disables unit
mntctl remove demo                 # cleans up config + unit
```
