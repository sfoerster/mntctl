use serde::{Deserialize, Serialize};

/// Global mntctl configuration (optional, loaded from ~/.config/mntctl/config.toml).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GlobalConfig {
    /// Default backend type for `mntctl add` when -t is omitted.
    #[serde(default)]
    pub default_backend: Option<String>,

    /// Default editor for `mntctl edit` (overrides $EDITOR).
    #[serde(default)]
    pub editor: Option<String>,
}

#[allow(dead_code)]
impl GlobalConfig {
    pub fn load() -> Self {
        let path = match dirs::config_dir() {
            Some(d) => d.join("mntctl").join("config.toml"),
            None => return Self::default(),
        };

        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}
