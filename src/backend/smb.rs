use crate::backend::{check_binaries, is_mountpoint, Backend};
use crate::config::{BackendType, MountConfig, MountScope};
use crate::error::MntctlError;
use crate::systemd::unit::SystemdUnit;
use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct SmbBackend;

impl Backend for SmbBackend {
    fn name(&self) -> &str {
        "smb"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Smb
    }

    fn mount(&self, config: &MountConfig) -> Result<()> {
        check_binaries(&self.required_binaries())?;

        let target = config.resolved_target()?;
        if !target.exists() {
            std::fs::create_dir_all(&target).with_context(|| {
                format!("failed to create target directory: {}", target.display())
            })?;
        }

        let mut cmd = std::process::Command::new("mount");
        cmd.arg("-t").arg("cifs");

        // Build mount options.
        let opts = build_mount_options(config);
        if !opts.is_empty() {
            cmd.arg("-o").arg(opts);
        }

        cmd.arg(config.source()).arg(&target);

        let output = cmd.output().context("failed to execute mount")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(MntctlError::MountError(format!("smb mount failed: {}", stderr.trim())).into())
        }
    }

    fn unmount(&self, config: &MountConfig) -> Result<()> {
        let target = config.resolved_target()?;

        let output = std::process::Command::new("umount")
            .arg(&target)
            .output()
            .context("failed to execute umount")?;

        if output.status.success() {
            Ok(())
        } else {
            log::warn!("umount failed, trying lazy unmount");
            let output = std::process::Command::new("umount")
                .arg("-l")
                .arg(&target)
                .output()
                .context("failed to execute umount -l")?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(MntctlError::UnmountError(format!("umount failed: {}", stderr.trim())).into())
            }
        }
    }

    fn is_mounted(&self, config: &MountConfig) -> Result<bool> {
        let target = config.resolved_target()?;
        is_mountpoint(&target)
    }

    fn validate_config(&self, config: &MountConfig) -> Result<()> {
        if config.source().is_empty() {
            return Err(MntctlError::ConfigError("source cannot be empty".to_string()).into());
        }
        if !config.source().starts_with("//") {
            return Err(MntctlError::ConfigError(
                "smb source must be in the format //server/share".to_string(),
            )
            .into());
        }
        if config.target().is_empty() {
            return Err(MntctlError::ConfigError("target cannot be empty".to_string()).into());
        }
        if config.scope() == MountScope::User {
            log::warn!("smb mounts require system scope; use --system flag");
        }
        Ok(())
    }

    fn generate_systemd_unit(&self, config: &MountConfig) -> Result<SystemdUnit> {
        self.validate_config(config)?;
        let target = config.resolved_target()?;
        let target_str = target.to_string_lossy().to_string();

        let unit_name = crate::backend::path_to_systemd_unit_name(&target_str);
        let opts = build_mount_options(config);

        Ok(SystemdUnit::mount_unit(
            &unit_name,
            &format!("mntctl mount: {} (smb)", config.name()),
            config.source(),
            &target_str,
            "cifs",
            &opts,
        ))
    }

    fn required_binaries(&self) -> Vec<&str> {
        vec!["mount.cifs"]
    }

    fn default_options(&self) -> HashMap<&str, &str> {
        HashMap::new()
    }
}

/// Build a comma-separated mount options string from config options.
fn build_mount_options(config: &MountConfig) -> String {
    let mut parts = Vec::new();
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
            parts.push(k.clone());
        } else {
            parts.push(format!("{k}={val}"));
        }
    }
    parts.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mount::MountSection;
    use crate::config::MountConfig;
    use std::collections::BTreeMap;

    fn sample_config() -> MountConfig {
        let mut options = BTreeMap::new();
        options.insert(
            "credentials".to_string(),
            toml::Value::String("/etc/samba/creds".to_string()),
        );
        options.insert("uid".to_string(), toml::Value::Integer(1000));
        options.insert("gid".to_string(), toml::Value::Integer(1000));

        MountConfig {
            mount: MountSection {
                name: "test-smb".to_string(),
                backend_type: BackendType::Smb,
                source: "//fileserver/share".to_string(),
                target: "/mnt/smb-share".to_string(),
                scope: MountScope::System,
            },
            options,
        }
    }

    #[test]
    fn validate_valid_config() {
        let backend = SmbBackend;
        assert!(backend.validate_config(&sample_config()).is_ok());
    }

    #[test]
    fn validate_empty_source() {
        let backend = SmbBackend;
        let mut config = sample_config();
        config.mount.source = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn validate_missing_double_slash() {
        let backend = SmbBackend;
        let mut config = sample_config();
        config.mount.source = "fileserver/share".to_string();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn generate_mount_unit() {
        let backend = SmbBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("[Mount]"));
        assert!(rendered.contains("What=//fileserver/share"));
        assert!(rendered.contains("Where=/mnt/smb-share"));
        assert!(rendered.contains("Type=cifs"));
        assert!(rendered.contains("credentials=/etc/samba/creds"));
        assert!(rendered.contains("uid=1000"));
    }

    #[test]
    fn unit_name_is_mount() {
        let backend = SmbBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        assert!(unit.name.ends_with(".mount"));
        assert_eq!(unit.name, "mnt-smb\\x2dshare.mount");
    }
}
