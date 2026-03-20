use crate::backend::BackendRegistry;
use crate::config::mount::MountSection;
use crate::config::{self, BackendType, MountConfig, MountScope};
use crate::error::MntctlError;
use crate::output::color;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::str::FromStr;

#[allow(clippy::too_many_arguments)]
pub fn run(
    name: &str,
    backend_type: Option<&str>,
    source: &str,
    target: &str,
    options: &[String],
    groups: &[String],
    system: bool,
    registry: &BackendRegistry,
) -> Result<()> {
    let global_config = config::GlobalConfig::load();
    let backend_type = backend_type
        .map(str::to_string)
        .or(global_config.default_backend)
        .ok_or_else(|| {
            MntctlError::ConfigError(
                "backend type is required (pass -t/--type or set default_backend in config.toml)"
                    .to_string(),
            )
        })?;

    let bt = BackendType::from_str(&backend_type)?;
    let scope = if system {
        MountScope::System
    } else {
        MountScope::User
    };

    // Prevent duplicate names across scopes to keep lookups unambiguous.
    if config::mount_config_exists_anywhere(name)? {
        return Err(MntctlError::MountAlreadyExists(name.to_string()).into());
    }
    let path = config::mount_config_path(name, scope)?;

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
            groups: groups.to_vec(),
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
