use anyhow::Result;
use console::style;

use crate::core::operations;
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "Re-link installed skills to agent skill directories")]
pub struct RelinkArgs {
    /// Target agent ID (omit to relink all agents)
    pub agent: Option<String>,
    /// Only relink this specific skill
    #[arg(long, value_name = "NAME")]
    pub skill: Option<String>,
    /// Overwrite conflicting paths (non-skm symlinks or files)
    #[arg(long)]
    pub force: bool,
    /// Back up conflicting paths before overwriting (requires --force)
    #[arg(long)]
    pub backup: bool,
    /// Show what would be done without making any changes
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: RelinkArgs) -> Result<()> {
    let result = match (&args.agent, &args.skill) {
        (Some(agent), Some(skill)) => operations::relink_selected(
            Some(agent),
            Some(skill),
            args.force,
            args.backup,
            args.dry_run,
        )?,
        (Some(agent), None) => {
            operations::relink_agent(agent, args.force, args.backup, args.dry_run)?
        }
        (None, Some(skill)) => {
            operations::relink_selected(None, Some(skill), args.force, args.backup, args.dry_run)?
        }
        (None, None) => operations::relink_all(args.force, args.backup, args.dry_run)?,
    };

    println!(
        "{} {}",
        style(i18n::t("relink")).green().bold(),
        i18n::fmt_relink_result(result.linked, result.conflicts, result.skipped)
    );
    for error in result.errors {
        eprintln!("{} {error}", style(i18n::t("warn")).yellow().bold());
    }
    Ok(())
}
