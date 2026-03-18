use crate::config::{self, MountScope};
use crate::output::color;
use anyhow::{Context, Result};

pub fn run(name: &str, system: bool) -> Result<()> {
    let config = config::find_mount_config_in_scope(
        name,
        if system {
            Some(MountScope::System)
        } else {
            None
        },
    )?;
    let scope = if system {
        MountScope::System
    } else {
        config.scope()
    };
    let path = config::mount_config_path(name, scope)?;

    if !path.exists() {
        anyhow::bail!("config file not found: {}", path.display());
    }

    let global_config = config::GlobalConfig::load();
    let editor = global_config
        .editor
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .unwrap_or_else(|| "vi".to_string());

    println!(
        "  Opening {} in {}",
        color::name_style(&path.display().to_string()),
        editor,
    );

    let mut cmd =
        if scope == MountScope::System && std::env::var_os("MNTCTL_SYSTEM_CONFIG_DIR").is_none() {
            let mut cmd = std::process::Command::new("pkexec");
            cmd.arg(&editor);
            cmd
        } else {
            std::process::Command::new(&editor)
        };
    let status = cmd
        .arg(&path)
        .status()
        .with_context(|| format!("failed to run editor: {editor}"))?;

    if !status.success() {
        anyhow::bail!("editor exited with non-zero status");
    }

    // Validate the edited config.
    match config::load_mount_config(&path) {
        Ok(_) => println!("{} Configuration valid", color::success("✓")),
        Err(e) => {
            println!("{} Configuration has errors: {e}", color::error("✗"));
            println!("  Run 'mntctl edit {name}' to fix");
        }
    }

    Ok(())
}
