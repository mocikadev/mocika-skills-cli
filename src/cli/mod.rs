pub mod agent_cmd;
pub mod backup;
pub mod config_cmd;
pub mod doctor;
pub mod install;
pub mod list;
pub mod relink;
pub mod scan;
pub mod search;
pub mod self_update;
pub mod source;
pub mod update;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skm", about = "AI Agent skill package manager", long_about = None, version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a skill from registry or Git repository
    Install(install::InstallArgs),
    /// Search skills in configured registries
    Search(search::SearchArgs),
    /// Detect installed AI agents and register them
    Scan(scan::ScanArgs),
    /// Re-link installed skills to agent directories
    Relink(relink::RelinkArgs),
    /// Update installed skills to latest version
    Update(update::UpdateArgs),
    /// List all installed skills
    List(list::ListArgs),
    /// Show detailed information about a skill
    Info {
        /// Skill name
        name: String,
    },
    /// Uninstall a skill and remove all its symlinks
    Uninstall {
        /// Skill name
        name: String,
    },
    /// Create a symlink for a skill in an agent directory
    Link {
        /// Skill name
        name: String,
        /// Agent ID (e.g. opencode, claude-code)
        agent: String,
    },
    /// Remove a skill symlink from an agent directory
    Unlink {
        /// Skill name
        name: String,
        /// Agent ID (e.g. opencode, claude-code)
        agent: String,
    },
    /// Manage skill registries
    #[command(subcommand)]
    Source(source::SourceCommands),
    /// Manage registered AI agents
    #[command(subcommand)]
    Agent(agent_cmd::AgentCommands),
    /// Manage skill backups
    #[command(subcommand)]
    Backup(backup::BackupCommands),
    /// Configure skm settings (language, etc.)
    #[command(subcommand)]
    Config(config_cmd::ConfigCommands),
    /// Update skm itself to the latest release
    SelfUpdate(self_update::SelfUpdateArgs),
    /// Check environment and diagnose common issues
    Doctor(doctor::DoctorArgs),
}
