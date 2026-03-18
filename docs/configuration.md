# Configuration

## Config file locations

| Scope  | Config directory                    |
|--------|-------------------------------------|
| User   | `~/.config/mntctl/mounts/`          |
| System | `/etc/mntctl/mounts/`               |

Each mount is stored as `<name>.toml`. Config files are created with 0600 permissions.

An optional global config can be placed at `~/.config/mntctl/config.toml`.

## Mount config format

```toml
[mount]
name = "my-mount"           # unique identifier
type = "sshfs"              # backend type
source = "user@host:/path"  # backend-specific source
target = "~/mnt/my-mount"   # local mount point (~ is expanded)
scope = "user"              # "user" or "system" (default: "user")

[options]
# Backend-specific key-value options.
# Values can be strings, booleans, or integers.
cache = "yes"
reconnect = true
ServerAliveInterval = 15
```

### Required fields

- `name` — unique mount identifier, used in commands and systemd unit names
- `type` — one of: `sshfs`, `rclone`, `nfs`, `smb`, `gocryptfs`, `cryfs`, `encfs`
- `source` — backend-specific source string
- `target` — local directory to mount to (created automatically if missing)

### Optional fields

- `scope` — `user` (default) or `system`

## Options

The `[options]` section is a flat key-value map. Each backend validates and interprets its own options.

### Value types

- **Strings**: `cache = "yes"` — passed as `key=value`
- **Booleans**: `reconnect = true` — `true` becomes a flag-style option, `false` is omitted
- **Integers**: `ServerAliveInterval = 15` — passed as `key=15`

### sshfs options

| Option                  | Type    | Description                            |
|-------------------------|---------|----------------------------------------|
| `cache`                 | string  | Enable caching (`yes`/`no`)            |
| `kernel_cache`          | bool    | Use kernel cache                       |
| `reconnect`             | bool    | Auto-reconnect on connection loss      |
| `ServerAliveInterval`   | integer | SSH keepalive interval in seconds      |
| `ServerAliveCountMax`   | integer | Max missed keepalives before disconnect |
| `StrictHostKeyChecking` | string  | SSH host key policy                    |
| `sftp_server`           | string  | Remote sftp-server command (passed via `-s`) |

### rclone options

Options are passed as `--key=value` flags to `rclone mount`.

| Option           | Type   | Description                        |
|------------------|--------|------------------------------------|
| `vfs-cache-mode` | string | VFS caching mode (`off`/`minimal`/`writes`/`full`) |
| `allow-other`    | bool   | Allow other users to access the mount |
| `dir-cache-time` | string | Cache directory listings duration   |
| `buffer-size`    | string | In-memory buffer size per file      |

### nfs options

Options are passed as `-o key=value` to `mount -t nfs`.

| Option    | Type    | Description                   |
|-----------|---------|-------------------------------|
| `rw`      | bool    | Read-write mount              |
| `soft`    | bool    | Soft mount (return errors on timeout) |
| `hard`    | bool    | Hard mount (retry indefinitely) |
| `timeo`   | integer | Timeout in deciseconds        |
| `retrans` | integer | Number of retransmissions     |
| `nfsvers` | string  | NFS version (e.g., `4.2`)     |

### smb options

Options are passed as `-o key=value` to `mount -t cifs`.

| Option        | Type    | Description                          |
|---------------|---------|--------------------------------------|
| `credentials` | string  | Path to credentials file (0600 perms) |
| `uid`         | integer | Map files to this UID                |
| `gid`         | integer | Map files to this GID                |
| `file_mode`   | string  | File permission mode (e.g., `0644`)  |
| `dir_mode`    | string  | Directory permission mode            |
| `guest`       | bool    | Guest access (no credentials)        |

## Global config

Optional file at `~/.config/mntctl/config.toml`:

```toml
default_backend = "sshfs"   # default for `mntctl add` when -t is omitted
editor = "nvim"             # preferred editor for `mntctl edit`
```

## Editing configs

Use `mntctl edit <name>` to open a mount config in your `$EDITOR`. After saving, the config is validated and any errors are reported.

You can also edit files directly — they are plain TOML at the paths listed above.
