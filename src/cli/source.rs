use anyhow::Result;
use clap::Subcommand;
use console::style;

use crate::core::config;
use crate::i18n;

#[derive(Subcommand)]
pub enum SourceCommands {
    /// Add a new skill registry source
    Add {
        /// Display name for this source
        name: String,
        /// Registry URL (https://skills.sh or GitHub repo URL)
        url: String,
    },
    /// Remove a registry source by name
    Remove {
        /// Source name to remove
        name: String,
    },
    /// List all configured registry sources
    List,
}

pub fn run(cmd: SourceCommands) -> Result<()> {
    match cmd {
        SourceCommands::Add { name, url } => {
            config::add_source(&name, &url)?;
            println!(
                "{} {}: {name}",
                style(i18n::t("ok")).green().bold(),
                i18n::t("source added")
            );
        }
        SourceCommands::Remove { name } => {
            config::remove_source(&name)?;
            println!(
                "{} {}: {name}",
                style(i18n::t("ok")).green().bold(),
                i18n::t("source removed")
            );
        }
        SourceCommands::List => {
            let sources = config::load_sources()?;
            if sources.sources.is_empty() {
                println!(
                    "{} {}",
                    style(i18n::t("info")).cyan().bold(),
                    i18n::t("no sources configured")
                );
            } else {
                for source in sources.sources {
                    println!(
                        "{}  {}  {}={}  type={}",
                        style(source.name).green(),
                        source.url,
                        i18n::t("enabled"),
                        source.enabled,
                        source.source_type,
                    );
                }
            }
        }
    }
    Ok(())
}
