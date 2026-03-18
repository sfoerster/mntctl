use crate::backend::BackendRegistry;
use crate::config::{self, MountScope};
use crate::output::color;
use crate::systemd::SystemdManager;
use anyhow::Result;

pub fn run(name: &str, system: bool, registry: &BackendRegistry) -> Result<()> {
    let config = config::find_mount_config(name)?;
    let scope = if system {
        MountScope::System
    } else {
        config.scope()
    };
    let backend = registry.get_or_err(config.backend_type())?;

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
