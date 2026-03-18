use crate::backend::BackendRegistry;
use crate::config;
use crate::output::color;
use anyhow::Result;

pub fn run(name: &str, registry: &BackendRegistry) -> Result<()> {
    let config = config::find_mount_config(name)?;
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

    backend.mount(&config)?;

    println!(
        "{} Mounted '{}' at {}",
        color::success("✓"),
        color::name_style(name),
        config.target(),
    );

    Ok(())
}
