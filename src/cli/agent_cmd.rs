use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use clap::Subcommand;
use console::style;

use crate::core::{agent, config};
use crate::i18n;

#[derive(Subcommand)]
pub enum AgentCommands {
    /// List all registered agents and their status
    List,
    /// Register a custom agent manually
    Add {
        /// Agent ID (e.g. my-agent)
        id: String,
        /// Path to the agent's skills directory (e.g. ~/.my-agent/skills)
        path: String,
    },
}

pub fn run(cmd: AgentCommands) -> Result<()> {
    match cmd {
        AgentCommands::List => run_list(),
        AgentCommands::Add { id, path } => {
            config::add_agent_entry(&id, &path)?;
            println!(
                "{} {}: {}",
                style(i18n::t("ok")).green().bold(),
                i18n::t("agent added"),
                id
            );
            Ok(())
        }
    }
}

fn run_list() -> Result<()> {
    let detected = agent::detect_agents()?
        .into_iter()
        .map(|item| (item.id.clone(), item))
        .collect::<std::collections::HashMap<_, _>>();
    let configured = config::load_agents()?;

    if configured.agents.is_empty() {
        println!(
            "{} {}",
            style(i18n::t("info")).cyan().bold(),
            i18n::t("no agents registered — run `skm scan` to detect installed agents")
        );
        return Ok(());
    }

    for (id, raw_path) in configured.agents {
        let expanded = config::expand_tilde(&raw_path);
        if let Some(status) = detected.get(&id) {
            println!(
                "{}  {}  {}={}  {}={}",
                style(&id).green(),
                status.skills_dir,
                i18n::t("installed"),
                status.installed,
                i18n::t("skills"),
                status.skill_count
            );
        } else {
            let path = PathBuf::from(&expanded);
            let skill_count = count_skill_dirs(&path)?;
            let installed = path.exists() || skill_count > 0;
            println!(
                "{}  {}  {}={}  {}={}",
                style(&id).green(),
                expanded,
                i18n::t("installed"),
                installed,
                i18n::t("skills"),
                skill_count
            );
        }
    }

    Ok(())
}

fn count_skill_dirs(path: &PathBuf) -> Result<usize> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Ok(0),
    };

    let mut count = 0;
    for entry in entries {
        let entry = entry?;
        let candidate = entry.path();
        let metadata = match fs::metadata(&candidate) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.is_dir() && candidate.join("SKILL.md").exists() {
            count += 1;
        }
    }
    Ok(count)
}
