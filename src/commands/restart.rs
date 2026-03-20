use crate::backend::{build_mount_context, BackendRegistry};
use crate::config::{self, MountConfig, MountScope};
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

    if backend.is_mounted(&config)? {
        backend.unmount(&config)?;
        println!("  Unmounted '{}'", name);
    }

    let ctx = build_mount_context(backend, &config)?;
    backend.mount(&config, &ctx)?;

    println!(
        "{} Restarted '{}' at {}",
        color::success("✓"),
        color::name_style(name),
        config.target(),
    );

    Ok(())
}

pub fn run_all(system: bool, registry: &BackendRegistry) -> Result<()> {
    let configs = if system {
        config::list_mount_configs(MountScope::System)?
    } else {
        config::list_all_mount_configs()?
    };
    run_batch(&configs, registry)
}

pub fn run_group(group: &str, system: bool, registry: &BackendRegistry) -> Result<()> {
    let configs = config::list_mount_configs_by_group(group, system)?;
    if configs.is_empty() {
        println!("No mounts in group '{group}'.");
        return Ok(());
    }
    run_batch(&configs, registry)
}

fn run_batch(configs: &[MountConfig], registry: &BackendRegistry) -> Result<()> {
    if configs.is_empty() {
        println!("No mounts configured.");
        return Ok(());
    }

    let mut restarted = 0u32;
    let mut errors = Vec::new();

    for cfg in configs {
        let backend = match registry.get(cfg.backend_type()) {
            Some(b) => b,
            None => continue,
        };

        if let Ok(true) = backend.is_mounted(cfg) {
            if let Err(e) = backend.unmount(cfg) {
                eprintln!(
                    "{} Failed to unmount '{}': {e}",
                    color::error("✗"),
                    color::name_style(cfg.name()),
                );
                errors.push(cfg.name().to_string());
                continue;
            }
        }

        let ctx = match build_mount_context(backend, cfg) {
            Ok(ctx) => ctx,
            Err(e) => {
                eprintln!(
                    "{} Failed to prepare '{}': {e}",
                    color::error("✗"),
                    color::name_style(cfg.name()),
                );
                errors.push(cfg.name().to_string());
                continue;
            }
        };

        match backend.mount(cfg, &ctx) {
            Ok(()) => {
                println!(
                    "{} Restarted '{}' at {}",
                    color::success("✓"),
                    color::name_style(cfg.name()),
                    cfg.target(),
                );
                restarted += 1;
            }
            Err(e) => {
                eprintln!(
                    "{} Failed to mount '{}': {e}",
                    color::error("✗"),
                    color::name_style(cfg.name()),
                );
                errors.push(cfg.name().to_string());
            }
        }
    }

    if restarted == 0 && errors.is_empty() {
        println!("No mounts configured.");
    } else if !errors.is_empty() {
        return Err(anyhow::anyhow!("failed to restart: {}", errors.join(", ")));
    }

    Ok(())
}
