use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser, Debug)]
#[command(
    name = "mntctl",
    about = "Modular remote & encrypted mount manager",
    version,
    author = "Steven Foerster <https://stevenfoerster.com>",
    after_help = "Written by Steven Foerster <https://stevenfoerster.com>\n\nUse 'mntctl <command> --help' for more information about a command."
)]
pub struct Cli {
    /// Operate on system-level mounts (uses pkexec for privilege escalation)
    #[arg(long, global = true)]
    pub system: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Add a new mount configuration
    Add {
        /// Mount name (used as identifier)
        name: String,

        /// Backend type (sshfs, rclone, nfs, smb, gocryptfs, cryfs, encfs)
        #[arg(short = 't', long = "type")]
        backend_type: Option<String>,

        /// Mount source (e.g. user@host:/path)
        #[arg(short = 's', long)]
        source: String,

        /// Mount target directory
        #[arg(short = 'T', long)]
        target: String,

        /// Mount options as key=val pairs (comma-separated)
        #[arg(short = 'o', long, value_delimiter = ',')]
        options: Vec<String>,

        /// Assign to groups (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        group: Vec<String>,
    },

    /// Remove a mount configuration
    Remove {
        /// Mount name
        name: String,

        /// Force removal even if mounted
        #[arg(long)]
        force: bool,
    },

    /// Mount a filesystem (transient)
    Start {
        /// Mount name
        name: Option<String>,

        /// Mount all configured filesystems
        #[arg(long)]
        all: bool,

        /// Mount all filesystems in a group
        #[arg(short, long)]
        group: Option<String>,
    },

    /// Unmount a filesystem
    Stop {
        /// Mount name
        name: Option<String>,

        /// Unmount all mounted filesystems
        #[arg(long)]
        all: bool,

        /// Unmount all filesystems in a group
        #[arg(short, long)]
        group: Option<String>,
    },

    /// Install and enable a systemd unit for persistent mounting
    Enable {
        /// Mount name
        name: String,
    },

    /// Disable a systemd unit
    Disable {
        /// Mount name
        name: String,
    },

    /// Unmount and remount a filesystem
    Restart {
        /// Mount name
        name: Option<String>,

        /// Restart all configured filesystems
        #[arg(long)]
        all: bool,

        /// Restart all filesystems in a group
        #[arg(short, long)]
        group: Option<String>,
    },

    /// Show detailed status of a mount, or overview of all mounts
    Status {
        /// Mount name (omit for overview)
        name: Option<String>,
    },

    /// List all configured mounts
    List {
        /// Filter by group
        #[arg(short, long)]
        group: Option<String>,
    },

    /// Open a mount configuration in $EDITOR
    Edit {
        /// Mount name
        name: String,
    },

    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Check system dependencies and report status
    Doctor,
}
