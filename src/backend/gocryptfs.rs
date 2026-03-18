use crate::backend::{
    check_binaries, fuse_is_mounted, fuse_unmount, Backend, MountContext,
    ENCRYPTED_RESERVED_OPTIONS,
};
use crate::config::{BackendType, MountConfig};
use crate::error::MntctlError;
use crate::systemd::unit::SystemdUnit;
use anyhow::{Context, Result};
use std::io::Write;
use std::process::Stdio;

pub struct GocryptfsBackend;

impl Backend for GocryptfsBackend {
    fn name(&self) -> &str {
        "gocryptfs"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Gocryptfs
    }

    fn mount(&self, config: &MountConfig, ctx: &MountContext) -> Result<()> {
        check_binaries(&self.required_binaries())?;

        let target = config.resolved_target()?;
        if !target.exists() {
            std::fs::create_dir_all(&target).with_context(|| {
                format!("failed to create target directory: {}", target.display())
            })?;
        }

        let mut cmd = std::process::Command::new("gocryptfs");

        if let Some(password_file) = config.option_str("password_file") {
            cmd.arg("-passfile").arg(password_file);
        } else if let Some(password_cmd) = config.option_str("password_cmd") {
            cmd.arg("-extpass").arg(password_cmd);
        } else {
            // Read passphrase from stdin.
            cmd.arg("-stdin");
        }

        // Pass through non-reserved options.
        for (k, v) in &config.options {
            if ENCRYPTED_RESERVED_OPTIONS.contains(&k.as_str()) {
                continue;
            }
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
                cmd.arg(format!("-{k}"));
            } else {
                cmd.arg(format!("-{k}")).arg(val);
            }
        }

        cmd.arg(config.source()).arg(&target);

        if ctx.passphrase.is_some() {
            cmd.stdin(Stdio::piped());
        }

        let mut child = cmd.spawn().context("failed to execute gocryptfs")?;

        if let Some(ref passphrase) = ctx.passphrase {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(passphrase.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(MntctlError::MountError(format!("gocryptfs failed: {}", stderr.trim())).into())
        }
    }

    fn unmount(&self, config: &MountConfig) -> Result<()> {
        let target = config.resolved_target()?;
        fuse_unmount(&target)
    }

    fn is_mounted(&self, config: &MountConfig) -> Result<bool> {
        let target = config.resolved_target()?;
        fuse_is_mounted(&target, "gocryptfs")
    }

    fn validate_config(&self, config: &MountConfig) -> Result<()> {
        if config.source().is_empty() {
            return Err(MntctlError::ConfigError("source cannot be empty".to_string()).into());
        }
        if config.target().is_empty() {
            return Err(MntctlError::ConfigError("target cannot be empty".to_string()).into());
        }
        Ok(())
    }

    fn generate_systemd_unit(&self, config: &MountConfig) -> Result<SystemdUnit> {
        self.validate_config(config)?;
        let target = config.resolved_target()?;

        let mut exec_args = Vec::new();

        if let Some(password_file) = config.option_str("password_file") {
            exec_args.push("-passfile".to_string());
            exec_args.push(password_file);
        } else if let Some(password_cmd) = config.option_str("password_cmd") {
            exec_args.push("-extpass".to_string());
            exec_args.push(password_cmd);
        }

        exec_args.push("-fg".to_string()); // foreground for systemd

        for (k, v) in &config.options {
            if ENCRYPTED_RESERVED_OPTIONS.contains(&k.as_str()) {
                continue;
            }
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
                exec_args.push(format!("-{k}"));
            } else {
                exec_args.push(format!("-{k}"));
                exec_args.push(val);
            }
        }

        exec_args.push(config.source().to_string());
        exec_args.push(target.to_string_lossy().to_string());

        let exec_start = format!("/usr/bin/gocryptfs {}", exec_args.join(" "));
        let exec_stop = format!("/usr/bin/fusermount -u {}", target.display());

        Ok(SystemdUnit::service(
            &format!("mntctl-{}", config.name()),
            &format!("mntctl mount: {} (gocryptfs)", config.name()),
            &exec_start,
            &exec_stop,
            "simple",
        ))
    }

    fn required_binaries(&self) -> Vec<&str> {
        vec!["gocryptfs", "fusermount"]
    }

    fn is_encrypted(&self) -> bool {
        true
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
            "password_file".to_string(),
            toml::Value::String("/home/user/.secrets/vault.pass".to_string()),
        );

        MountConfig {
            mount: MountSection {
                name: "test-gocryptfs".to_string(),
                backend_type: BackendType::Gocryptfs,
                source: "/home/user/.encrypted/vault".to_string(),
                target: "/tmp/mntctl-gocryptfs-test".to_string(),
                scope: MountScope::User,
            },
            options,
        }
    }

    #[test]
    fn validate_valid_config() {
        let backend = GocryptfsBackend;
        assert!(backend.validate_config(&sample_config()).is_ok());
    }

    #[test]
    fn validate_empty_source() {
        let backend = GocryptfsBackend;
        let mut config = sample_config();
        config.mount.source = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn validate_empty_target() {
        let backend = GocryptfsBackend;
        let mut config = sample_config();
        config.mount.target = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn generate_systemd_unit() {
        let backend = GocryptfsBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("[Service]"));
        assert!(rendered.contains("gocryptfs"));
        assert!(rendered.contains("-fg"));
        assert!(rendered.contains("-passfile"));
        assert!(rendered.contains("/home/user/.secrets/vault.pass"));
        assert!(rendered.contains("fusermount -u"));
    }

    #[test]
    fn generate_unit_with_password_cmd() {
        let backend = GocryptfsBackend;
        let mut config = sample_config();
        config.options.clear();
        config.options.insert(
            "password_cmd".to_string(),
            toml::Value::String("pass show vault".to_string()),
        );
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("-extpass"));
        assert!(rendered.contains("pass show vault"));
    }

    #[test]
    fn unit_name_is_service() {
        let backend = GocryptfsBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        assert!(unit.name.ends_with(".service"));
    }

    #[test]
    fn is_encrypted_returns_true() {
        let backend = GocryptfsBackend;
        assert!(backend.is_encrypted());
    }
}
