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

## Phase 4 — More Backends (complete)

- rclone backend (FUSE, Type=notify)
- nfs backend (kernel, .mount units, system-scope warning)
- smb backend (kernel, credentials file support)
- path_to_systemd_unit_name() for kernel mount unit naming (systemd-escape style)
- unit_name_for_config() helper — FUSE uses .service, kernel uses .mount
- Fixed all commands to use dynamic unit name resolution
- Tests: unit generation per backend, path encoding, validation
- Docs: example configs per backend

## Phase 5 — Encrypted Backends (complete)

- [x] gocryptfs backend (-fg foreground, -passfile for systemd)
- [x] cryfs backend (CRYFS_FRONTEND=noninteractive, --passphrase-file)
- [x] encfs backend (--extpass for external password command)
- [x] Interactive passphrase prompting via rpassword for `mntctl start`
- [x] Validation: `mntctl enable` requires password_file or password_cmd in config
- [x] Tests: passphrase handling, unit generation
- [x] Docs: encrypted backend guide / examples

## Phase 6 — Groups & Batch Operations (complete)

- [x] `groups` field on mount config (optional, backward-compatible)
- [x] `mntctl add -g group1,group2` to assign groups
- [x] `mntctl start/stop/restart --all` to operate on all mounts
- [x] `mntctl start/stop/restart --group <name>` to operate on a group
- [x] `mntctl list --group <name>` to filter by group
- [x] Batch error handling: continue on failure, report summary

## Phase 7 — Polish

- [ ] Man page generation (clap_mangen or hidden subcommand)
- [x] Release workflow: multi-arch binaries, checksums, GitHub Release
- [ ] AUR package / deb packaging
- [x] ~~`mntctl doctor` subcommand: check binary availability, config validity, systemd health~~
- [ ] `mntctl log <name>`: tail journalctl for a mount's systemd unit
- [ ] Colored diff output for `mntctl edit` validation errors
- [ ] Config migration tooling (import from fstab entries)

## Ideas (not yet planned)

- Automount-on-access via systemd .automount for FUSE backends
- Health checks: periodic connectivity verification with automatic restart
- Desktop notifications on mount failure (via notify-send or D-Bus)
- Bash/zsh prompt integration: show active mounts in PS1
- NetworkManager dispatcher integration: auto-mount when specific networks connect
