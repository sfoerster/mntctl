use crate::backend::BackendRegistry;
use crate::config::MountConfig;
use crate::output::color;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, ContentArrangement, Table};

/// Render a table of all mount configurations with their current status.
pub fn render_mount_table(configs: &[MountConfig], registry: &BackendRegistry) -> String {
    if configs.is_empty() {
        return "No mounts configured.".to_string();
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("NAME"),
            Cell::new("TYPE"),
            Cell::new("SOURCE"),
            Cell::new("TARGET"),
            Cell::new("SCOPE"),
            Cell::new("STATUS"),
            Cell::new("ENABLED"),
        ]);

    for config in configs {
        let mounted = match registry.get(config.backend_type()) {
            Some(backend) => match backend.is_mounted(config) {
                Ok(true) => color::status_style("mounted"),
                Ok(false) => color::status_style("unmounted"),
                Err(_) => color::status_style("error"),
            },
            None => "unknown".to_string(),
        };

        let unit_name = format!("mntctl-{}.service", config.name());
        let enabled = match crate::systemd::SystemdManager::is_enabled(&unit_name, config.scope()) {
            Ok(true) => color::status_style("enabled"),
            Ok(false) => color::status_style("disabled"),
            Err(_) => color::status_style("disabled"),
        };

        table.add_row(vec![
            Cell::new(config.name()),
            Cell::new(config.backend_type().to_string()),
            Cell::new(truncate(config.source(), 40)),
            Cell::new(truncate(config.target(), 35)),
            Cell::new(config.scope().to_string()),
            Cell::new(&mounted),
            Cell::new(&enabled),
        ]);
    }

    table.to_string()
}

/// Truncate a string with ellipsis if it exceeds max_len.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
