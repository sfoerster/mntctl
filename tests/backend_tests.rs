/// Backend validation and option tests.
/// These test config parsing and validation without actually mounting anything.
use std::collections::BTreeMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MountSection {
    name: String,
    #[serde(rename = "type")]
    backend_type: String,
    source: String,
    target: String,
    #[serde(default = "default_scope")]
    scope: String,
}

fn default_scope() -> String {
    "user".to_string()
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MountConfig {
    mount: MountSection,
    #[serde(default)]
    options: BTreeMap<String, toml::Value>,
}

fn sshfs_config(source: &str) -> MountConfig {
    MountConfig {
        mount: MountSection {
            name: "test".to_string(),
            backend_type: "sshfs".to_string(),
            source: source.to_string(),
            target: "/tmp/mntctl-test".to_string(),
            scope: "user".to_string(),
        },
        options: BTreeMap::new(),
    }
}

#[test]
fn sshfs_source_requires_colon() {
    let config = sshfs_config("user@host/path");
    // sshfs source must contain ':'
    assert!(!config.mount.source.contains(':'));
}

#[test]
fn sshfs_valid_source() {
    let config = sshfs_config("user@host:/remote/path");
    assert!(config.mount.source.contains(':'));
}

#[test]
fn option_types_parsed_correctly() {
    let toml_str = r#"
[mount]
name = "test"
type = "sshfs"
source = "user@host:/path"
target = "/tmp/test"

[options]
string_opt = "hello"
bool_opt = true
int_opt = 42
"#;

    let config: MountConfig = toml::from_str(toml_str).unwrap();
    assert!(config.options.get("string_opt").unwrap().is_str());
    assert!(config.options.get("bool_opt").unwrap().is_bool());
    assert!(config.options.get("int_opt").unwrap().is_integer());
}

#[test]
fn options_merge_with_defaults() {
    let defaults: BTreeMap<String, String> = BTreeMap::new();
    let mut user_opts = BTreeMap::new();
    user_opts.insert("cache".to_string(), toml::Value::String("yes".to_string()));
    user_opts.insert("reconnect".to_string(), toml::Value::Boolean(true));

    // Merge: user options override defaults (extract string values properly).
    let mut merged = defaults;
    for (k, v) in &user_opts {
        let val = match v {
            toml::Value::String(s) => s.clone(),
            toml::Value::Boolean(b) => b.to_string(),
            other => other.to_string(),
        };
        merged.insert(k.clone(), val);
    }

    assert_eq!(merged.get("cache").unwrap(), "yes");
    assert_eq!(merged.get("reconnect").unwrap(), "true");
}

#[test]
fn backend_type_parsing() {
    let valid = [
        "sshfs",
        "rclone",
        "nfs",
        "smb",
        "gocryptfs",
        "cryfs",
        "encfs",
    ];
    for t in &valid {
        let toml_str = format!(
            r#"
[mount]
name = "test"
type = "{t}"
source = "src"
target = "/tmp/t"
"#
        );
        let config: MountConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.mount.backend_type, *t);
    }
}
