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

    let mut mounted = 0u32;
    let mut errors = Vec::new();

    for cfg in configs {
        let backend = match registry.get(cfg.backend_type()) {
            Some(b) => b,
            None => continue,
        };

        match backend.is_mounted(cfg) {
            Ok(true) => continue,
            Err(_) => continue,
            Ok(false) => {}
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
                    "{} Mounted '{}' at {}",
                    color::success("✓"),
                    color::name_style(cfg.name()),
                    cfg.target(),
                );
                mounted += 1;
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

    if mounted == 0 && errors.is_empty() {
        println!("All filesystems already mounted.");
    } else if !errors.is_empty() {
        return Err(anyhow::anyhow!("failed to mount: {}", errors.join(", ")));
    }

    Ok(())
}
