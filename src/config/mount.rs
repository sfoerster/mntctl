use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use crate::error::MntctlError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    Sshfs,
    Rclone,
    Nfs,
    Smb,
    Gocryptfs,
    Cryfs,
    Encfs,
}

impl fmt::Display for BackendType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sshfs => write!(f, "sshfs"),
            Self::Rclone => write!(f, "rclone"),
            Self::Nfs => write!(f, "nfs"),
            Self::Smb => write!(f, "smb"),
            Self::Gocryptfs => write!(f, "gocryptfs"),
            Self::Cryfs => write!(f, "cryfs"),
            Self::Encfs => write!(f, "encfs"),
        }
    }
}

impl FromStr for BackendType {
    type Err = MntctlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sshfs" => Ok(Self::Sshfs),
            "rclone" => Ok(Self::Rclone),
            "nfs" => Ok(Self::Nfs),
            "smb" | "cifs" => Ok(Self::Smb),
            "gocryptfs" => Ok(Self::Gocryptfs),
            "cryfs" => Ok(Self::Cryfs),
            "encfs" => Ok(Self::Encfs),
            other => Err(MntctlError::UnknownBackend(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountScope {
    User,
    System,
}

impl fmt::Display for MountScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::System => write!(f, "system"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountSection {
    pub name: String,
    #[serde(rename = "type")]
    pub backend_type: BackendType,
    pub source: String,
    pub target: String,
    #[serde(default = "default_scope")]
    pub scope: MountScope,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
}

fn default_scope() -> MountScope {
    MountScope::User
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub mount: MountSection,
    #[serde(default)]
    pub options: BTreeMap<String, toml::Value>,
}

#[allow(dead_code)]
impl MountConfig {
    pub fn name(&self) -> &str {
        &self.mount.name
    }

    pub fn backend_type(&self) -> BackendType {
        self.mount.backend_type
    }

    pub fn source(&self) -> &str {
        &self.mount.source
    }

    pub fn target(&self) -> &str {
        &self.mount.target
    }

    pub fn scope(&self) -> MountScope {
        self.mount.scope
    }

    pub fn groups(&self) -> &[String] {
        &self.mount.groups
    }

    /// Resolve the target path, expanding `~` to the user's home directory.
    pub fn resolved_target(&self) -> anyhow::Result<std::path::PathBuf> {
        crate::config::expand_tilde(&self.mount.target)
    }

    /// Get an option value as a string.
    pub fn option_str(&self, key: &str) -> Option<String> {
        self.options.get(key).map(|v| match v {
            toml::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
    }

    /// Get an option value as a bool.
    pub fn option_bool(&self, key: &str) -> Option<bool> {
        self.options.get(key).and_then(|v| v.as_bool())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_backend_type() {
        assert_eq!(BackendType::from_str("sshfs").unwrap(), BackendType::Sshfs);
        assert_eq!(BackendType::from_str("SSHFS").unwrap(), BackendType::Sshfs);
        assert_eq!(BackendType::from_str("nfs").unwrap(), BackendType::Nfs);
        assert_eq!(BackendType::from_str("smb").unwrap(), BackendType::Smb);
        assert_eq!(BackendType::from_str("cifs").unwrap(), BackendType::Smb);
        assert!(BackendType::from_str("bogus").is_err());
    }

    #[test]
    fn backend_type_display_roundtrip() {
        for bt in [
            BackendType::Sshfs,
            BackendType::Rclone,
            BackendType::Nfs,
            BackendType::Smb,
            BackendType::Gocryptfs,
            BackendType::Cryfs,
            BackendType::Encfs,
        ] {
            let s = bt.to_string();
            assert_eq!(BackendType::from_str(&s).unwrap(), bt);
        }
    }

    #[test]
    fn parse_mount_config_toml() {
        let toml_str = r#"
[mount]
name = "test-mount"
type = "sshfs"
source = "user@host:/path"
target = "~/mnt/test"
scope = "user"

[options]
cache = "yes"
reconnect = true
ServerAliveInterval = 15
"#;
        let config: MountConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.name(), "test-mount");
        assert_eq!(config.backend_type(), BackendType::Sshfs);
        assert_eq!(config.source(), "user@host:/path");
        assert_eq!(config.target(), "~/mnt/test");
        assert_eq!(config.scope(), MountScope::User);
        assert_eq!(config.groups(), &[] as &[String]);
        assert_eq!(config.option_str("cache").unwrap(), "yes");
        assert_eq!(config.option_bool("reconnect").unwrap(), true);
    }

    #[test]
    fn parse_mount_config_default_scope() {
        let toml_str = r#"
[mount]
name = "test"
type = "nfs"
source = "server:/export"
target = "/mnt/nfs"
"#;
        let config: MountConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.scope(), MountScope::User);
        assert_eq!(config.groups(), &[] as &[String]);
    }

    #[test]
    fn parse_mount_config_with_groups() {
        let toml_str = r#"
[mount]
name = "test"
type = "sshfs"
source = "user@host:/path"
target = "~/mnt/test"
groups = ["work", "daily"]
"#;
        let config: MountConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.groups(), &["work", "daily"]);
    }

    #[test]
    fn serialize_mount_config() {
        let config = MountConfig {
            mount: MountSection {
                name: "test".to_string(),
                backend_type: BackendType::Sshfs,
                source: "user@host:/path".to_string(),
                target: "~/mnt/test".to_string(),
                scope: MountScope::User,
                groups: vec![],
            },
            options: BTreeMap::new(),
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        let parsed: MountConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.name(), config.name());
        // Empty groups should not appear in serialized output
        assert!(!serialized.contains("groups"));
    }

    #[test]
    fn serialize_mount_config_with_groups() {
        let config = MountConfig {
            mount: MountSection {
                name: "test".to_string(),
                backend_type: BackendType::Sshfs,
                source: "user@host:/path".to_string(),
                target: "~/mnt/test".to_string(),
                scope: MountScope::User,
                groups: vec!["work".to_string(), "daily".to_string()],
            },
            options: BTreeMap::new(),
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        assert!(serialized.contains("groups"));
        let parsed: MountConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(parsed.groups(), &["work", "daily"]);
    }
}
