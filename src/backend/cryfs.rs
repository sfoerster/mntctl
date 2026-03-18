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

pub struct CryfsBackend;

impl Backend for CryfsBackend {
    fn name(&self) -> &str {
        "cryfs"
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Cryfs
    }

    fn mount(&self, config: &MountConfig, ctx: &MountContext) -> Result<()> {
        check_binaries(&self.required_binaries())?;

        let target = config.resolved_target()?;
        if !target.exists() {
            std::fs::create_dir_all(&target).with_context(|| {
                format!("failed to create target directory: {}", target.display())
            })?;
        }

        let mut cmd = std::process::Command::new("cryfs");

        // Disable interactive prompts.
        cmd.env("CRYFS_FRONTEND", "noninteractive");

        if let Some(password_file) = config.option_str("password_file") {
            cmd.arg("--passphrase-file").arg(password_file);
        } else if let Some(password_cmd) = config.option_str("password_cmd") {
            // CryFS doesn't have an extpass flag; use password_cmd via stdin.
            cmd.env("CRYFS_FRONTEND", "noninteractive");
            // We'll pipe the output of the command via stdin below.
            let pw_output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&password_cmd)
                .output()
                .with_context(|| format!("failed to execute password_cmd: {password_cmd}"))?;
            let passphrase = String::from_utf8_lossy(&pw_output.stdout)
                .trim()
                .to_string();
            cmd.stdin(Stdio::piped());

            // Pass through non-reserved options.
            for (k, v) in &config.options {
                if ENCRYPTED_RESERVED_OPTIONS.contains(&k.as_str()) {
                    continue;
                }
                let val = option_to_string(v);
                if let Some(val) = val {
                    if val.is_empty() {
                        cmd.arg(format!("--{k}"));
                    } else {
                        cmd.arg(format!("--{k}")).arg(val);
                    }
                }
            }

            cmd.arg(config.source()).arg(&target);

            let mut child = cmd.spawn().context("failed to execute cryfs")?;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(passphrase.as_bytes())?;
                stdin.write_all(b"\n")?;
            }
            let output = child.wait_with_output()?;
            return if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(MntctlError::MountError(format!("cryfs failed: {}", stderr.trim())).into())
            };
        }

        // Pass through non-reserved options.
        for (k, v) in &config.options {
            if ENCRYPTED_RESERVED_OPTIONS.contains(&k.as_str()) {
                continue;
            }
            let val = option_to_string(v);
            if let Some(val) = val {
                if val.is_empty() {
                    cmd.arg(format!("--{k}"));
                } else {
                    cmd.arg(format!("--{k}")).arg(val);
                }
            }
        }

        cmd.arg(config.source()).arg(&target);

        // Pipe passphrase from context if available.
        if ctx.passphrase.is_some() {
            cmd.stdin(Stdio::piped());
        }

        let mut child = cmd.spawn().context("failed to execute cryfs")?;

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
            Err(MntctlError::MountError(format!("cryfs failed: {}", stderr.trim())).into())
        }
    }

    fn unmount(&self, config: &MountConfig) -> Result<()> {
        let target = config.resolved_target()?;
        fuse_unmount(&target)
    }

    fn is_mounted(&self, config: &MountConfig) -> Result<bool> {
        let target = config.resolved_target()?;
        fuse_is_mounted(&target, "cryfs")
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
            exec_args.push("--passphrase-file".to_string());
            exec_args.push(password_file);
        }
        // password_cmd handled via Environment + stdin in the unit

        exec_args.push("-f".to_string()); // foreground for systemd

        for (k, v) in &config.options {
            if ENCRYPTED_RESERVED_OPTIONS.contains(&k.as_str()) {
                continue;
            }
            let val = option_to_string(v);
            if let Some(val) = val {
                if val.is_empty() {
                    exec_args.push(format!("--{k}"));
                } else {
                    exec_args.push(format!("--{k}"));
                    exec_args.push(val);
                }
            }
        }

        exec_args.push(config.source().to_string());
        exec_args.push(target.to_string_lossy().to_string());

        let exec_start = format!("/usr/bin/cryfs {}", exec_args.join(" "));
        let exec_stop = format!("/usr/bin/fusermount -u {}", target.display());

        let mut unit = SystemdUnit::service(
            &format!("mntctl-{}", config.name()),
            &format!("mntctl mount: {} (cryfs)", config.name()),
            &exec_start,
            &exec_stop,
            "simple",
        );

        unit.add_entry("Service", "Environment", "CRYFS_FRONTEND=noninteractive");

        Ok(unit)
    }

    fn required_binaries(&self) -> Vec<&str> {
        vec!["cryfs", "fusermount"]
    }

    fn is_encrypted(&self) -> bool {
        true
    }
}

fn option_to_string(v: &toml::Value) -> Option<String> {
    match v {
        toml::Value::String(s) => Some(s.clone()),
        toml::Value::Boolean(b) => {
            if *b {
                Some(String::new())
            } else {
                None
            }
        }
        toml::Value::Integer(i) => Some(i.to_string()),
        other => Some(other.to_string()),
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
            toml::Value::String("/home/user/.secrets/cryfs.pass".to_string()),
        );

        MountConfig {
            mount: MountSection {
                name: "test-cryfs".to_string(),
                backend_type: BackendType::Cryfs,
                source: "/home/user/.encrypted/cryfs-vault".to_string(),
                target: "/tmp/mntctl-cryfs-test".to_string(),
                scope: MountScope::User,
            },
            options,
        }
    }

    #[test]
    fn validate_valid_config() {
        let backend = CryfsBackend;
        assert!(backend.validate_config(&sample_config()).is_ok());
    }

    #[test]
    fn validate_empty_source() {
        let backend = CryfsBackend;
        let mut config = sample_config();
        config.mount.source = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn validate_empty_target() {
        let backend = CryfsBackend;
        let mut config = sample_config();
        config.mount.target = String::new();
        assert!(backend.validate_config(&config).is_err());
    }

    #[test]
    fn generate_systemd_unit() {
        let backend = CryfsBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        let rendered = unit.render();
        assert!(rendered.contains("[Service]"));
        assert!(rendered.contains("cryfs"));
        assert!(rendered.contains("-f"));
        assert!(rendered.contains("--passphrase-file"));
        assert!(rendered.contains("/home/user/.secrets/cryfs.pass"));
        assert!(rendered.contains("CRYFS_FRONTEND=noninteractive"));
        assert!(rendered.contains("fusermount -u"));
    }

    #[test]
    fn unit_name_is_service() {
        let backend = CryfsBackend;
        let config = sample_config();
        let unit = backend.generate_systemd_unit(&config).unwrap();
        assert!(unit.name.ends_with(".service"));
    }

    #[test]
    fn is_encrypted_returns_true() {
        let backend = CryfsBackend;
        assert!(backend.is_encrypted());
    }
}
