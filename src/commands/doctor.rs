use crate::backend::BackendRegistry;
use crate::output::color;
use anyhow::Result;

struct Check {
    name: String,
    status: CheckStatus,
    detail: String,
}

enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

pub fn run(registry: &BackendRegistry) -> Result<()> {
    let mut checks = vec![
        check_proc_mounts(),
        check_systemd(),
        check_binary("systemctl"),
        check_binary("fusermount"),
        check_binary("pkexec"),
    ];

    // Per-backend binary checks.
    use crate::config::BackendType;
    let backends: &[(BackendType, &[&str])] = &[
        (BackendType::Sshfs, &["sshfs"]),
        (BackendType::Rclone, &["rclone"]),
        (BackendType::Nfs, &["mount.nfs"]),
        (BackendType::Smb, &["mount.cifs"]),
        (BackendType::Gocryptfs, &["gocryptfs"]),
        (BackendType::Cryfs, &["cryfs"]),
        (BackendType::Encfs, &["encfs"]),
    ];

    for (backend_type, binaries) in backends {
        let registered = registry.get(*backend_type).is_some();
        for bin in *binaries {
            let mut check = check_binary(bin);
            if !registered {
                check.name = format!("{} (backend not yet implemented)", check.name);
            }
            checks.push(check);
        }
    }

    // Print results.
    println!("{}", color::label_style("mntctl doctor"));
    println!();

    let mut has_fail = false;
    for check in &checks {
        let icon = match check.status {
            CheckStatus::Ok => color::success("ok"),
            CheckStatus::Warn => color::info("--"),
            CheckStatus::Fail => {
                has_fail = true;
                color::error("!!")
            }
        };
        println!("  [{}] {}: {}", icon, check.name, check.detail);
    }

    println!();
    if has_fail {
        println!(
            "{}",
            color::info(
                "Some checks failed. Install missing dependencies for the backends you need."
            )
        );
    } else {
        println!("{}", color::success("All checks passed."));
    }

    Ok(())
}

fn check_proc_mounts() -> Check {
    let path = std::path::Path::new("/proc/mounts");
    if path.exists() {
        Check {
            name: "/proc/mounts".to_string(),
            status: CheckStatus::Ok,
            detail: "available".to_string(),
        }
    } else {
        Check {
            name: "/proc/mounts".to_string(),
            status: CheckStatus::Fail,
            detail: "not found (mount status detection will not work)".to_string(),
        }
    }
}

fn check_systemd() -> Check {
    // Check if systemd is the init system (PID 1).
    match std::fs::read_link("/proc/1/exe") {
        Ok(path) => {
            let path_str = path.to_string_lossy();
            if path_str.contains("systemd") {
                Check {
                    name: "systemd (init)".to_string(),
                    status: CheckStatus::Ok,
                    detail: format!("PID 1 is {}", path_str),
                }
            } else {
                Check {
                    name: "systemd (init)".to_string(),
                    status: CheckStatus::Fail,
                    detail: format!("PID 1 is {} (mntctl requires systemd)", path_str),
                }
            }
        }
        Err(_) => {
            // Can't read /proc/1/exe — check for systemctl instead.
            if which::which("systemctl").is_ok() {
                Check {
                    name: "systemd (init)".to_string(),
                    status: CheckStatus::Warn,
                    detail: "could not verify PID 1, but systemctl is available".to_string(),
                }
            } else {
                Check {
                    name: "systemd (init)".to_string(),
                    status: CheckStatus::Fail,
                    detail: "could not verify init system and systemctl not found".to_string(),
                }
            }
        }
    }
}

fn check_binary(name: &str) -> Check {
    match which::which(name) {
        Ok(path) => Check {
            name: name.to_string(),
            status: CheckStatus::Ok,
            detail: path.to_string_lossy().to_string(),
        },
        Err(_) => Check {
            name: name.to_string(),
            status: CheckStatus::Warn,
            detail: "not found".to_string(),
        },
    }
}
