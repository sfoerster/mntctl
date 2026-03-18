use crate::backend::BackendRegistry;
use crate::config::{self, MountConfig};
use crate::output::color;
use crate::systemd::SystemdManager;
use anyhow::Result;

pub fn run(name: Option<&str>, system: bool, registry: &BackendRegistry) -> Result<()> {
    match name {
        Some(name) => show_single(name, registry),
        None => show_overview(system, registry),
    }
}

fn show_single(name: &str, registry: &BackendRegistry) -> Result<()> {
    let config = config::find_mount_config(name)?;

    println!(
        "{}",
        color::label_style(&format!("Mount: {}", color::name_style(name)))
    );
    println!("  Type:   {}", config.backend_type());
    println!("  Source: {}", config.source());
    println!("  Target: {}", config.target());
    println!("  Scope:  {}", config.scope());

    // Mount status.
    let mount_status = match registry.get(config.backend_type()) {
        Some(backend) => match backend.is_mounted(&config) {
            Ok(true) => color::status_style("mounted"),
            Ok(false) => color::status_style("unmounted"),
            Err(e) => color::status_style(&format!("error: {e}")),
        },
        None => "unknown (backend not loaded)".to_string(),
    };
    println!("  Status: {}", mount_status);

    // systemd unit status.
    let unit_name = format!("mntctl-{}.service", name);
    let enabled = SystemdManager::is_enabled(&unit_name, config.scope()).unwrap_or(false);
    let active = SystemdManager::is_active(&unit_name, config.scope()).unwrap_or(false);
    println!(
        "  Unit:   {} ({})",
        unit_name,
        if enabled {
            color::status_style("enabled")
        } else {
            color::status_style("disabled")
        },
    );
    if enabled || active {
        println!(
            "  Active: {}",
            if active {
                color::status_style("active")
            } else {
                color::status_style("inactive")
            }
        );
    }

    // Options.
    if !config.options.is_empty() {
        println!("  Options:");
        for (k, v) in &config.options {
            println!("    {k} = {v}");
        }
    }

    Ok(())
}

fn show_overview(system: bool, registry: &BackendRegistry) -> Result<()> {
    let configs = if system {
        config::list_mount_configs(config::MountScope::System)?
    } else {
        config::list_all_mount_configs()?
    };

    if configs.is_empty() {
        println!("No mounts configured.");
        return Ok(());
    }

    for config in &configs {
        print_summary_line(config, registry);
    }

    Ok(())
}

fn print_summary_line(config: &MountConfig, registry: &BackendRegistry) {
    let mounted = match registry.get(config.backend_type()) {
        Some(backend) => match backend.is_mounted(config) {
            Ok(true) => color::status_style("mounted"),
            Ok(false) => color::status_style("unmounted"),
            Err(_) => color::status_style("error"),
        },
        None => "?".to_string(),
    };

    println!(
        "  {} [{}] {} -> {} ({})",
        color::name_style(config.name()),
        config.backend_type(),
        config.source(),
        config.target(),
        mounted,
    );
}
