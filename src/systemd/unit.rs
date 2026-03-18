/// Represents a systemd unit file that can be rendered to a string.
#[derive(Debug, Clone)]
pub struct SystemdUnit {
    pub name: String,
    pub sections: Vec<UnitSection>,
}

#[derive(Debug, Clone)]
pub struct UnitSection {
    pub name: String,
    pub entries: Vec<(String, String)>,
}

impl SystemdUnit {
    /// Create a standard .service unit for FUSE-based mounts.
    pub fn service(
        name: &str,
        description: &str,
        exec_start: &str,
        exec_stop: &str,
        service_type: &str,
    ) -> Self {
        Self {
            name: format!("{name}.service"),
            sections: vec![
                UnitSection {
                    name: "Unit".to_string(),
                    entries: vec![
                        ("Description".to_string(), description.to_string()),
                        ("After".to_string(), "network-online.target".to_string()),
                        ("Wants".to_string(), "network-online.target".to_string()),
                    ],
                },
                UnitSection {
                    name: "Service".to_string(),
                    entries: vec![
                        ("Type".to_string(), service_type.to_string()),
                        ("ExecStart".to_string(), exec_start.to_string()),
                        ("ExecStop".to_string(), exec_stop.to_string()),
                        ("Restart".to_string(), "on-failure".to_string()),
                        ("RestartSec".to_string(), "5".to_string()),
                    ],
                },
                UnitSection {
                    name: "Install".to_string(),
                    entries: vec![("WantedBy".to_string(), "default.target".to_string())],
                },
            ],
        }
    }

    /// Create a .mount unit for kernel-based mounts.
    #[allow(dead_code)]
    pub fn mount_unit(
        name: &str,
        description: &str,
        what: &str,
        where_path: &str,
        fs_type: &str,
        options: &str,
    ) -> Self {
        let mut mount_entries = vec![
            ("What".to_string(), what.to_string()),
            ("Where".to_string(), where_path.to_string()),
            ("Type".to_string(), fs_type.to_string()),
        ];
        if !options.is_empty() {
            mount_entries.push(("Options".to_string(), options.to_string()));
        }

        Self {
            name: format!("{name}.mount"),
            sections: vec![
                UnitSection {
                    name: "Unit".to_string(),
                    entries: vec![
                        ("Description".to_string(), description.to_string()),
                        ("After".to_string(), "network-online.target".to_string()),
                        ("Wants".to_string(), "network-online.target".to_string()),
                    ],
                },
                UnitSection {
                    name: "Mount".to_string(),
                    entries: mount_entries,
                },
                UnitSection {
                    name: "Install".to_string(),
                    entries: vec![("WantedBy".to_string(), "default.target".to_string())],
                },
            ],
        }
    }

    /// Add an entry to an existing section by name.
    pub fn add_entry(&mut self, section_name: &str, key: &str, value: &str) {
        for section in &mut self.sections {
            if section.name == section_name {
                section.entries.push((key.to_string(), value.to_string()));
                return;
            }
        }
    }

    /// Render the unit file to a string.
    pub fn render(&self) -> String {
        let mut output = String::new();
        for (i, section) in self.sections.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&format!("[{}]\n", section.name));
            for (key, value) in &section.entries {
                output.push_str(&format!("{key}={value}\n"));
            }
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_service_unit() {
        let unit = SystemdUnit::service(
            "mntctl-test",
            "mntctl mount: test (sshfs)",
            "/usr/bin/sshfs user@host:/path /mnt/test -f",
            "/usr/bin/fusermount -u /mnt/test",
            "simple",
        );

        let rendered = unit.render();
        assert!(rendered.contains("[Unit]"));
        assert!(rendered.contains("Description=mntctl mount: test (sshfs)"));
        assert!(rendered.contains("[Service]"));
        assert!(rendered.contains("Type=simple"));
        assert!(rendered.contains("ExecStart=/usr/bin/sshfs user@host:/path /mnt/test -f"));
        assert!(rendered.contains("ExecStop=/usr/bin/fusermount -u /mnt/test"));
        assert!(rendered.contains("Restart=on-failure"));
        assert!(rendered.contains("[Install]"));
        assert!(rendered.contains("WantedBy=default.target"));
    }

    #[test]
    fn render_mount_unit() {
        let unit = SystemdUnit::mount_unit(
            "mnt-nfs",
            "mntctl mount: nfs-share (nfs)",
            "server:/export",
            "/mnt/nfs",
            "nfs4",
            "rw,soft",
        );

        let rendered = unit.render();
        assert!(rendered.contains("[Mount]"));
        assert!(rendered.contains("What=server:/export"));
        assert!(rendered.contains("Where=/mnt/nfs"));
        assert!(rendered.contains("Type=nfs4"));
        assert!(rendered.contains("Options=rw,soft"));
    }

    #[test]
    fn unit_name_has_correct_suffix() {
        let service = SystemdUnit::service("test", "desc", "cmd", "stop", "simple");
        assert!(service.name.ends_with(".service"));

        let mount = SystemdUnit::mount_unit("test", "desc", "what", "where", "nfs", "");
        assert!(mount.name.ends_with(".mount"));
    }
}
