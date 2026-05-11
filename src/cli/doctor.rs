use anyhow::Result;
use console::style;

use crate::core::{agent, config, lock, operations, skill};
use crate::i18n;
use crate::models::LinkState;

#[derive(clap::Args)]
#[command(about = "Check environment and diagnose common issues")]
pub struct DoctorArgs {
    #[command(subcommand)]
    pub command: Option<DoctorCommand>,
}

#[derive(clap::Subcommand)]
pub enum DoctorCommand {
    #[command(about = "Scan and fix SKILL.md frontmatter issues in installed skills")]
    FixSkills(FixSkillsArgs),
    #[command(about = "Sync symlinks: remove broken links and create missing ones")]
    Sync(SyncArgs),
}

#[derive(clap::Args)]
pub struct FixSkillsArgs {
    #[arg(long, help = "Preview changes without applying them")]
    pub dry_run: bool,
}

#[derive(clap::Args)]
pub struct SyncArgs {
    #[arg(
        long,
        help = "Only process this agent (default: all registered agents)"
    )]
    pub agent: Option<String>,
    #[arg(long, help = "Preview changes without making them")]
    pub dry_run: bool,
    #[arg(long, help = "Overwrite conflicting regular files/directories")]
    pub force: bool,
}

pub fn run(args: DoctorArgs) -> Result<()> {
    match args.command {
        Some(DoctorCommand::FixSkills(fix_args)) => run_fix_skills(fix_args),
        Some(DoctorCommand::Sync(sync_args)) => run_sync(sync_args),
        None => run_checks(),
    }
}

fn run_checks() -> Result<()> {
    let mut issues = 0usize;

    issues += check_env();
    issues += check_agents()?;
    issues += check_links()?;
    issues += check_integrity()?;

    println!();
    if issues == 0 {
        println!(
            "{} {}",
            style("✓").green().bold(),
            i18n::fmt_doctor_summary(0)
        );
    } else {
        println!(
            "{} {}",
            style("✗").red().bold(),
            i18n::fmt_doctor_summary(issues)
        );
        std::process::exit(1);
    }

    Ok(())
}

fn run_sync(args: SyncArgs) -> Result<()> {
    let header = if args.dry_run {
        format!(
            "{} {}",
            i18n::t("doctor.sync"),
            i18n::t("doctor.sync.dry_run")
        )
    } else {
        i18n::t("doctor.sync").to_string()
    };
    println!("{}", style(header).cyan().bold());

    let agent_id = args.agent.as_deref();
    let result = operations::sync_agent_links(agent_id, args.force, args.dry_run)?;

    println!();

    for (agent, skill) in &result.fixed_items {
        println!(
            "  {}  {:<30}  {:<16}  {}",
            style("✓").green(),
            style(skill).bold(),
            style(agent).dim(),
            style(i18n::t("doctor.sync.item.fixed")).yellow()
        );
    }
    for (agent, skill) in &result.linked_items {
        println!(
            "  {}  {:<30}  {:<16}  {}",
            style("✓").green(),
            style(skill).bold(),
            style(agent).dim(),
            style(i18n::t("doctor.sync.item.linked")).green()
        );
    }
    for (agent, skill) in &result.conflict_items {
        println!(
            "  {}  {:<30}  {:<16}  {}",
            style("!").yellow().bold(),
            style(skill).bold(),
            style(agent).dim(),
            style(i18n::t("doctor.sync.item.conflict")).yellow()
        );
    }
    for err in &result.errors {
        println!("  {} {}", style("✗").red(), style(err).dim());
    }

    println!();
    println!(
        "  {} {}",
        style("→").dim(),
        i18n::fmt_sync_summary(result.fixed, result.linked, result.conflicts, result.ok)
    );

    if result.conflicts > 0 && !args.force {
        println!(
            "  {} {}",
            style("!").yellow().bold(),
            i18n::t("doctor.sync.conflict_hint")
        );
    }

    Ok(())
}

fn run_fix_skills(args: FixSkillsArgs) -> Result<()> {
    let header = if args.dry_run {
        format!(
            "{} {}",
            i18n::t("doctor.fix_skills"),
            i18n::t("doctor.fix_skills.dry_run")
        )
    } else {
        i18n::t("doctor.fix_skills").to_string()
    };
    println!("{}", style(header).cyan().bold());

    let lock_entries = lock::list_skill_entries().unwrap_or_default();
    if lock_entries.is_empty() {
        println!(
            "  {} {}",
            style("·").dim(),
            i18n::t("doctor.fix_skills.no_skills")
        );
        return Ok(());
    }

    let mut checked = 0usize;
    let mut fixed_count = 0usize;
    let mut warned_count = 0usize;

    for (skill_id, _) in &lock_entries {
        checked += 1;
        let result = skill::fix_skill_frontmatter(skill_id, args.dry_run)?;

        if !result.warnings.is_empty() {
            warned_count += 1;
            for warn in &result.warnings {
                print_check_with_hint(false, skill_id, i18n::t("doctor.fix_skills.warn"), warn);
            }
        } else if !result.fixed.is_empty() {
            fixed_count += 1;
            let fixes_str = result.fixed.join(", ");
            print_check_with_hint(
                true,
                skill_id,
                i18n::t("doctor.fix_skills.fixed"),
                &fixes_str,
            );
        } else {
            print_check(true, skill_id, i18n::t("doctor.fix_skills.ok"));
        }
    }

    println!();
    println!(
        "  {} {}",
        style("→").dim(),
        i18n::fmt_fix_skills_summary(checked, fixed_count, warned_count)
    );

    Ok(())
}

