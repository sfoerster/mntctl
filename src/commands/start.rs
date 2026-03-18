use crate::backend::{build_mount_context, BackendRegistry};
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

    // Idempotent: already mounted = info + success.
    if backend.is_mounted(&config)? {
        println!(
            "{} '{}' is already mounted at {}",
            color::info("ℹ"),
            color::name_style(name),
            config.target(),
        );
        return Ok(());
    }

    let ctx = build_mount_context(backend, &config)?;
    backend.mount(&config, &ctx)?;

    println!(
        "{} Mounted '{}' at {}",
        color::success("✓"),
        color::name_style(name),
        config.target(),
    );

    Ok(())
}
