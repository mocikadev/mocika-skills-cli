use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::core::{skill, update};
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "List all installed skills")]
pub struct ListArgs {
    /// Only show skills with available updates
    #[arg(long)]
    pub outdated: bool,
}

pub fn run(args: ListArgs) -> Result<()> {
    if args.outdated {
        return run_outdated();
    }

    let skills = skill::scan_skills()?;
    if skills.is_empty() {
        println!(
            "{} {}",
            style(i18n::t("info")).cyan().bold(),
            i18n::t("no installed skills")
        );
        return Ok(());
    }

    for item in skills {
        let installed_on = if item.installed_on.is_empty() {
            "-".to_string()
        } else {
            item.installed_on.join(", ")
        };
        println!(
            "{}  {}  {}",
            style(item.id).green(),
            item.display_name,
            installed_on
        );
    }
    Ok(())
}

fn run_outdated() -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(i18n::t("checking for updates"));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let checks = update::check_updates(&[])?;
    spinner.finish_and_clear();

    let outdated: Vec<_> = checks.into_iter().filter(|c| c.has_update).collect();

    if outdated.is_empty() {
        println!(
            "{} {}",
            style(i18n::t("info")).cyan().bold(),
            i18n::t("no outdated skills")
        );
        return Ok(());
    }

    for item in outdated {
        println!(
            "{}  {}",
            style(item.skill_id).green(),
            item.message.unwrap_or_default()
        );
    }
    Ok(())
}

pub fn run_info(name: &str) -> Result<()> {
    let detail =
        skill::read_skill_detail(name)?.ok_or_else(|| anyhow::anyhow!("skill not found: {name}"))?;

    println!("{}", style(&detail.summary.display_name).green().bold());
    println!("{}: {}", i18n::t("id"), detail.summary.id);
    println!("{}: {}", i18n::t("path"), detail.summary.canonical_path);
    println!("{}: {}", i18n::t("scope"), detail.scope);
    println!(
        "{}: {}",
        i18n::t("installed_on"),
        if detail.summary.installed_on.is_empty() {
            "-".to_string()
        } else {
            detail.summary.installed_on.join(", ")
        }
    );
    println!(
        "{}: {}",
        i18n::t("frontmatter"),
        serde_json::to_string_pretty(&detail.frontmatter)?
    );
    if let Some(lock_entry) = detail.lock_entry {
        println!(
            "{}: {}",
            i18n::t("lock"),
            serde_json::to_string_pretty(&lock_entry)?
        );
    }
    Ok(())
}
