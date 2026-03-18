use crate::backend::BackendRegistry;
use crate::config::mount::MountSection;
use crate::config::{self, BackendType, MountConfig, MountScope};
use crate::error::MntctlError;
use crate::output::color;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::str::FromStr;

pub fn run(
    name: &str,
    backend_type: &str,
    source: &str,
    target: &str,
    options: &[String],
    system: bool,
    registry: &BackendRegistry,
) -> Result<()> {
    let bt = BackendType::from_str(backend_type)?;
    let scope = if system {
        MountScope::System
    } else {
        MountScope::User
    };

    // Check if mount already exists.
    let path = config::mount_config_path(name, scope)?;
    if path.exists() {
        return Err(MntctlError::MountAlreadyExists(name.to_string()).into());
    }

    // Parse options.
    let mut opts = BTreeMap::new();
    for opt in options {
        if let Some((k, v)) = opt.split_once('=') {
            // Try to parse as bool or integer, otherwise keep as string.
            let value = if v == "true" {
                toml::Value::Boolean(true)
            } else if v == "false" {
                toml::Value::Boolean(false)
            } else if let Ok(i) = v.parse::<i64>() {
                toml::Value::Integer(i)
            } else {
                toml::Value::String(v.to_string())
            };
            opts.insert(k.to_string(), value);
        } else {
            // Flag-style option (no value = true).
            opts.insert(opt.clone(), toml::Value::Boolean(true));
        }
    }

    let config = MountConfig {
        mount: MountSection {
            name: name.to_string(),
            backend_type: bt,
            source: source.to_string(),
            target: target.to_string(),
            scope,
        },
        options: opts,
    };

    // Validate config via backend.
    let backend = registry.get_or_err(bt)?;
    backend
        .validate_config(&config)
        .context("configuration validation failed")?;

    // Save config.
    config::save_mount_config(&config, &path)?;

    println!(
        "{} Added mount '{}' ({})",
        color::success("✓"),
        color::name_style(name),
        bt,
    );
    println!("  Config: {}", path.display());

    Ok(())
}
