# Roadmap

## Phase 1 — Foundation (complete)

- Project scaffolding: Cargo.toml, LICENSE, README, CONTRIBUTING, .gitignore
- GitHub CI: ci.yml (fmt, clippy, test, build)
- Core types: error.rs, config/mount.rs, config/global.rs, config/mod.rs
- Backend trait + empty registry: backend/mod.rs
- systemd skeleton: systemd/unit.rs (SystemdUnit + render), systemd/manager.rs
- CLI definition: cli.rs (all subcommands defined)
- Output: output/table.rs, output/color.rs
- Commands: list, status
- Entry point: main.rs
- Tests: config parsing, CLI help/version, empty list

## Phase 2 — First Backend (sshfs) + Core Commands (complete)

- sshfs backend: mount, unmount, is_mounted, validate, unit generation
- Commands: add, start, stop, remove
- systemd/manager.rs: systemctl wrapper implementation
- Shared helpers: expand_tilde, check_binaries, fuse_unmount, fuse_is_mounted, is_mountpoint
- Tests: sshfs unit generation, validation, option merging

## Phase 3 — Persistence (complete)

- Commands: enable, disable, restart, edit, completion, doctor
- systemd unit install/remove logic
- Polkit policy file
- Tests: CLI integration tests for all commands

## Phase 4 — More Backends

- [ ] rclone backend (FUSE, Type=notify, sd_notify support)
- [ ] nfs backend (kernel, .mount + .automount units, auto-system-scope promotion)
- [ ] smb backend (kernel, credentials file support)
- [ ] Extract fuse_service_unit shared helper for common FUSE service unit template
- [ ] path_to_systemd_unit_name() for kernel mount unit naming (systemd-escape style)
- [ ] Tests: unit generation per backend, path encoding
- [ ] Docs: example configs per backend

## Phase 5 — Encrypted Backends

- [ ] gocryptfs backend (-fg foreground, -passfile for systemd)
- [ ] cryfs backend (CRYFS_FRONTEND=noninteractive, --passphrase-file)
- [ ] encfs backend (--extpass for external password command)
- [ ] Interactive passphrase prompting via rpassword for `mntctl start`
- [ ] Validation: `mntctl enable` requires password_file or password_cmd in config
- [ ] Tests: passphrase handling, unit generation
- [ ] Docs: encrypted backend guide

## Phase 6 — Polish

- [ ] Man page generation (clap_mangen or hidden subcommand)
- [ ] Release workflow: multi-arch binaries, checksums, GitHub Release
- [ ] AUR package / deb packaging
- [x] ~~`mntctl doctor` subcommand: check binary availability, config validity, systemd health~~
- [ ] `mntctl log <name>`: tail journalctl for a mount's systemd unit
- [ ] Colored diff output for `mntctl edit` validation errors
- [ ] Config migration tooling (import from fstab entries)

## Ideas (not yet planned)

- Automount-on-access via systemd .automount for FUSE backends
- Mount groups: start/stop multiple related mounts together
- Health checks: periodic connectivity verification with automatic restart
- Desktop notifications on mount failure (via notify-send or D-Bus)
- Bash/zsh prompt integration: show active mounts in PS1
- NetworkManager dispatcher integration: auto-mount when specific networks connect
