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

## rclone (implemented)

Mounts rclone remotes via FUSE. Supports any rclone remote (Google Drive, S3, SFTP, etc.).

**Required binaries**: `rclone`, `fusermount`

**Source format**: `remote:path` (as configured in `rclone config`)

**systemd unit**: `.service` with `Type=notify` (rclone supports sd_notify)

**Example config**:

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

Options are passed as `--key=value` flags to `rclone mount`.

See [examples/rclone.md](examples/rclone.md) for more.

---

## nfs (implemented)

Kernel NFS mounts.

**Required binaries**: `mount.nfs`

**Source format**: `host:/export/path`

**systemd unit**: `.mount` with path-encoded unit names (e.g., `/mnt/nfs-data` becomes `mnt-nfs\x2ddata.mount`)

**Default scope**: system (kernel mounts require root; warns if user scope is used)

**Example config**:

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

See [examples/nfs.md](examples/nfs.md) for more.

---

## smb (implemented)

Kernel CIFS/SMB mounts.

**Required binaries**: `mount.cifs`

**Source format**: `//server/share`

**systemd unit**: `.mount` with path-encoded unit names

**Default scope**: system (kernel mounts require root; warns if user scope is used)

Supports `credentials` option pointing to a credentials file with 0600 permissions.

**Example config**:

```toml
[mount]
name = "office-share"
type = "smb"
source = "//fileserver/share"
target = "/mnt/office"
scope = "system"

[options]
credentials = "/etc/samba/creds"
uid = 1000
gid = 1000
```

See [examples/smb.md](examples/smb.md) for more.

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
