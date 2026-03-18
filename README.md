# mntctl

Modular remote & encrypted mount manager with systemd integration.

`mntctl` wraps multiple mount backends (sshfs, rclone, NFS, SMB, gocryptfs, cryfs, encfs) behind a consistent `start/stop/enable/disable` interface inspired by `systemctl`. It manages per-mount TOML configs and generates systemd units for persistent mounts.

## Features

- **Unified CLI** for all mount types with systemctl-style lifecycle commands
- **TOML configuration** per mount in `~/.config/mntctl/mounts/` (user) or `/etc/mntctl/mounts/` (system)
- **systemd integration** — generate, install, enable/disable persistent mount units
- **Backend extensibility** — adding a new backend is one file + one registration line
- **No runtime dependencies** — single static binary, no async runtime
- **Privilege model** — FUSE backends run unprivileged; `--system` flag uses pkexec for system-level mounts

## Installation

### From source

```bash
cargo install --path .
```

### From release binaries

Pre-built binaries will be available on the [Releases](https://github.com/sfoerster/mntctl/releases) page once the first version is tagged. The release workflow builds for `x86_64` and `aarch64` automatically on `v*` tags.

## Quick start

```bash
# Add a mount configuration
mntctl add bastion -t sshfs -s user@host:/remote/path -T ~/mnt/bastion

# Mount it
mntctl start bastion

# Check status
mntctl status bastion

# Unmount
mntctl stop bastion

# Make it persistent (creates a systemd user service)
mntctl enable bastion

# Check systemd unit
systemctl --user status mntctl-bastion.service

# Disable persistent mount
mntctl disable bastion

# Remove configuration and unit
mntctl remove bastion
```

## CLI reference

```
mntctl add <name> -t <backend> -s <source> -T <target> [-o key=val,...]
mntctl remove <name> [--force]
mntctl start <name>          # mount now (transient)
mntctl stop <name>           # unmount now
mntctl enable <name>         # install + enable systemd unit (persistent)
mntctl disable <name>        # disable systemd unit
mntctl restart <name>        # stop + start
mntctl status [name]         # detailed info or overview
mntctl list                  # table of all mounts with status
mntctl edit <name>           # open TOML in $EDITOR
mntctl completion <shell>    # generate shell completions (bash, zsh, fish)
mntctl doctor                # check system dependencies
```

Global flags:
- `--system` — operate on system-level mounts (uses pkexec)
- `-v, --verbose` — enable debug logging

## Configuration

Each mount is a TOML file in `~/.config/mntctl/mounts/<name>.toml`:

```toml
[mount]
name = "bastion-e2a"
type = "sshfs"
source = "admin@bastion.example.com:/opt/data"
target = "~/mnt/bastion-e2a"
scope = "user"

[options]
cache = "yes"
reconnect = true
ServerAliveInterval = 15
```

See [docs/configuration.md](docs/configuration.md) for full details.

## Backends

| Backend   | Type   | Unmount        | systemd Unit             | Default Scope | Status      |
|-----------|--------|----------------|--------------------------|---------------|-------------|
| sshfs     | FUSE   | fusermount -u  | .service (Type=simple)   | user          | implemented |
| rclone    | FUSE   | fusermount -u  | .service (Type=notify)   | user          | planned     |
| nfs       | kernel | umount         | .mount + .automount      | system        | planned     |
| smb       | kernel | umount         | .mount + .automount      | system        | planned     |
| gocryptfs | FUSE   | fusermount -u  | .service + passfile      | user          | planned     |
| cryfs     | FUSE   | fusermount -u  | .service + passfile      | user          | planned     |
| encfs     | FUSE   | fusermount -u  | .service + extpass       | user          | planned     |

See [docs/backends.md](docs/backends.md) for backend-specific details.

## Shell completions

```bash
# Bash
mntctl completion bash > ~/.local/share/bash-completion/completions/mntctl

# Zsh
mntctl completion zsh > ~/.local/share/zsh/site-functions/_mntctl

# Fish
mntctl completion fish > ~/.config/fish/completions/mntctl.fish
```

## Development

```bash
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details.

## Architecture

See [docs/architecture.md](docs/architecture.md) for design details.

## License

Apache 2.0 — see [LICENSE](LICENSE).
