use crate::config::{self, MountScope};
use crate::output::color;
use anyhow::{Context, Result};

pub fn run(name: &str, system: bool) -> Result<()> {
    let config = config::find_mount_config(name)?;
    let scope = if system {
        MountScope::System
    } else {
        config.scope()
    };
    let path = config::mount_config_path(name, scope)?;

    if !path.exists() {
        anyhow::bail!("config file not found: {}", path.display());
    }

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    println!(
        "  Opening {} in {}",
        color::name_style(&path.display().to_string()),
        editor,
    );

    let status = std::process::Command::new(&editor)
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
