use crate::backend::BackendRegistry;
use crate::config;
use crate::output::color;
use anyhow::Result;

pub fn run(name: &str, registry: &BackendRegistry) -> Result<()> {
    let config = config::find_mount_config(name)?;
    let backend = registry.get_or_err(config.backend_type())?;

    if backend.is_mounted(&config)? {
        backend.unmount(&config)?;
        println!("  Unmounted '{}'", name);
    }

    backend.mount(&config)?;

    println!(
        "{} Restarted '{}' at {}",
        color::success("✓"),
        color::name_style(name),
        config.target(),
    );

    Ok(())
}
