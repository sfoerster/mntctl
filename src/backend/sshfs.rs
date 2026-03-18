use crate::backend::{check_binaries, fuse_is_mounted, fuse_unmount, Backend};
use crate::config::{BackendType, MountConfig};
use crate::error::MntctlError;
use crate::systemd::unit::SystemdUnit;
use anyhow::{Context, Result};
use std::collections::HashMap;

pub struct SshfsBackend;

impl Backend for SshfsBackend {
    fn name(&self) -> &str {
        "sshfs"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Sshfs
    }

    fn mount(&self, config: &MountConfig) -> Result<()> {
        check_binaries(&self.required_binaries())?;

        let target = config.resolved_target()?;
        if !target.exists() {
            std::fs::create_dir_all(&target).with_context(|| {
                format!("failed to create target directory: {}", target.display())
            })?;
        }

        let mut cmd = std::process::Command::new("sshfs");
        cmd.arg(config.source()).arg(&target);

        // Build sshfs options (merge defaults with user config).
        let mut opts: HashMap<String, String> = self
            .default_options()
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        for (k, v) in &config.options {
            let val = match v {
                toml::Value::String(s) => s.clone(),
                toml::Value::Boolean(b) => {
                    if *b {
                        String::new() // flag-style option
                    } else {
                        continue; // skip false booleans
                    }
                }
                toml::Value::Integer(i) => i.to_string(),
                other => other.to_string(),
            };
            opts.insert(k.clone(), val);
        }

        // SSH-specific options go into -o ssh_option=value.
        let ssh_opts = [
            "ServerAliveInterval",
            "ServerAliveCountMax",
            "StrictHostKeyChecking",
        ];
        let mut ssh_opt_parts = Vec::new();
        let mut sshfs_opt_parts = Vec::new();

        for (k, v) in &opts {
            if ssh_opts.contains(&k.as_str()) {
                if v.is_empty() {
                    ssh_opt_parts.push(k.clone());
                } else {
                    ssh_opt_parts.push(format!("{k}={v}"));
                }
            } else if k == "sftp_server" {
                cmd.arg("-s").arg(v);
            } else if v.is_empty() {
                sshfs_opt_parts.push(k.clone());
            } else {
                sshfs_opt_parts.push(format!("{k}={v}"));
            }
        }

        if !sshfs_opt_parts.is_empty() {
            cmd.arg("-o").arg(sshfs_opt_parts.join(","));
        }
        if !ssh_opt_parts.is_empty() {
            cmd.arg("-o").arg(ssh_opt_parts.join(","));
        }

        // Run in foreground for systemd compatibility (sshfs -f).
        // For transient mounts we do NOT pass -f so it daemonizes.
        let output = cmd.output().context("failed to execute sshfs")?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(MntctlError::MountError(format!("sshfs failed: {}", stderr.trim())).into())
        }
    }

    fn unmount(&self, config: &MountConfig) -> Result<()> {
        let target = config.resolved_target()?;
        fuse_unmount(&target)
    }

    fn is_mounted(&self, config: &MountConfig) -> Result<bool> {
        let target = config.resolved_target()?;
        fuse_is_mounted(&target, "sshfs")
    }

    fn validate_config(&self, config: &MountConfig) -> Result<()> {
        if config.source().is_empty() {
            return Err(MntctlError::ConfigError("source cannot be empty".to_string()).into());
        }
        if !config.source().contains(':') {
            return Err(MntctlError::ConfigError(
                "sshfs source must be in the format user@host:/path".to_string(),
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
            config.source().to_string(),
            target.to_string_lossy().to_string(),
            "-f".to_string(), // foreground for systemd
        ];

        let ssh_opts = [
            "ServerAliveInterval",
            "ServerAliveCountMax",
            "StrictHostKeyChecking",
        ];
        let mut sshfs_opt_parts = Vec::new();
        let mut ssh_opt_parts = Vec::new();

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

            if ssh_opts.contains(&k.as_str()) {
                if val.is_empty() {
                    ssh_opt_parts.push(k.clone());
                } else {
                    ssh_opt_parts.push(format!("{k}={val}"));
                }
            } else if k == "sftp_server" {
                exec_args.push("-s".to_string());
                exec_args.push(val);
            } else if val.is_empty() {
                sshfs_opt_parts.push(k.clone());
            } else {
                sshfs_opt_parts.push(format!("{k}={val}"));
            }
        }

        if !sshfs_opt_parts.is_empty() {
            exec_args.push("-o".to_string());
            exec_args.push(sshfs_opt_parts.join(","));
        }
        if !ssh_opt_parts.is_empty() {
            exec_args.push("-o".to_string());
            exec_args.push(ssh_opt_parts.join(","));
        }

        let exec_start = format!("/usr/bin/sshfs {}", exec_args.join(" "));
        let exec_stop = format!("/usr/bin/fusermount -u {}", target.display());

        Ok(SystemdUnit::service(
            &format!("mntctl-{}", config.name()),
            &format!("mntctl mount: {} (sshfs)", config.name()),
            &exec_start,
            &exec_stop,
            "simple",
        ))
    }

    fn required_binaries(&self) -> Vec<&str> {
        vec!["sshfs", "fusermount"]
    }

    fn default_options(&self) -> HashMap<&str, &str> {
        // Typed as owned strings since callers will need to merge.
        // We return &str references to static strings.
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
        options.insert("cache".to_string(), toml::Value::String("yes".to_string()));
        options.insert("reconnect".to_string(), toml::Value::Boolean(true));
        options.insert("ServerAliveInterval".to_string(), toml::Value::Integer(15));

        MountConfig {
            mount: MountSection {
                name: "test-sshfs".to_string(),
                backend_type: BackendType::Sshfs,
                source: "user@host:/remote/path".to_string(),
                target: "/tmp/mntctl-test".to_string(),
                scope: MountScope::User,
            },
            options,
        }
    }

    #[test]
    fn validate_valid_config() {
        let backend = SshfsBackend;
        let config = sample_config();
        assert!(backend.validate_config(&config).is_ok());
    }

    #[test]
    fn validate_empty_source() {
        let backend = SshfsBackend;
        let mut config = sample_config();
        config.mount.source = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn validate_missing_colon_in_source() {
        let backend = SshfsBackend;
        let mut config = sample_config();
        config.mount.source = "user@host/path".to_string();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn generate_systemd_unit() {
        let backend = SshfsBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("[Unit]"));
        assert!(rendered.contains("[Service]"));
        assert!(rendered.contains("ExecStart="));
        assert!(rendered.contains("sshfs"));
        assert!(rendered.contains("user@host:/remote/path"));
        assert!(rendered.contains("-f")); // foreground flag
        assert!(rendered.contains("cache=yes"));
        assert!(rendered.contains("reconnect"));
        assert!(rendered.contains("ServerAliveInterval=15"));
    }

    #[test]
    fn generate_unit_with_sftp_server() {
        let backend = SshfsBackend;
        let mut config = sample_config();
        config.options.insert(
            "sftp_server".to_string(),
            toml::Value::String(
                "/usr/bin/sudo -u user /usr/libexec/openssh/sftp-server".to_string(),
            ),
        );
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("-s /usr/bin/sudo -u user /usr/libexec/openssh/sftp-server"));
    }
}
