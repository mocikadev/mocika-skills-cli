use anyhow::{anyhow, bail, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::core::{agent, config, operations, registry};
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "Install a skill from registry or Git repository")]
pub struct InstallArgs {
    /// Skill name, owner/repo[:subpath], or full Git URL
    pub name: String,
    /// Link installed skill to agent(s): agent ID or "all"
    #[arg(long, value_name = "AGENT")]
    pub link_to: Option<String>,
}

pub fn run(args: InstallArgs) -> Result<()> {
    let (repo_url, skill_subpath) = resolve_install_source(&args.name)?;
    let target_agents = resolve_target_agents(args.link_to.as_deref())?;

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));
    if let Ok(template) = ProgressStyle::with_template("{spinner:.green} {wide_msg}") {
        spinner.set_style(template);
    }

    let summary = operations::install_skill_from_repo_with_progress(
        &repo_url,
        skill_subpath,
        &target_agents,
        |message, done, total| {
            spinner.set_message(i18n::fmt_progress(message, done, total));
        },
    )?;
    spinner.finish_and_clear();

    println!(
        "{} {} ({})",
        style(i18n::t("installed")).green().bold(),
        summary.display_name,
        summary.id
    );
    if !summary.installed_on.is_empty() {
        println!(
            "{}: {}",
            i18n::t("linked to"),
            summary.installed_on.join(", ")
        );
    }

    Ok(())
}

pub fn run_uninstall(name: &str) -> Result<()> {
    operations::delete_skill(name)?;
    println!("{} {name}", style(i18n::t("uninstalled")).yellow().bold());
    Ok(())
}

pub fn run_link(name: &str, agent_id: &str) -> Result<()> {
    operations::assign_skill(name, agent_id)?;
    println!(
        "{} {name} → {agent_id}",
        style(i18n::t("linked")).green().bold()
    );
    Ok(())
}

pub fn run_unlink(name: &str, agent_id: &str) -> Result<()> {
    operations::unassign_skill(name, agent_id)?;
    println!(
        "{} {}",
        style(i18n::t("unlinked")).yellow().bold(),
        i18n::fmt_unlinked_from(name, agent_id)
    );
    Ok(())
}

fn resolve_install_source(input: &str) -> Result<(String, Option<String>)> {
    if let Some((repo, subpath)) = parse_direct_repo_target(input) {
        return Ok((repo, subpath));
    }

    let sources = config::load_sources()?;
    let enabled_sources = sources
        .sources
        .into_iter()
        .filter(|entry| entry.enabled)
        .collect::<Vec<_>>();
    if enabled_sources.is_empty() {
        bail!("no enabled sources configured");
    }

    let query = input.trim();
    for source in enabled_sources {
        let matches = match source.source_type {
            config::SourceType::SkillsSh => registry::search_skills(&source.url, query, 20)?,
            config::SourceType::GitHub | config::SourceType::Git => {
                registry::search_git_source(&source.url, query, 20)?
            }
        };
        let best = matches
            .iter()
            .find(|item| {
                item.skill_id.eq_ignore_ascii_case(query) || item.name.eq_ignore_ascii_case(query)
            })
            .or_else(|| matches.first());

        if let Some(best) = best {
            return Ok((
                registry_source_to_repo_url(&best.source),
                Some(best.skill_id.clone()),
            ));
        }
    }

    Err(anyhow!("skill not found in configured sources: {query}"))
}

fn resolve_target_agents(link_to: Option<&str>) -> Result<Vec<String>> {
    let target = link_to.unwrap_or("all");
    if target.eq_ignore_ascii_case("all") {
        let mut ids = config::load_agents()?
            .agents
            .keys()
            .filter(|id| agent::is_agent_present(id))
            .cloned()
            .collect::<Vec<_>>();
        if ids.is_empty() {
            ids = agent::all_installed_agents()?
                .into_iter()
                .map(|item| item.id)
                .collect();
        }
        ids.sort();
        ids.dedup();
        Ok(ids)
    } else {
        Ok(vec![target.to_string()])
    }
}

fn parse_direct_repo_target(input: &str) -> Option<(String, Option<String>)> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.contains("://") || trimmed.starts_with("git@") {
        if let Some((repo, subpath)) = trimmed.split_once('#') {
            let normalized_repo = repo.trim().to_string();
            let normalized_subpath = subpath.trim();
            return Some((
                normalized_repo,
                if normalized_subpath.is_empty() {
                    None
                } else {
                    Some(normalized_subpath.to_string())
                },
            ));
        }

        if let Some(result) = parse_github_web_url(trimmed) {
            return Some(result);
        }

        return Some((trimmed.to_string(), None));
    }

    if let Some((repo, subpath)) = trimmed.split_once(':') {
        let repo = repo.trim();
        let subpath = subpath.trim();
        if repo.chars().filter(|ch| *ch == '/').count() == 1 {
            return Some((
                format!("https://github.com/{repo}.git"),
                if subpath.is_empty() {
                    None
                } else {
                    Some(subpath.to_string())
                },
            ));
        }
    }

    let slash_count = trimmed.chars().filter(|ch| *ch == '/').count();
    if slash_count == 1 {
        return Some((format!("https://github.com/{trimmed}.git"), None));
    }

    None
}

fn parse_github_web_url(url: &str) -> Option<(String, Option<String>)> {
    let lower = url.to_lowercase();
    let host_prefix = ["https://github.com/", "http://github.com/"]
        .iter()
        .find(|prefix| lower.starts_with(*prefix))?;

    let after_host = &url[host_prefix.len()..].trim_end_matches('/');
    let parts: Vec<&str> = after_host.splitn(5, '/').collect();

    let (owner, repo) = match parts.as_slice() {
        [owner, repo, ..] => (*owner, repo.trim_end_matches(".git")),
        _ => return None,
    };

    let repo_url = format!("https://github.com/{owner}/{repo}.git");

    let subpath = match parts.as_slice() {
        [_, _, "tree", _branch, path] => Some(path.trim_matches('/').to_string()),
        [_, _, subpath @ ..] if !subpath.is_empty() => {
            let joined = subpath.join("/").trim_matches('/').to_string();
            if joined.is_empty() {
                None
            } else {
                Some(joined)
            }
        }
        _ => None,
    };

    Some((repo_url, subpath))
}

fn registry_source_to_repo_url(source: &str) -> String {
    let trimmed = source.trim();
    if trimmed.contains("://") || trimmed.starts_with("git@") {
        trimmed.to_string()
    } else {
        format!("https://github.com/{trimmed}.git")
    }
}
