# Architecture

## Overview

mntctl is a synchronous Rust CLI built with clap. It manages mount configurations as TOML files and delegates actual mount/unmount operations to pluggable backends.

```
main.rs → cli.rs (clap parsing)
        → commands/*.rs (one module per subcommand)
        → backend/mod.rs (Backend trait + registry)
        → backend/<type>.rs (per-backend implementation)
        → config/*.rs (TOML load/save/list)
        → systemd/*.rs (unit generation + systemctl wrapper)
        → output/*.rs (table rendering + color)
```

## Backend trait

Every mount backend implements the `Backend` trait:

```rust
pub trait Backend: Send + Sync {
    fn name(&self) -> &str;
    fn backend_type(&self) -> BackendType;
    fn mount(&self, config: &MountConfig, ctx: &MountContext) -> Result<()>;
    fn unmount(&self, config: &MountConfig) -> Result<()>;
    fn is_mounted(&self, config: &MountConfig) -> Result<bool>;
    fn validate_config(&self, config: &MountConfig) -> Result<()>;
    fn generate_systemd_unit(&self, config: &MountConfig) -> Result<SystemdUnit>;
    fn required_binaries(&self) -> Vec<&str>;
    fn default_options(&self) -> HashMap<&str, &str> { HashMap::new() }
}
```

Adding a backend requires:
1. Create `src/backend/<name>.rs` implementing the trait
2. Register it in `BackendRegistry::new()` in `src/backend/mod.rs`

## Backend registry

A simple `HashMap<BackendType, Box<dyn Backend>>` with explicit registration. No macros or inventory magic — everything is greppable.

## systemd integration

- **FUSE backends** generate `.service` units with `Type=simple` (the FUSE process runs in the foreground via `-f` flag). Cleanup uses `fusermount -u`.
- **Kernel backends** (NFS, SMB) generate `.mount` units with path-encoded filenames.
- **User scope** units go to `~/.config/systemd/user/`
- **System scope** units go to `/etc/systemd/system/` (written via pkexec)

## Shared FUSE helpers

Five of seven backends use FUSE. Common logic is extracted into `backend/mod.rs`:

- `fuse_unmount(target)` — fusermount with lazy fallback
- `fuse_is_mounted(target, fuse_type)` — checks `/proc/mounts` for `fuse.<type>`
- `check_binaries(bins)` — verifies required binaries are on `$PATH`
- `is_mountpoint(target)` — generic `/proc/mounts` check

## Privilege model

- Default operations are user-level (no root needed for FUSE)
- `--system` flag targets system-scope configs and wraps system file/unit/command operations with pkexec
- NFS/SMB backends require system scope and warn if configured as user mounts
- A polkit policy file (`polkit/org.mntctl.policy`) authorizes pkexec

## Error handling

- `MntctlError` enum (thiserror) for typed, domain-specific errors
- `anyhow::Context` at call sites for rich error chains
- Exit codes: 0 = success, 1 = general error, 2 = config error, 3 = systemd error
- Never panics, never unwraps

## Design invariants

- **Idempotent**: `start` on already-mounted = info message + success; `stop` on unmounted = info + success
- **Config permissions**: 0600 for mount config files (may contain credentials)
- **No mocking mounts in CI**: unit generation tested via string assertions, actual mount/unmount is manual only
- **Graceful dependency checks**: `mntctl doctor` verifies systemd, /proc/mounts, and all backend binaries before you need them
