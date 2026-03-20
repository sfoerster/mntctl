use crate::backend::BackendRegistry;
use crate::config;
use crate::output::table;
use anyhow::Result;

pub fn run(group: Option<&str>, system: bool, registry: &BackendRegistry) -> Result<()> {
    let configs = if let Some(group) = group {
        config::list_mount_configs_by_group(group, system)?
    } else if system {
        config::list_mount_configs(config::MountScope::System)?
    } else {
        config::list_all_mount_configs()?
    };

    println!("{}", table::render_mount_table(&configs, registry));

    Ok(())
}
