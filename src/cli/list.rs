use anyhow::{anyhow, Result};
use console::style;

use crate::core::skill;
use crate::i18n;

pub fn run() -> Result<()> {
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

pub fn run_info(name: &str) -> Result<()> {
    let detail =
        skill::read_skill_detail(name)?.ok_or_else(|| anyhow!("skill not found: {name}"))?;

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
