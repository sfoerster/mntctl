pub mod sshfs;

use crate::config::{BackendType, MountConfig};
use crate::systemd::unit::SystemdUnit;
use anyhow::Result;
use std::collections::HashMap;

/// Trait that all mount backends must implement.
#[allow(dead_code)]
pub trait Backend: Send + Sync {
    /// Human-readable backend name.
    fn name(&self) -> &str;

    /// The backend type enum variant.
    fn backend_type(&self) -> BackendType;

    /// Mount the filesystem.
    fn mount(&self, config: &MountConfig) -> Result<()>;

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
#[allow(dead_code)]
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
pub fn fuse_unmount(target: &std::path::Path) -> Result<()> {
    let output = std::process::Command::new("fusermount")
        .arg("-u")
        .arg(target)
        .output()?;

    if output.status.success() {
        return Ok(());
    }

    // Lazy unmount fallback.
    log::warn!("fusermount -u failed, trying lazy unmount");
    let output = std::process::Command::new("fusermount")
        .arg("-uz")
        .arg(target)
        .output()?;

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

/// Check that all required binaries for a backend are available on $PATH.
pub fn check_binaries(binaries: &[&str]) -> Result<()> {
    for bin in binaries {
        which::which(bin)
            .map_err(|_| crate::error::MntctlError::BinaryNotFound((*bin).to_string()))?;
    }
    Ok(())
}
