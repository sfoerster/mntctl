use crate::backend::BackendRegistry;
use crate::config;
use crate::config::MountScope;
use crate::output::color;
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
    let backend = registry.get_or_err(config.backend_type())?;

    // Idempotent: not mounted = info + success.
    if !backend.is_mounted(&config)? {
        println!(
            "{} '{}' is not mounted",
            color::info("ℹ"),
            color::name_style(name),
        );
        return Ok(());
    }

    backend.unmount(&config)?;

    println!(
        "{} Unmounted '{}'",
        color::success("✓"),
        color::name_style(name),
    );

    Ok(())
}
