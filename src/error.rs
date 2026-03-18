use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum MntctlError {
    #[error("mount '{0}' not found")]
    MountNotFound(String),

    #[error("mount '{0}' already exists")]
    MountAlreadyExists(String),

    #[error("mount '{0}' is already mounted")]
    AlreadyMounted(String),

    #[error("mount '{0}' is not mounted")]
    NotMounted(String),

    #[error("unknown backend type: {0}")]
    UnknownBackend(String),

    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("failed to read config file {path}: {source}")]
    ConfigReadError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse config file {path}: {source}")]
    ConfigParseError {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("failed to write config file {path}: {source}")]
    ConfigWriteError {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("systemd error: {0}")]
    SystemdError(String),

    #[error("mount command failed: {0}")]
    MountError(String),

    #[error("unmount command failed: {0}")]
    UnmountError(String),

    #[error("required binary not found: {0}")]
    BinaryNotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("target directory does not exist: {0}")]
    TargetNotFound(PathBuf),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Exit codes matching the plan specification.
#[allow(dead_code)]
pub const EXIT_SUCCESS: i32 = 0;
pub const EXIT_GENERAL_ERROR: i32 = 1;
pub const EXIT_CONFIG_ERROR: i32 = 2;
pub const EXIT_SYSTEMD_ERROR: i32 = 3;

impl MntctlError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::ConfigError(_)
            | Self::ConfigReadError { .. }
            | Self::ConfigParseError { .. }
            | Self::ConfigWriteError { .. }
            | Self::UnknownBackend(_) => EXIT_CONFIG_ERROR,

            Self::SystemdError(_) => EXIT_SYSTEMD_ERROR,

            _ => EXIT_GENERAL_ERROR,
        }
    }
}
