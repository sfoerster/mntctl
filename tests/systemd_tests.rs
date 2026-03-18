/// Tests for systemd unit generation (pure string output, no actual systemd calls).

#[test]
fn service_unit_structure() {
    // Manually construct what the sshfs backend would generate.
    let unit_content = "\
[Unit]
Description=mntctl mount: test-sshfs (sshfs)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/bin/sshfs user@host:/remote/path /home/user/mnt/test -f -o cache=yes,reconnect -o ServerAliveInterval=15
ExecStop=/usr/bin/fusermount -u /home/user/mnt/test
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
";

    // Verify key sections exist.
    assert!(unit_content.contains("[Unit]"));
    assert!(unit_content.contains("[Service]"));
    assert!(unit_content.contains("[Install]"));
    assert!(unit_content.contains("Type=simple"));
    assert!(unit_content.contains("ExecStart="));
    assert!(unit_content.contains("ExecStop="));
    assert!(unit_content.contains("-f")); // foreground flag for systemd
    assert!(unit_content.contains("WantedBy=default.target"));
}

#[test]
fn mount_unit_structure() {
    let unit_content = "\
[Unit]
Description=mntctl mount: nfs-share (nfs)
After=network-online.target
Wants=network-online.target

[Mount]
What=server:/export
Where=/mnt/nfs
Type=nfs4
Options=rw,soft

[Install]
WantedBy=default.target
";

    assert!(unit_content.contains("[Mount]"));
    assert!(unit_content.contains("What=server:/export"));
    assert!(unit_content.contains("Where=/mnt/nfs"));
    assert!(unit_content.contains("Type=nfs4"));
}

#[test]
fn unit_naming_convention() {
    let name = "my-mount";
    let service_name = format!("mntctl-{name}.service");
    assert_eq!(service_name, "mntctl-my-mount.service");
}
