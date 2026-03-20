# mntctl

Modular remote & encrypted mount manager with systemd integration.

`mntctl` wraps multiple mount backends (sshfs, rclone, NFS, SMB, gocryptfs, cryfs, encfs) behind a consistent `start/stop/enable/disable` interface inspired by `systemctl`. It manages per-mount TOML configs and generates systemd units for persistent mounts.

## Features

- **Unified CLI** for all mount types with systemctl-style lifecycle commands
- **TOML configuration** per mount in `~/.config/mntctl/mounts/` (user) or `/etc/mntctl/mounts/` (system)
- **systemd integration** — generate, install, enable/disable persistent mount units
- **Backend extensibility** — adding a new backend is one file + one registration line
- **No runtime dependencies** — single static binary, no async runtime
- **Privilege model** — FUSE backends run unprivileged by default; `--system` targets system-scope configs and uses pkexec for system-level file/unit/command operations

## Installation

### From source

```bash
make && sudo make install
```

Or via Cargo:

```bash
cargo install --path .
```

To uninstall:

```bash
sudo make uninstall
```

### From release binaries

Download the latest binary from [Releases](https://github.com/sfoerster/mntctl/releases) for your architecture.

## Quick start

```bash
# Add a mount configuration
mntctl add bastion -t sshfs -s user@host:/remote/path -T ~/mnt/bastion

# Add with group tags
mntctl add work-data -t sshfs -s user@work:/data -T ~/mnt/work -g work,daily

# Mount it
mntctl start bastion

# Mount all filesystems in a group
mntctl start --group work

# Check status
mntctl status bastion

# Unmount
mntctl stop bastion

# Unmount all mounted filesystems
mntctl stop --all

# Unmount a group
mntctl stop --group work

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
mntctl add <name> [-t <backend>] -s <source> -T <target> [-o key=val,...] [-g group,...]
mntctl remove <name> [--force]
mntctl start <name>          # mount now (transient)
mntctl start --all           # mount all configured filesystems
mntctl start --group <name>  # mount all in a group
mntctl stop <name>           # unmount now
mntctl stop --all            # unmount all mounted filesystems
mntctl stop --group <name>   # unmount all in a group
mntctl enable <name>         # install + enable systemd unit (persistent)
mntctl disable <name>        # disable systemd unit
mntctl restart <name>        # stop + start
mntctl restart --all         # restart all configured filesystems
mntctl restart --group <name> # restart all in a group
mntctl status [name]         # detailed info or overview
mntctl list [-g <group>]     # table of all mounts (optionally filtered by group)
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
groups = ["work", "daily"]   # optional group tags

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
| rclone    | FUSE   | fusermount -u  | .service (Type=notify)   | user          | implemented |
| nfs       | kernel | umount         | .mount                   | system        | implemented |
| smb       | kernel | umount         | .mount                   | system        | implemented |
| gocryptfs | FUSE   | fusermount -u  | .service + passfile/extpass | user       | implemented |
| cryfs     | FUSE   | fusermount -u  | .service + shell wrapper | user          | implemented |
| encfs     | FUSE   | fusermount -u  | .service + extpass       | user          | implemented |

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
