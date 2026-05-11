use std::path::PathBuf;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::install;
use crate::core::{bundle, lock, operations};
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "Install skills from a bundle file")]
pub struct ImportArgs {
    pub file: String,
    #[arg(long, value_name = "AGENT")]
    pub link_to: Option<String>,
    #[arg(long)]
    pub force: bool,
}

pub fn run(args: ImportArgs) -> Result<()> {
    let path = PathBuf::from(&args.file);
    let b = bundle::read_bundle(&path)?;

    if b.skills.is_empty() {
        println!("{}", i18n::t("import.empty_bundle"));
        return Ok(());
    }

    let target_agents = install::resolve_target_agents(args.link_to.as_deref())?;

    let mut installed_count = 0usize;
    let mut skipped_count = 0usize;
    let mut failed_count = 0usize;

    for entry in &b.skills {
        let already_installed = lock::get_skill_entry(&entry.name)
            .ok()
            .and_then(|v| v)
            .is_some();

        if already_installed && !args.force {
            println!(
                "  {}  {:<30}  {}",
                style("·").dim(),
                entry.name,
                style(i18n::t("import.skipped")).dim()
            );
            skipped_count += 1;
            continue;
        }

        let Some((repo_url, skill_subpath)) = install::parse_direct_repo_target(&entry.source)
        else {
            println!(
                "  {}  {:<30}  {}",
                style("✗").red(),
                entry.name,
                style(i18n::t("import.failed_parse")).red()
            );
            failed_count += 1;
            continue;
        };

        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));
        if let Ok(template) = ProgressStyle::with_template("{spinner:.green} {wide_msg}") {
            spinner.set_style(template);
        }
        spinner.set_message(format!("{} {}", i18n::t("import.installing"), entry.name));

        let skill_name = entry.name.clone();
        let result = if repo_url.starts_with("local://") {
            let local_path = repo_url.trim_start_matches("local://").to_string();
            operations::install_skill_from_local_with_progress(
                &local_path,
                skill_subpath,
                &target_agents,
                |msg, done, total| {
                    spinner.set_message(format!(
                        "{} {}",
                        skill_name,
                        i18n::fmt_progress(msg, done, total)
                    ));
                },
            )
        } else {
            operations::install_skill_from_repo_with_progress(
                &repo_url,
                skill_subpath,
                &target_agents,
                |msg, done, total| {
                    spinner.set_message(format!(
                        "{} {}",
                        skill_name,
                        i18n::fmt_progress(msg, done, total)
                    ));
                },
            )
        };

        spinner.finish_and_clear();

        match result {
            Ok(summary) => {
                println!(
                    "  {}  {} ({})",
                    style("✓").green(),
                    summary.display_name,
                    summary.id
                );
                installed_count += 1;
            }
            Err(e) => {
                println!(
                    "  {}  {:<30}  {}",
                    style("✗").red(),
                    entry.name,
                    style(e.to_string()).red()
                );
                failed_count += 1;
            }
        }
    }

    println!();
    println!(
        "  {} {}",
        style("→").dim(),
        i18n::fmt_import_summary(installed_count, skipped_count, failed_count)
    );

    Ok(())
}
