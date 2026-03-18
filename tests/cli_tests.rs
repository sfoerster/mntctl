use assert_cmd::Command;
use predicates::prelude::*;

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
