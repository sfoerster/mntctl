use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

fn write_global_config(base: &std::path::Path, contents: &str) {
    let config_dir = base.join("mntctl");
    fs::create_dir_all(&config_dir).unwrap();
    fs::write(config_dir.join("config.toml"), contents).unwrap();
}

fn write_mount_config(base: &std::path::Path, name: &str, contents: &str) {
    let mounts_dir = base.join("mntctl").join("mounts");
    fs::create_dir_all(&mounts_dir).unwrap();
    fs::write(mounts_dir.join(format!("{name}.toml")), contents).unwrap();
}

#[test]
fn cli_no_args_shows_help() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn cli_help_flag() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Modular remote & encrypted mount manager",
        ));
}

#[test]
fn cli_version_flag() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mntctl"));
}

#[test]
fn cli_list_empty() {
    // Use a temp dir so no real configs are read.
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("list")
        .env("XDG_CONFIG_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No mounts configured"));
}

#[test]
fn cli_status_no_name_empty() {
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("status")
        .env("XDG_CONFIG_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No mounts configured"));
}

#[test]
fn cli_status_nonexistent_mount() {
    let tmp = tempfile::tempdir().unwrap();
    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["status", "nonexistent"])
        .env("XDG_CONFIG_HOME", tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn cli_add_subcommand_help() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Backend type"))
        .stdout(predicate::str::contains("Mount source"));
}

#[test]
fn cli_completion_bash() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_mntctl"));
}

#[test]
fn cli_doctor_runs() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("mntctl doctor"))
        .stdout(predicate::str::contains("/proc/mounts"))
        .stdout(predicate::str::contains("systemd"));
}

#[test]
fn cli_doctor_checks_core_binaries() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("systemctl"))
        .stdout(predicate::str::contains("fusermount"));
}

#[test]
fn cli_doctor_checks_backend_binaries() {
    Command::cargo_bin("mntctl")
        .unwrap()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("sshfs"))
        .stdout(predicate::str::contains("rclone"));
}

#[test]
fn cli_add_uses_default_backend_from_global_config() {
    let tmp = tempfile::tempdir().unwrap();
    write_global_config(tmp.path(), "default_backend = \"sshfs\"\n");

    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["add", "demo", "-s", "user@host:/path", "-T", "/tmp/demo"])
        .env("XDG_CONFIG_HOME", tmp.path())
        .assert()
        .success();

    let config_path = tmp.path().join("mntctl").join("mounts").join("demo.toml");
    let config = fs::read_to_string(config_path).unwrap();
    assert!(config.contains("type = \"sshfs\""));
}

#[test]
fn cli_edit_uses_global_editor_override() {
    let tmp = tempfile::tempdir().unwrap();
    write_global_config(tmp.path(), "editor = \"/bin/true\"\n");
    write_mount_config(
        tmp.path(),
        "demo",
        r#"
[mount]
name = "demo"
type = "sshfs"
source = "user@host:/path"
target = "/tmp/demo"
"#,
    );

    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["edit", "demo"])
        .env("XDG_CONFIG_HOME", tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration valid"));
}

#[test]
fn cli_add_rejects_duplicate_name_across_scopes() {
    let tmp = tempfile::tempdir().unwrap();
    let system_dir = tmp.path().join("system-mounts");

    Command::cargo_bin("mntctl")
        .unwrap()
        .args([
            "add",
            "dup",
            "-t",
            "sshfs",
            "-s",
            "user@host:/path",
            "-T",
            "/tmp/dup",
        ])
        .env("XDG_CONFIG_HOME", tmp.path())
        .env("MNTCTL_SYSTEM_CONFIG_DIR", &system_dir)
        .assert()
        .success();

    Command::cargo_bin("mntctl")
        .unwrap()
        .args([
            "--system",
            "add",
            "dup",
            "-t",
            "nfs",
            "-s",
            "server:/export",
            "-T",
            "/mnt/dup",
        ])
        .env("XDG_CONFIG_HOME", tmp.path())
        .env("MNTCTL_SYSTEM_CONFIG_DIR", &system_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn cli_status_system_flag_prefers_system_scope() {
    let tmp = tempfile::tempdir().unwrap();
    let system_dir = tmp.path().join("system-mounts");
    write_mount_config(
        tmp.path(),
        "shared",
        r#"
[mount]
name = "shared"
type = "sshfs"
source = "user@host:/user"
target = "/tmp/user"
"#,
    );
    fs::create_dir_all(&system_dir).unwrap();
    fs::write(
        system_dir.join("shared.toml"),
        r#"
[mount]
name = "shared"
type = "nfs"
source = "server:/system"
target = "/mnt/system"
scope = "system"
"#,
    )
    .unwrap();

    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["--system", "status", "shared"])
        .env("XDG_CONFIG_HOME", tmp.path())
        .env("MNTCTL_SYSTEM_CONFIG_DIR", &system_dir)
        .env("MNTCTL_SYSTEMCTL_BIN", "/bin/true")
        .assert()
        .success()
        .stdout(predicate::str::contains("server:/system"))
        .stdout(predicate::str::contains("Scope:  system"));
}

#[test]
fn cli_enable_system_writes_nonempty_unit() {
    let tmp = tempfile::tempdir().unwrap();
    let system_dir = tmp.path().join("system-mounts");
    let systemd_dir = tmp.path().join("systemd");

    Command::cargo_bin("mntctl")
        .unwrap()
        .args([
            "--system",
            "add",
            "share",
            "-t",
            "nfs",
            "-s",
            "server:/export",
            "-T",
            "/mnt/share",
        ])
        .env("XDG_CONFIG_HOME", tmp.path())
        .env("MNTCTL_SYSTEM_CONFIG_DIR", &system_dir)
        .assert()
        .success();

    Command::cargo_bin("mntctl")
        .unwrap()
        .args(["--system", "enable", "share"])
        .env("XDG_CONFIG_HOME", tmp.path())
        .env("MNTCTL_SYSTEM_CONFIG_DIR", &system_dir)
        .env("MNTCTL_SYSTEMD_SYSTEM_DIR", &systemd_dir)
        .env("MNTCTL_SYSTEMCTL_BIN", "/bin/true")
        .assert()
        .success();

    let unit_path = systemd_dir.join("mnt-share.mount");
    let unit = fs::read_to_string(unit_path).unwrap();
    assert!(unit.contains("[Mount]"));
    assert!(unit.contains("What=server:/export"));
}
