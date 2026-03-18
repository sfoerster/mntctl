use crate::backend::{self, BackendRegistry};
use crate::config::{self, MountScope};
use crate::error::MntctlError;
use crate::output::color;
use crate::systemd::SystemdManager;
use anyhow::Result;

pub fn run(name: &str, force: bool, system: bool, registry: &BackendRegistry) -> Result<()> {
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

    // Check if mounted.
    if let Some(backend) = registry.get(config.backend_type()) {
        if backend.is_mounted(&config)? {
            if force {
                backend.unmount(&config)?;
                println!("  Unmounted '{}'", name);
            } else {
                return Err(MntctlError::AlreadyMounted(format!(
                    "{} (use --force to unmount and remove)",
                    name,
                ))
                .into());
            }
        }
    }

    // Disable and remove systemd unit if it exists.
    let unit_name = backend::unit_name_for_config(&config)?;
    if SystemdManager::is_enabled(&unit_name, scope).unwrap_or(false) {
        SystemdManager::disable(&unit_name, scope)?;
        SystemdManager::daemon_reload(scope)?;
    }
    SystemdManager::remove_unit(&unit_name, scope)?;

    // Delete config file.
    config::delete_mount_config(name, scope)?;

    println!(
        "{} Removed mount '{}'",
        color::success("✓"),
        color::name_style(name),
    );

    Ok(())
}
