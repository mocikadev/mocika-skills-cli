use anyhow::Result;
use console::style;

use crate::core::{agent, config};
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "Detect installed AI agents and register them in agents.toml")]
pub struct ScanArgs {
    /// Preview detected agents without writing to agents.toml
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: ScanArgs) -> Result<()> {
    let detected = agent::detect_agents()?;
    let existing = config::load_agents()?;
    let mut new_ids = Vec::new();
    let mut removed_ids = Vec::new();

    for id in existing.agents.keys() {
        if !agent::is_agent_present(id) {
            removed_ids.push(id.clone());
        }
    }
    removed_ids.sort();

    for item in detected.into_iter().filter(|item| item.installed) {
        if existing.agents.contains_key(&item.id) {
            continue;
        }
        new_ids.push(item.id.clone());
        if !args.dry_run {
            config::add_agent_entry(&item.id, &item.skills_dir)?;
        }
    }

    for id in &removed_ids {
        if !args.dry_run {
            config::remove_agent_entry(id)?;
        }
    }

    if new_ids.is_empty() && removed_ids.is_empty() {
        println!(
            "{} {}",
            style(i18n::t("scan")).cyan().bold(),
            i18n::t("no changes detected")
        );
    } else {
        if !new_ids.is_empty() {
            println!(
                "{} {}",
                style(i18n::t("scan")).green().bold(),
                i18n::fmt_new_agents(new_ids.len(), &new_ids.join(", "))
            );
        }
        if !removed_ids.is_empty() {
            println!(
                "{} {}",
                style(i18n::t("scan")).yellow().bold(),
                i18n::fmt_removed_agents(removed_ids.len(), &removed_ids.join(", "))
            );
        }
    }

    Ok(())
}
