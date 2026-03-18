use crate::backend::BackendRegistry;
use crate::config::{self, MountScope};
use crate::error::MntctlError;
use crate::output::color;
use crate::systemd::SystemdManager;
use anyhow::Result;

pub fn run(name: &str, system: bool, registry: &BackendRegistry) -> Result<()> {
    let config = config::find_mount_config_in_scope(
        name,
        if system {
            Some(MountScope::System)
        } else {
            None
        },
    )?;
    let scope = if system {
        MountScope::System
    } else {
        config.scope()
    };
    let backend = registry.get_or_err(config.backend_type())?;

    // Encrypted backends require password_file or password_cmd for systemd.
    if backend.is_encrypted()
        && config.option_str("password_file").is_none()
        && config.option_str("password_cmd").is_none()
    {
        return Err(MntctlError::ConfigError(
            "encrypted backends require 'password_file' or 'password_cmd' in config for systemd persistence".to_string(),
        )
        .into());
    }

    let unit = backend.generate_systemd_unit(&config)?;
    let unit_name = unit.name.clone();

    SystemdManager::install_unit(&unit, scope)?;
    SystemdManager::daemon_reload(scope)?;
    SystemdManager::enable(&unit_name, scope)?;

    println!(
        "{} Enabled '{}' ({})",
        color::success("✓"),
        color::name_style(name),
        unit_name,
    );

    Ok(())
}
