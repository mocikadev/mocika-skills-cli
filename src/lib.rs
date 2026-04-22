#![deny(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]

pub mod cli;
pub mod core;
pub mod error;
pub mod i18n;
pub mod models;

use anyhow::Result;
use clap::{CommandFactory, FromArgMatches};

pub fn run() -> Result<()> {
    let detected_lang = {
        use crate::core::skm_config;

        let from_config = skm_config::load_lang();
        from_config.unwrap_or_else(|| {
            let lang_env = std::env::var("LANG").unwrap_or_default();
            if lang_env.starts_with("zh") {
                i18n::Lang::Zh
            } else {
                i18n::Lang::En
            }
        })
    };
    i18n::init(detected_lang);

    let cmd = build_i18n_cmd();
    let matches = cmd.get_matches();
    let cli = cli::Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    match cli.command {
        cli::Commands::Install(args) => cli::install::run(args),
        cli::Commands::Search(args) => cli::search::run(args),
        cli::Commands::Scan(args) => cli::scan::run(args),
        cli::Commands::Relink(args) => cli::relink::run(args),
        cli::Commands::Update(args) => cli::update::run(args),
        cli::Commands::List => cli::list::run(),
        cli::Commands::Info { name } => cli::list::run_info(&name),
        cli::Commands::Uninstall { name } => cli::install::run_uninstall(&name),
        cli::Commands::Link { name, agent } => cli::install::run_link(&name, &agent),
        cli::Commands::Unlink { name, agent } => cli::install::run_unlink(&name, &agent),
        cli::Commands::Source(cmd) => cli::source::run(cmd),
        cli::Commands::Agent(cmd) => cli::agent_cmd::run(cmd),
        cli::Commands::Backup(cmd) => cli::backup::run(cmd),
        cli::Commands::Config(cmd) => cli::config_cmd::run(cmd),
        cli::Commands::SelfUpdate(args) => cli::self_update::run(args),
    }
}

