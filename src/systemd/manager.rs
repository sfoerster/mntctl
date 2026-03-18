use crate::config::MountScope;
use crate::error::MntctlError;
use crate::systemd::unit::SystemdUnit;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Manages systemd unit files and systemctl operations.
#[allow(dead_code)]
pub struct SystemdManager;

#[allow(dead_code)]
impl SystemdManager {
    /// Get the directory where systemd units should be installed.
    pub fn unit_dir(scope: MountScope) -> Result<PathBuf> {
        match scope {
            MountScope::User => {
                let config = dirs::config_dir().context("could not determine config directory")?;
                Ok(config.join("systemd").join("user"))
            }
            MountScope::System => Ok(PathBuf::from("/etc/systemd/system")),
        }
    }

    /// Get the full path for a unit file.
    pub fn unit_path(unit_name: &str, scope: MountScope) -> Result<PathBuf> {
        Ok(Self::unit_dir(scope)?.join(unit_name))
    }

    /// Install a unit file to disk.
    pub fn install_unit(unit: &SystemdUnit, scope: MountScope) -> Result<PathBuf> {
        let dir = Self::unit_dir(scope)?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create unit directory: {}", dir.display()))?;

        let path = dir.join(&unit.name);
        let content = unit.render();

        if scope == MountScope::System {
            // Write via pkexec for system scope.
            let status = Command::new("pkexec")
                .arg("tee")
                .arg(&path)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .spawn()
                .context("failed to run pkexec")?
                .wait_with_output();

            match status {
                Ok(output) if output.status.success() => {}
                _ => {
                    return Err(MntctlError::SystemdError(
                        "failed to install system unit via pkexec".to_string(),
                    )
                    .into());
                }
            }
        } else {
            std::fs::write(&path, content)
                .with_context(|| format!("failed to write unit file: {}", path.display()))?;
        }

        Ok(path)
    }

    /// Remove a unit file from disk.
    pub fn remove_unit(unit_name: &str, scope: MountScope) -> Result<()> {
        let path = Self::unit_path(unit_name, scope)?;
        if path.exists() {
            if scope == MountScope::System {
                let status = Command::new("pkexec")
                    .arg("rm")
                    .arg(&path)
                    .status()
                    .context("failed to run pkexec")?;
                if !status.success() {
                    return Err(MntctlError::SystemdError(
                        "failed to remove system unit via pkexec".to_string(),
                    )
                    .into());
                }
            } else {
                std::fs::remove_file(&path)
                    .with_context(|| format!("failed to remove unit file: {}", path.display()))?;
            }
        }
        Ok(())
    }

    /// Run systemctl daemon-reload.
    pub fn daemon_reload(scope: MountScope) -> Result<()> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.arg("daemon-reload");
        Self::run_systemctl(&mut cmd, "daemon-reload")
    }

    /// Enable a systemd unit.
    pub fn enable(unit_name: &str, scope: MountScope) -> Result<()> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["enable", unit_name]);
        Self::run_systemctl(&mut cmd, "enable")
    }

    /// Disable a systemd unit.
    pub fn disable(unit_name: &str, scope: MountScope) -> Result<()> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["disable", unit_name]);
        Self::run_systemctl(&mut cmd, "disable")
    }

    /// Start a systemd unit.
    pub fn start(unit_name: &str, scope: MountScope) -> Result<()> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["start", unit_name]);
        Self::run_systemctl(&mut cmd, "start")
    }

    /// Stop a systemd unit.
    pub fn stop(unit_name: &str, scope: MountScope) -> Result<()> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["stop", unit_name]);
        Self::run_systemctl(&mut cmd, "stop")
    }

    /// Check if a unit is active.
    pub fn is_active(unit_name: &str, scope: MountScope) -> Result<bool> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["is-active", "--quiet", unit_name]);
        let status = cmd.status().context("failed to run systemctl")?;
        Ok(status.success())
    }

    /// Check if a unit is enabled.
    pub fn is_enabled(unit_name: &str, scope: MountScope) -> Result<bool> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["is-enabled", "--quiet", unit_name]);
        let status = cmd.status().context("failed to run systemctl")?;
        Ok(status.success())
    }

    /// Get the status output for a unit.
    pub fn status_output(unit_name: &str, scope: MountScope) -> Result<String> {
        let mut cmd = Self::systemctl_cmd(scope);
        cmd.args(["status", "--no-pager", unit_name]);
        let output = cmd.output().context("failed to run systemctl")?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Build the base systemctl command with scope flags.
    fn systemctl_cmd(scope: MountScope) -> Command {
        match scope {
            MountScope::User => {
                let mut cmd = Command::new("systemctl");
                cmd.arg("--user");
                cmd
            }
            MountScope::System => {
                let mut cmd = Command::new("pkexec");
                cmd.arg("systemctl");
                cmd
            }
        }
    }

    fn run_systemctl(cmd: &mut Command, operation: &str) -> Result<()> {
        let output = cmd.output().context("failed to run systemctl")?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(MntctlError::SystemdError(format!(
                "systemctl {operation} failed: {}",
                stderr.trim()
            ))
            .into())
        }
    }
}
