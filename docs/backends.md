# Backends

## sshfs (implemented)

Mounts remote directories over SSH using FUSE.

**Required binaries**: `sshfs`, `fusermount`

**Source format**: `user@host:/remote/path`

**systemd unit**: `.service` with `Type=simple` (sshfs runs in foreground with `-f`)

**Example config**:

```toml
[mount]
name = "dev-server"
type = "sshfs"
source = "deploy@server.example.com:/opt/app"
target = "~/mnt/dev-server"

[options]
reconnect = true
ServerAliveInterval = 15
cache = "yes"
```

**Example with sftp-server override** (e.g., sudo to another user on the remote):

```toml
[mount]
name = "bastion-e2a"
type = "sshfs"
source = "admin@bastion.example.com:/opt/data"
target = "~/mnt/bastion-e2a"

[options]
cache = "yes"
kernel_cache = true
reconnect = true
ServerAliveInterval = 15
sftp_server = "/usr/bin/sudo -u appuser /usr/libexec/openssh/sftp-server"
```

### SSH options

Options matching known SSH config keys (`ServerAliveInterval`, `ServerAliveCountMax`, `StrictHostKeyChecking`) are passed as SSH options. All other options are passed as sshfs `-o` flags.

---

## rclone (planned)

Mounts rclone remotes via FUSE.

**systemd unit**: `.service` with `Type=notify` (rclone supports sd_notify)

---

## nfs (planned)

Kernel NFS mounts.

**systemd unit**: `.mount` + `.automount` with path-encoded unit names

**Default scope**: system (auto-promoted during validation)

---

## smb (planned)

Kernel CIFS/SMB mounts.

**systemd unit**: `.mount` + `.automount` with path-encoded unit names

**Default scope**: system (auto-promoted during validation)

Supports `credentials_file` option for storing username/password separately.

---

## gocryptfs (planned)

Encrypted FUSE filesystem.

**systemd unit**: `.service` with `-passfile` for non-interactive password

**Password handling**:
- `mntctl start`: interactive prompt via terminal
- `mntctl enable`: requires `password_file` in options

---

## cryfs (planned)

Encrypted FUSE filesystem.

**systemd unit**: `.service` with `CRYFS_FRONTEND=noninteractive` and `--passphrase-file`

---

## encfs (planned)

Encrypted FUSE filesystem.

**systemd unit**: `.service` with `--extpass` for external password command
