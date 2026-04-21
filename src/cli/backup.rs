use anyhow::Result;
use clap::Subcommand;
use console::style;

use crate::core::backup;
use crate::i18n;

#[derive(Subcommand)]
pub enum BackupCommands {
    /// List all backup snapshots for a skill
    List {
        /// Skill name
        name: String,
    },
    /// Restore a skill from a backup snapshot
    Restore {
        /// Skill name
        name: String,
        /// Snapshot ID to restore (defaults to latest)
        snapshot_id: Option<String>,
    },
    /// Delete a specific backup snapshot
    Delete {
        /// Skill name
        name: String,
        /// Snapshot ID to delete
        snapshot_id: String,
    },
}

pub fn run(cmd: BackupCommands) -> Result<()> {
    match cmd {
        BackupCommands::List { name } => {
            let backups = backup::list_backups(&name)?;
            if backups.is_empty() {
                println!(
                    "{} {} {name}",
                    style(i18n::t("info")).cyan().bold(),
                    i18n::t("no backups found for")
                );
            } else {
                for b in backups {
                    println!(
                        "{}  {}  {}",
                        style(b.snapshot_id).green(),
                        b.created_at,
                        b.backup_path
                    );
                }
            }
        }
        BackupCommands::Restore { name, snapshot_id } => {
            backup::restore_backup(&name, snapshot_id)?;
            println!(
                "{} {} {name}",
                style(i18n::t("ok")).green().bold(),
                i18n::t("restored")
            );
        }
        BackupCommands::Delete { name, snapshot_id } => {
            backup::delete_backup(&name, &snapshot_id)?;
            println!(
                "{} {} {snapshot_id}",
                style(i18n::t("ok")).green().bold(),
                i18n::t("deleted backup")
            );
        }
    }
    Ok(())
}
