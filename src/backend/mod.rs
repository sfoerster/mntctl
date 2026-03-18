pub mod cryfs;
pub mod encfs;
pub mod gocryptfs;
pub mod nfs;
pub mod rclone;
pub mod smb;
pub mod sshfs;

use crate::config::{BackendType, MountConfig};
use crate::systemd::unit::SystemdUnit;
use anyhow::Result;
use std::collections::HashMap;
use std::process::{Command, Output};

/// Options reserved by encrypted backends (excluded from generic pass-through).
pub const ENCRYPTED_RESERVED_OPTIONS: &[&str] = &["password_file", "password_cmd"];

/// Context passed to mount operations (e.g., an interactively prompted passphrase).
#[derive(Default)]
pub struct MountContext {
    pub passphrase: Option<String>,
}

/// Trait that all mount backends must implement.
#[allow(dead_code)]
pub trait Backend: Send + Sync {
    /// Human-readable backend name.
    fn name(&self) -> &str;

    /// The backend type enum variant.
    fn backend_type(&self) -> BackendType;

    /// Mount the filesystem.
    fn mount(&self, config: &MountConfig, ctx: &MountContext) -> Result<()>;

    /// Unmount the filesystem.
    fn unmount(&self, config: &MountConfig) -> Result<()>;

    /// Check if the filesystem is currently mounted.
    fn is_mounted(&self, config: &MountConfig) -> Result<bool>;

    /// Validate the mount configuration for this backend.
    fn validate_config(&self, config: &MountConfig) -> Result<()>;

    /// Generate the systemd unit file content for this mount.
    fn generate_systemd_unit(&self, config: &MountConfig) -> Result<SystemdUnit>;

    /// List of binary names that must be present on the system.
    fn required_binaries(&self) -> Vec<&str>;

    /// Whether this backend manages encrypted filesystems.
    fn is_encrypted(&self) -> bool {
        false
    }

    /// Default options for this backend.
    fn default_options(&self) -> HashMap<&str, &str> {
        HashMap::new()
    }
}

/// Registry of available backends.
pub struct BackendRegistry {
    backends: HashMap<BackendType, Box<dyn Backend>>,
}

impl BackendRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            backends: HashMap::new(),
        };
        registry.register(Box::new(sshfs::SshfsBackend));
        registry.register(Box::new(rclone::RcloneBackend));
        registry.register(Box::new(nfs::NfsBackend));
        registry.register(Box::new(smb::SmbBackend));
        registry.register(Box::new(gocryptfs::GocryptfsBackend));
        registry.register(Box::new(cryfs::CryfsBackend));
        registry.register(Box::new(encfs::EncfsBackend));
        registry
    }

    fn register(&mut self, backend: Box<dyn Backend>) {
        self.backends.insert(backend.backend_type(), backend);
    }

    pub fn get(&self, backend_type: BackendType) -> Option<&dyn Backend> {
        self.backends.get(&backend_type).map(|b| b.as_ref())
    }

    pub fn get_or_err(&self, backend_type: BackendType) -> Result<&dyn Backend> {
        self.get(backend_type)
            .ok_or_else(|| anyhow::anyhow!("backend '{}' not yet implemented", backend_type))
    }
}

/// Check if a target path is currently a mountpoint by reading /proc/mounts.
pub fn is_mountpoint(target: &std::path::Path) -> Result<bool> {
    let mounts = std::fs::read_to_string("/proc/mounts")?;
    let target_str = target.to_string_lossy();
    Ok(mounts.lines().any(|line| {
        line.split_whitespace()
            .nth(1)
            .is_some_and(|mp| mp == target_str.as_ref())
    }))
}

/// Check if a target is mounted with a specific FUSE filesystem type.
pub fn fuse_is_mounted(target: &std::path::Path, fuse_type: &str) -> Result<bool> {
    let mounts = std::fs::read_to_string("/proc/mounts")?;
    let target_str = target.to_string_lossy();
    let type_str = format!("fuse.{fuse_type}");
    Ok(mounts.lines().any(|line| {
        let parts: Vec<&str> = line.split_whitespace().collect();
        parts.len() >= 3 && parts[1] == target_str.as_ref() && parts[2] == type_str
    }))
}

/// Unmount a FUSE filesystem using fusermount, with lazy fallback.
pub fn fuse_unmount(
    target: &std::path::Path,
    scope: Option<crate::config::MountScope>,
) -> Result<()> {
    let output = run_command_for_scope(
        "fusermount",
        &["-u".to_string(), target.to_string_lossy().to_string()],
        scope,
    )?;

    if output.status.success() {
        return Ok(());
    }

    // Lazy unmount fallback.
    log::warn!("fusermount -u failed, trying lazy unmount");
    let output = run_command_for_scope(
        "fusermount",
        &["-uz".to_string(), target.to_string_lossy().to_string()],
        scope,
    )?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(crate::error::MntctlError::UnmountError(format!(
            "fusermount failed: {}",
            stderr.trim()
        ))
        .into())
    }
}