fn build_i18n_cmd() -> clap::Command {
    use crate::i18n::t;

    fn tmpl() -> String {
        format!(
            "{{before-help}}{{about-with-newline}}\n{}: {{usage}}\n\n{{all-args}}{{after-help}}",
            t("heading.usage")
        )
    }

    fn h() -> clap::Arg {
        clap::Arg::new("help")
            .short('h')
            .long("help")
            .action(clap::ArgAction::Help)
            .help(t("flag.help"))
            .help_heading(t("heading.options"))
    }

    fn opt(a: clap::Arg, key: &str) -> clap::Arg {
        a.help(t(key)).help_heading(t("heading.options"))
    }

    fn pos(a: clap::Arg, key: &str) -> clap::Arg {
        a.help(t(key)).help_heading(t("heading.arguments"))
    }

    cli::Cli::command()
        .about(t("cmd.skm"))
        .help_template(tmpl())
        .subcommand_help_heading(t("heading.commands"))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .arg(h())
        .arg(
            clap::Arg::new("version")
                .short('V')
                .long("version")
                .action(clap::ArgAction::Version)
                .help(t("flag.version"))
                .help_heading(t("heading.options")),
        )
        .mut_subcommand("install", |c| {
            c.about(t("cmd.install"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("name", |a| pos(a, "arg.install.name"))
                .mut_arg("link_to", |a| opt(a, "arg.install.link-to"))
        })
        .mut_subcommand("search", |c| {
            c.about(t("cmd.search"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("keyword", |a| pos(a, "arg.search.keyword"))
                .mut_arg("limit", |a| opt(a, "arg.search.limit"))
        })
        .mut_subcommand("scan", |c| {
            c.about(t("cmd.scan"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("dry_run", |a| opt(a, "arg.scan.dry-run"))
        })
        .mut_subcommand("relink", |c| {
            c.about(t("cmd.relink"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("agent", |a| pos(a, "arg.relink.agent"))
                .mut_arg("skill", |a| opt(a, "arg.relink.skill"))
                .mut_arg("force", |a| opt(a, "arg.relink.force"))
                .mut_arg("backup", |a| opt(a, "arg.relink.backup"))
                .mut_arg("dry_run", |a| opt(a, "arg.relink.dry-run"))
        })
        .mut_subcommand("update", |c| {
            c.about(t("cmd.update"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("name", |a| pos(a, "arg.update.name"))
                .mut_arg("check", |a| opt(a, "arg.update.check"))
        })
        .mut_subcommand("list", |c| {
            c.about(t("cmd.list"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
        })
        .mut_subcommand("info", |c| {
            c.about(t("cmd.info"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("name", |a| pos(a, "arg.info.name"))
        })
        .mut_subcommand("uninstall", |c| {
            c.about(t("cmd.uninstall"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("name", |a| pos(a, "arg.uninstall.name"))
        })
        .mut_subcommand("link", |c| {
            c.about(t("cmd.link"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("name", |a| pos(a, "arg.link.name"))
                .mut_arg("agent", |a| pos(a, "arg.link.agent"))
        })
        .mut_subcommand("unlink", |c| {
            c.about(t("cmd.unlink"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("name", |a| pos(a, "arg.unlink.name"))
                .mut_arg("agent", |a| pos(a, "arg.unlink.agent"))
        })
        .mut_subcommand("source", |c| {
            c.about(t("cmd.source"))
                .help_template(tmpl())
                .subcommand_help_heading(t("heading.commands"))
                .disable_help_flag(true)
                .arg(h())
                .mut_subcommand("add", |c| {
                    c.about(t("cmd.source.add"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("name", |a| pos(a, "arg.source.add.name"))
                        .mut_arg("url", |a| pos(a, "arg.source.add.url"))
                })
                .mut_subcommand("remove", |c| {
                    c.about(t("cmd.source.remove"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("name", |a| pos(a, "arg.source.remove.name"))
                })
                .mut_subcommand("list", |c| {
                    c.about(t("cmd.source.list"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                })
        })
        .mut_subcommand("agent", |c| {
            c.about(t("cmd.agent"))
                .help_template(tmpl())
                .subcommand_help_heading(t("heading.commands"))
                .disable_help_flag(true)
                .arg(h())
                .mut_subcommand("list", |c| {
                    c.about(t("cmd.agent.list"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                })
                .mut_subcommand("add", |c| {
                    c.about(t("cmd.agent.add"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("id", |a| pos(a, "arg.agent.add.id"))
                        .mut_arg("path", |a| pos(a, "arg.agent.add.path"))
                })
        })
        .mut_subcommand("backup", |c| {
            c.about(t("cmd.backup"))
                .help_template(tmpl())
                .subcommand_help_heading(t("heading.commands"))
                .disable_help_flag(true)
                .arg(h())
                .mut_subcommand("list", |c| {
                    c.about(t("cmd.backup.list"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("name", |a| pos(a, "arg.backup.list.name"))
                })
                .mut_subcommand("restore", |c| {
                    c.about(t("cmd.backup.restore"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("name", |a| pos(a, "arg.backup.restore.name"))
                        .mut_arg("snapshot_id", |a| pos(a, "arg.backup.restore.snapshot-id"))
                })
                .mut_subcommand("delete", |c| {
                    c.about(t("cmd.backup.delete"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("name", |a| pos(a, "arg.backup.delete.name"))
                        .mut_arg("snapshot_id", |a| pos(a, "arg.backup.delete.snapshot-id"))
                })
        })
        .mut_subcommand("config", |c| {
            c.about(t("cmd.config"))
                .help_template(tmpl())
                .subcommand_help_heading(t("heading.commands"))
                .disable_help_flag(true)
                .arg(h())
                .mut_subcommand("lang", |c| {
                    c.about(t("cmd.config.lang"))
                        .help_template(tmpl())
                        .disable_help_flag(true)
                        .arg(h())
                        .mut_arg("lang", |a| pos(a, "arg.config.lang.lang"))
                        .mut_arg("reset", |a| opt(a, "arg.config.lang.reset"))
                })
        })
        .mut_subcommand("self-update", |c| {
            c.about(t("cmd.self-update"))
                .help_template(tmpl())
                .disable_help_flag(true)
                .arg(h())
                .mut_arg("check", |a| opt(a, "arg.self-update.check"))
        })
}
