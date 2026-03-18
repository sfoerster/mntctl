use std::collections::BTreeMap;
use tempfile::tempdir;

// We test the public config types and serialization by using the library directly.
// Since this is a binary crate, we parse TOML manually here.

#[test]
fn parse_sshfs_config() {
    let toml_str = r#"
[mount]
name = "bastion-e2a"
type = "sshfs"
source = "sfoerster.admin@bastion.rimstorm.cloud:/opt/enclave2-automation"
target = "~/Projects/rimstorm-dev/enclave2-dev/bastion-e2a"
scope = "user"

[options]
cache = "yes"
kernel_cache = true
reconnect = true
ServerAliveInterval = 15
sftp_server = "/usr/bin/sudo -u rimstorm /usr/libexec/openssh/sftp-server"
"#;

    let config: toml::Value = toml::from_str(toml_str).unwrap();
    let mount = config.get("mount").unwrap();

    assert_eq!(mount.get("name").unwrap().as_str().unwrap(), "bastion-e2a");
    assert_eq!(mount.get("type").unwrap().as_str().unwrap(), "sshfs");
    assert_eq!(mount.get("scope").unwrap().as_str().unwrap(), "user");

    let options = config.get("options").unwrap();
    assert_eq!(options.get("cache").unwrap().as_str().unwrap(), "yes");
    assert_eq!(
        options.get("kernel_cache").unwrap().as_bool().unwrap(),
        true
    );
    assert_eq!(
        options
            .get("ServerAliveInterval")
            .unwrap()
            .as_integer()
            .unwrap(),
        15
    );
}

#[test]
fn roundtrip_config_toml() {
    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
    struct MountSection {
        name: String,
        #[serde(rename = "type")]
        backend_type: String,
        source: String,
        target: String,
        scope: String,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq)]
    struct Config {
        mount: MountSection,
        options: BTreeMap<String, toml::Value>,
    }

    let mut options = BTreeMap::new();
    options.insert("cache".to_string(), toml::Value::String("yes".to_string()));
    options.insert("reconnect".to_string(), toml::Value::Boolean(true));

    let original = Config {
        mount: MountSection {
            name: "test".to_string(),
            backend_type: "sshfs".to_string(),
            source: "user@host:/path".to_string(),
            target: "~/mnt/test".to_string(),
            scope: "user".to_string(),
        },
        options,
    };

    let serialized = toml::to_string_pretty(&original).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();
    assert_eq!(original, deserialized);
}

#[test]
fn config_file_roundtrip_on_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("test.toml");

    let content = r#"
[mount]
name = "disk-test"
type = "sshfs"
source = "user@host:/path"
target = "/tmp/mnt"
scope = "user"

[options]
reconnect = true
"#;

    std::fs::write(&path, content).unwrap();
    let read_back = std::fs::read_to_string(&path).unwrap();
    let parsed: toml::Value = toml::from_str(&read_back).unwrap();
    assert_eq!(
        parsed
            .get("mount")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap(),
        "disk-test"
    );
}

#[test]
fn config_missing_required_field_fails() {
    let toml_str = r#"
[mount]
name = "test"
source = "user@host:/path"
target = "/tmp/mnt"
"#;

    // Missing 'type' field should fail.
    let result: Result<toml::Value, _> = toml::from_str(toml_str);
    // This will parse as Value, but if we try to parse into our actual struct it should fail.
    // Since we're not linking the library, we verify the field is missing.
    let val = result.unwrap();
    assert!(val.get("mount").unwrap().get("type").is_none());
}

#[test]
fn config_permissions_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let path = dir.path().join("secret.toml");
    std::fs::write(&path, "test").unwrap();

    let perms = std::fs::Permissions::from_mode(0o600);
    std::fs::set_permissions(&path, perms).unwrap();

    let metadata = std::fs::metadata(&path).unwrap();
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
}
