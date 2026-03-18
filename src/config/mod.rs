pub mod global;
pub mod mount;

#[allow(unused_imports)]
pub use global::GlobalConfig;
pub use mount::{BackendType, MountConfig, MountScope};

use crate::error::MntctlError;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Return the mounts config directory for the given scope.
pub fn mounts_dir(scope: MountScope) -> Result<PathBuf> {
    match scope {
        MountScope::User => {
            let config_dir =
                dirs::config_dir().context("could not determine user config directory")?;
            Ok(config_dir.join("mntctl").join("mounts"))
        }
        MountScope::System => Ok(PathBuf::from("/etc/mntctl/mounts")),
    }
}

/// Return the config file path for a mount.
pub fn mount_config_path(name: &str, scope: MountScope) -> Result<PathBuf> {
    Ok(mounts_dir(scope)?.join(format!("{name}.toml")))
}

/// Expand `~` at the start of a path to the user's home directory.
pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/") {
        let home = dirs::home_dir().context("could not determine home directory")?;
        Ok(home.join(rest))
    } else if path == "~" {
        dirs::home_dir().context("could not determine home directory")
    } else {
        Ok(PathBuf::from(path))
    }
}

/// Load a single mount config from a TOML file.
pub fn load_mount_config(path: &Path) -> Result<MountConfig> {
    let contents = std::fs::read_to_string(path).map_err(|e| MntctlError::ConfigReadError {
        path: path.to_path_buf(),
        source: e,
    })?;
    let config: MountConfig =
        toml::from_str(&contents).map_err(|e| MntctlError::ConfigParseError {
            path: path.to_path_buf(),
            source: e,
        })?;
    Ok(config)
}

/// Save a mount config to a TOML file with 0600 permissions.
pub fn save_mount_config(config: &MountConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config directory: {}", parent.display()))?;
    }

    let contents = toml::to_string_pretty(config).context("failed to serialize mount config")?;

    std::fs::write(path, &contents).map_err(|e| MntctlError::ConfigWriteError {
        path: path.to_path_buf(),
        source: e,
    })?;

    // Set permissions to 0600 (owner read/write only).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)
            .with_context(|| format!("failed to set permissions on {}", path.display()))?;
    }

    Ok(())
}

/// List all mount configs for a given scope.
pub fn list_mount_configs(scope: MountScope) -> Result<Vec<MountConfig>> {
    let dir = mounts_dir(scope)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut configs = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("failed to read config directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            match load_mount_config(&path) {
                Ok(config) => configs.push(config),
                Err(e) => {
                    log::warn!("skipping invalid config {}: {e}", path.display());
                }
            }
        }
    }

    configs.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(configs)
}

/// List all mount configs across both scopes.
pub fn list_all_mount_configs() -> Result<Vec<MountConfig>> {
    let mut configs = list_mount_configs(MountScope::User)?;
    match list_mount_configs(MountScope::System) {
        Ok(system_configs) => configs.extend(system_configs),
        Err(e) => {
            log::debug!("could not read system configs: {e}");
        }
    }
    configs.sort_by(|a, b| a.name().cmp(b.name()));
    Ok(configs)
}

/// Find a mount config by name, searching user scope first then system.
pub fn find_mount_config(name: &str) -> Result<MountConfig> {
    for scope in [MountScope::User, MountScope::System] {
        let path = mount_config_path(name, scope)?;
        if path.exists() {
            return load_mount_config(&path);
        }
    }
    Err(MntctlError::MountNotFound(name.to_string()).into())
}

/// Delete a mount config file.
pub fn delete_mount_config(name: &str, scope: MountScope) -> Result<()> {
    let path = mount_config_path(name, scope)?;
    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("failed to remove config file: {}", path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_works() {
        let expanded = expand_tilde("~/test/path").unwrap();
        assert!(expanded.is_absolute());
        assert!(expanded.ends_with("test/path"));
    }

    #[test]
    fn expand_tilde_absolute_passthrough() {
        let expanded = expand_tilde("/absolute/path").unwrap();
        assert_eq!(expanded, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn expand_tilde_bare() {
        let expanded = expand_tilde("~").unwrap();
        assert!(expanded.is_absolute());
    }
}
