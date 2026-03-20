mod backend;
mod cli;
mod commands;
mod config;
mod error;
mod output;
mod systemd;

use backend::BackendRegistry;
use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();

    let registry = BackendRegistry::new();

    let result = match cli.command {
        Command::Add {
            name,
            backend_type,
            source,
            target,
            options,
            group,
        } => commands::add::run(
            &name,
            backend_type.as_deref(),
            &source,
            &target,
            &options,
            &group,
            cli.system,
            &registry,
        ),

        Command::Remove { name, force } => {
            commands::remove::run(&name, force, cli.system, &registry)
        }

        Command::Start { name, all, group } => {
            if all {
                commands::start::run_all(cli.system, &registry)
            } else if let Some(group) = group {
                commands::start::run_group(&group, cli.system, &registry)
            } else if let Some(name) = name {
                commands::start::run(&name, cli.system, &registry)
            } else {
                eprintln!(
                    "{}: provide a mount name, --all, or --group",
                    output::color::error("error")
                );
                std::process::exit(1);
            }
        }

        Command::Stop { name, all, group } => {
            if all {
                commands::stop::run_all(cli.system, &registry)
            } else if let Some(group) = group {
                commands::stop::run_group(&group, cli.system, &registry)
            } else if let Some(name) = name {
                commands::stop::run(&name, cli.system, &registry)
            } else {
                eprintln!(
                    "{}: provide a mount name, --all, or --group",
                    output::color::error("error")
                );
                std::process::exit(1);
            }
        }

        Command::Enable { name } => commands::enable::run(&name, cli.system, &registry),

        Command::Disable { name } => commands::disable::run(&name, cli.system),

        Command::Restart { name, all, group } => {
            if all {
                commands::restart::run_all(cli.system, &registry)
            } else if let Some(group) = group {
                commands::restart::run_group(&group, cli.system, &registry)
            } else if let Some(name) = name {
                commands::restart::run(&name, cli.system, &registry)
            } else {
                eprintln!(
                    "{}: provide a mount name, --all, or --group",
                    output::color::error("error")
                );
                std::process::exit(1);
            }
        }

        Command::Status { name } => commands::status::run(name.as_deref(), cli.system, &registry),

        Command::List { group } => commands::list::run(group.as_deref(), cli.system, &registry),

        Command::Edit { name } => commands::edit::run(&name, cli.system),

        Command::Completion { shell } => {
            commands::completion::run(shell);
            Ok(())
        }

        Command::Doctor => commands::doctor::run(&registry),
    };

    if let Err(e) = result {
        eprintln!("{}: {e:#}", output::color::error("error"));

        let exit_code = e
            .downcast_ref::<error::MntctlError>()
            .map(|e| e.exit_code())
            .unwrap_or(error::EXIT_GENERAL_ERROR);

        std::process::exit(exit_code);
    }
}