/// Run a command, optionally via pkexec for system-scope operations.
pub fn run_command_for_scope(
    program: &str,
    args: &[String],
    scope: Option<crate::config::MountScope>,
) -> Result<Output> {
    let mut cmd = build_scoped_command(program, args, scope);
    Ok(cmd.output()?)
}

/// Build a command for the given scope.
pub fn build_scoped_command(
    program: &str,
    args: &[String],
    scope: Option<crate::config::MountScope>,
) -> Command {
    match scope {
        Some(crate::config::MountScope::System) => {
            let mut cmd = Command::new("pkexec");
            cmd.arg(program);
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        }
        _ => {
            let mut cmd = Command::new(program);
            for arg in args {
                cmd.arg(arg);
            }
            cmd
        }
    }
}

/// Check that all required binaries for a backend are available on $PATH.
pub fn check_binaries(binaries: &[&str]) -> Result<()> {
    for bin in binaries {
        which::which(bin)
            .map_err(|_| crate::error::MntctlError::BinaryNotFound((*bin).to_string()))?;
    }
    Ok(())
}

/// Convert an absolute mount path to a systemd unit name.
///
/// Mirrors `systemd-escape --path`: strips leading `/`, replaces `/` with `-`,
/// and escapes special characters with `\xNN`.
pub fn path_to_systemd_unit_name(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return "-".to_string();
    }

    let mut result = String::new();
    for (i, component) in trimmed.split('/').enumerate() {
        if i > 0 {
            result.push('-');
        }
        for ch in component.chars() {
            match ch {
                // Characters that need escaping in systemd unit names.
                '-' => result.push_str("\\x2d"),
                '\\' => result.push_str("\\x5c"),
                ' ' => result.push_str("\\x20"),
                _ if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' => {
                    result.push(ch);
                }
                _ => {
                    // Escape as \xNN for each UTF-8 byte.
                    for byte in ch.to_string().as_bytes() {
                        result.push_str(&format!("\\x{:02x}", byte));
                    }
                }
            }
        }
    }
    result
}

/// Determine the systemd unit name for a given mount config.
///
/// FUSE backends use `mntctl-<name>.service`.
/// Kernel backends (nfs, smb) use path-encoded `.mount` names.
pub fn unit_name_for_config(config: &MountConfig) -> Result<String> {
    match config.backend_type() {
        BackendType::Nfs | BackendType::Smb => {
            let target = config.resolved_target()?;
            let target_str = target.to_string_lossy().to_string();
            let unit_base = path_to_systemd_unit_name(&target_str);
            Ok(format!("{unit_base}.mount"))
        }
        _ => Ok(format!("mntctl-{}.service", config.name())),
    }
}

/// Build a MountContext, prompting for a passphrase if needed.
pub fn build_mount_context(backend: &dyn Backend, config: &MountConfig) -> Result<MountContext> {
    if backend.is_encrypted()
        && config.option_str("password_file").is_none()
        && config.option_str("password_cmd").is_none()
    {
        let passphrase =
            rpassword::prompt_password(format!("Passphrase for '{}': ", config.name()))?;
        Ok(MountContext {
            passphrase: Some(passphrase),
        })
    } else {
        Ok(MountContext::default())
    }
}

/// Quote a command argument for `/bin/sh -lc`.
pub fn shell_quote(arg: &str) -> String {
    let mut quoted = String::from("'");
    for ch in arg.chars() {
        if ch == '\'' {
            quoted.push_str("'\"'\"'");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('\'');
    quoted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_to_unit_simple() {
        assert_eq!(path_to_systemd_unit_name("/mnt/data"), "mnt-data");
    }

    #[test]
    fn path_to_unit_with_dash() {
        assert_eq!(
            path_to_systemd_unit_name("/mnt/nfs-data"),
            "mnt-nfs\\x2ddata"
        );
    }

    #[test]
    fn path_to_unit_deep_path() {
        assert_eq!(
            path_to_systemd_unit_name("/mnt/remote/server/share"),
            "mnt-remote-server-share"
        );
    }

    #[test]
    fn path_to_unit_trailing_slash() {
        assert_eq!(path_to_systemd_unit_name("/mnt/data/"), "mnt-data");
    }

    #[test]
    fn path_to_unit_root() {
        assert_eq!(path_to_systemd_unit_name("/"), "-");
    }
}