fn check_env() -> usize {
    let mut issues = 0usize;
    println!("{}", style(i18n::t("doctor.env")).cyan().bold());

    let shared_ok = agent::shared_skills_dir()
        .map(|dir| dir.exists())
        .unwrap_or(false);
    print_check(
        shared_ok,
        i18n::t("doctor.shared_dir"),
        i18n::t(if shared_ok {
            "doctor.exists"
        } else {
            "doctor.missing"
        }),
    );
    if !shared_ok {
        issues += 1;
    }

    let lock_ok = lock::read_json().is_ok();
    print_check(
        lock_ok,
        i18n::t("doctor.lock_file"),
        i18n::t(if lock_ok {
            "doctor.readable"
        } else {
            "doctor.unreadable"
        }),
    );
    if !lock_ok {
        issues += 1;
    }

    let agents_toml_ok = config::load_agents().is_ok();
    print_check(
        agents_toml_ok,
        i18n::t("doctor.agents_toml"),
        i18n::t(if agents_toml_ok {
            "doctor.readable"
        } else {
            "doctor.unreadable"
        }),
    );
    if !agents_toml_ok {
        issues += 1;
    }

    issues
}

fn check_agents() -> Result<usize> {
    let mut issues = 0usize;
    let agents_config = config::load_agents().unwrap_or_default();
    let agent_dirs = agent::agent_skills_dirs_from_config(&agents_config);

    println!();
    println!(
        "{}",
        style(format!(
            "{} ({})",
            i18n::t("doctor.agents"),
            agent_dirs.len()
        ))
        .cyan()
        .bold()
    );

    if agent_dirs.is_empty() {
        println!("  {} {}", style("·").dim(), i18n::t("doctor.no_agents"));
        return Ok(0);
    }

    for (agent_id, _skills_dir) in &agent_dirs {
        let present = agent::is_agent_present(agent_id);
        if present {
            print_check(true, agent_id, i18n::t("doctor.installed"));
        } else {
            print_check_with_hint(
                false,
                agent_id,
                i18n::t("doctor.not_installed"),
                i18n::t("doctor.stale_hint"),
            );
            issues += 1;
        }
    }

    Ok(issues)
}

fn check_links() -> Result<usize> {
    let mut issues = 0usize;
    let agents_config = config::load_agents().unwrap_or_default();
    let agent_dirs = agent::agent_skills_dirs_from_config(&agents_config);

    if agent_dirs.is_empty() {
        return Ok(0);
    }

    let lock_entries = lock::list_skill_entries().unwrap_or_default();
    if lock_entries.is_empty() {
        return Ok(0);
    }

    let mut link_issues = 0usize;
    let mut link_lines: Vec<(bool, String)> = Vec::new();

    for (skill_id, _) in &lock_entries {
        for (agent_id, agent_skills_dir) in &agent_dirs {
            let state = skill::check_link_state(skill_id, agent_skills_dir);
            let ok = state == LinkState::Linked;
            let state_label = match state {
                LinkState::Linked => i18n::t("doctor.linked"),
                LinkState::NotLinked => i18n::t("doctor.not_linked"),
                LinkState::Conflict => i18n::t("doctor.conflict"),
            };
            let line = format!(
                "  {}  {:<30}  {:<15}  {}",
                if ok {
                    style("✓").green().to_string()
                } else {
                    style("✗").red().to_string()
                },
                skill_id,
                agent_id,
                style(state_label).color256(if ok { 2 } else { 9 }),
            );
            if !ok {
                link_issues += 1;
            }
            link_lines.push((ok, line));
        }
    }

    println!();
    println!(
        "{}",
        style(format!(
            "{} ({} {})",
            i18n::t("doctor.links"),
            link_issues,
            i18n::t("doctor.issues")
        ))
        .cyan()
        .bold()
    );

    for (_, line) in link_lines {
        println!("{line}");
    }

    issues += link_issues;
    Ok(issues)
}

fn print_check(ok: bool, label: &str, status: &str) {
    println!(
        "  {}  {:<30}  {}",
        if ok {
            style("✓").green().to_string()
        } else {
            style("✗").red().to_string()
        },
        label,
        style(status).color256(if ok { 2 } else { 9 })
    );
}

fn print_check_with_hint(ok: bool, label: &str, status: &str, hint: &str) {
    println!(
        "  {}  {:<30}  {}  ({})",
        if ok {
            style("✓").green().to_string()
        } else {
            style("✗").red().to_string()
        },
        label,
        style(status).color256(if ok { 2 } else { 9 }),
        style(hint).dim()
    );
}

fn check_integrity() -> Result<usize> {
    let lock_entries = lock::list_skill_entries().unwrap_or_default();
    if lock_entries.is_empty() {
        return Ok(0);
    }

    let mut issues = 0usize;
    let mut lines: Vec<(bool, String)> = Vec::new();

    for (skill_id, _) in &lock_entries {
        match operations::skill_hash_check(skill_id) {
            Ok(Some((stored, current))) => {
                let ok = stored == current;
                if !ok {
                    issues += 1;
                }
                lines.push((
                    ok,
                    format!(
                        "  {}  {:<30}  {}",
                        if ok {
                            style("✓").green().to_string()
                        } else {
                            style("✗").red().to_string()
                        },
                        skill_id,
                        style(if ok {
                            i18n::t("doctor.intact")
                        } else {
                            i18n::t("doctor.modified")
                        })
                        .color256(if ok { 2 } else { 9 }),
                    ),
                ));
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }

    if lines.is_empty() {
        return Ok(0);
    }

    println!();
    println!(
        "{}",
        style(format!(
            "{} ({} {})",
            i18n::t("doctor.integrity"),
            issues,
            i18n::t("doctor.issues")
        ))
        .cyan()
        .bold()
    );

    for (_, line) in lines {
        println!("{line}");
    }

    Ok(issues)
}
