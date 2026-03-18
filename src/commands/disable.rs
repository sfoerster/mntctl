use crate::config::{self, MountScope};
use crate::output::color;
use crate::systemd::SystemdManager;
use anyhow::Result;

pub fn run(name: &str, system: bool) -> Result<()> {
    let config = config::find_mount_config(name)?;
    let scope = if system {
        MountScope::System
    } else {
        config.scope()
    };

    let unit_name = format!("mntctl-{}.service", name);

    SystemdManager::disable(&unit_name, scope)?;
    SystemdManager::daemon_reload(scope)?;

    println!(
        "{} Disabled '{}'",
        color::success("✓"),
        color::name_style(name),
    );

    Ok(())
}
