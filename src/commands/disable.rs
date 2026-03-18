use crate::backend;
use crate::config::{self, MountScope};
use crate::output::color;
use crate::systemd::SystemdManager;
use anyhow::Result;

pub fn run(name: &str, system: bool) -> Result<()> {
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

    let unit_name = backend::unit_name_for_config(&config)?;

    SystemdManager::disable(&unit_name, scope)?;
    SystemdManager::daemon_reload(scope)?;

    println!(
        "{} Disabled '{}'",
        color::success("✓"),
        color::name_style(name),
    );

    Ok(())
}
