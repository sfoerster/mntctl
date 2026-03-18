use crate::backend::{
    check_binaries, fuse_is_mounted, fuse_unmount, run_command_for_scope, Backend, MountContext,
};
use crate::config::{BackendType, MountConfig};
use crate::error::MntctlError;
use crate::systemd::unit::{render_exec_command, SystemdUnit};
use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct RcloneBackend;

impl Backend for RcloneBackend {
    fn name(&self) -> &str {
        "rclone"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Rclone
    }

    fn mount(&self, config: &MountConfig, _ctx: &MountContext) -> Result<()> {
        check_binaries(&self.required_binaries())?;

        let target = config.resolved_target()?;
        if !target.exists() {
            std::fs::create_dir_all(&target).with_context(|| {
                format!("failed to create target directory: {}", target.display())
            })?;
        }

        let mut args = vec![
            "mount".to_string(),
            config.source().to_string(),
            target.to_string_lossy().to_string(),
        ];

        // Apply options.
        for (k, v) in &config.options {
            let val = match v {
                toml::Value::String(s) => s.clone(),
                toml::Value::Boolean(b) => {
                    if *b {
                        String::new()
                    } else {
                        continue;
                    }
                }
                toml::Value::Integer(i) => i.to_string(),
                other => other.to_string(),
            };

            if val.is_empty() {
                args.push(format!("--{k}"));
            } else {
                args.push(format!("--{k}={val}"));
            }
        }

        let output = run_command_for_scope("rclone", &args, Some(config.scope()))
            .context("failed to execute rclone")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(MntctlError::MountError(format!("rclone mount failed: {}", stderr.trim())).into())
        }
    }

    fn unmount(&self, config: &MountConfig) -> Result<()> {
        let target = config.resolved_target()?;
        fuse_unmount(&target, Some(config.scope()))
    }

    fn is_mounted(&self, config: &MountConfig) -> Result<bool> {
        let target = config.resolved_target()?;
        fuse_is_mounted(&target, "rclone")
    }

    fn validate_config(&self, config: &MountConfig) -> Result<()> {
        if config.source().is_empty() {
            return Err(MntctlError::ConfigError("source cannot be empty".to_string()).into());
        }
        if !config.source().contains(':') {
            return Err(MntctlError::ConfigError(
                "rclone source must be in the format remote:path".to_string(),
            )
            .into());
        }
        if config.target().is_empty() {
            return Err(MntctlError::ConfigError("target cannot be empty".to_string()).into());
        }
        Ok(())
    }

    fn generate_systemd_unit(&self, config: &MountConfig) -> Result<SystemdUnit> {
        self.validate_config(config)?;
        let target = config.resolved_target()?;

        let mut exec_args = vec![
            "mount".to_string(),
            config.source().to_string(),
            target.to_string_lossy().to_string(),
        ];

        for (k, v) in &config.options {
            let val = match v {
                toml::Value::String(s) => s.clone(),
                toml::Value::Boolean(b) => {
                    if *b {
                        String::new()
                    } else {
                        continue;
                    }
                }
                toml::Value::Integer(i) => i.to_string(),
                other => other.to_string(),
            };

            if val.is_empty() {
                exec_args.push(format!("--{k}"));
            } else {
                exec_args.push(format!("--{k}={val}"));
            }
        }

        let exec_start = render_exec_command("/usr/bin/rclone", &exec_args);
        let exec_stop = render_exec_command(
            "/usr/bin/fusermount",
            &["-u".to_string(), target.display().to_string()],
        );

        Ok(SystemdUnit::service(
            &format!("mntctl-{}", config.name()),
            &format!("mntctl mount: {} (rclone)", config.name()),
            &exec_start,
            &exec_stop,
            "notify",
        ))
    }

    fn required_binaries(&self) -> Vec<&str> {
        vec!["rclone", "fusermount"]
    }

    fn default_options(&self) -> HashMap<&str, &str> {
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mount::MountSection;
    use crate::config::{MountConfig, MountScope};
    use std::collections::BTreeMap;

    fn sample_config() -> MountConfig {
        let mut options = BTreeMap::new();
        options.insert(
            "vfs-cache-mode".to_string(),
            toml::Value::String("full".to_string()),
        );
        options.insert("allow-other".to_string(), toml::Value::Boolean(true));

        MountConfig {
            mount: MountSection {
                name: "test-rclone".to_string(),
                backend_type: BackendType::Rclone,
                source: "gdrive:documents".to_string(),
                target: "/tmp/mntctl-rclone-test".to_string(),
                scope: MountScope::User,
            },
            options,
        }
    }

    #[test]
    fn validate_valid_config() {
        let backend = RcloneBackend;
        assert!(backend.validate_config(&sample_config()).is_ok());
    }

    #[test]
    fn validate_empty_source() {
        let backend = RcloneBackend;
        let mut config = sample_config();
        config.mount.source = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn validate_missing_colon() {
        let backend = RcloneBackend;
        let mut config = sample_config();
        config.mount.source = "gdrive-no-colon".to_string();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn generate_systemd_unit() {
        let backend = RcloneBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("[Service]"));
        assert!(rendered.contains("Type=notify"));
        assert!(rendered.contains("rclone mount"));
        assert!(rendered.contains("gdrive:documents"));
        assert!(rendered.contains("--vfs-cache-mode=full"));
        assert!(rendered.contains("--allow-other"));
        assert!(rendered.contains("fusermount -u"));
    }

    #[test]
    fn unit_name_is_service() {
        let backend = RcloneBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        assert_eq!(unit.name, "mntctl-test-rclone.service");
    }
}
