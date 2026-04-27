use std::collections::BTreeMap;

use anyhow::Result;
use console::style;

use crate::core::{config, registry};
use crate::i18n;
use crate::models::RegistrySkill;

#[derive(clap::Args)]
#[command(about = "Search skills in configured registries")]
pub struct SearchArgs {
    /// Keyword to search for
    pub keyword: String,
    /// Maximum number of results to show
    #[arg(long, default_value = "20", value_name = "N")]
    pub limit: usize,
}

pub fn run(args: SearchArgs) -> Result<()> {
    let sources = config::load_sources()?;
    let mut merged = BTreeMap::<String, RegistrySkill>::new();

    for source in sources.sources.into_iter().filter(|entry| entry.enabled) {
        let skills = match source.source_type {
            config::SourceType::SkillsSh => {
                registry::search_skills(&source.url, &args.keyword, args.limit)?
            }
            config::SourceType::GitHub | config::SourceType::Git => {
                registry::search_git_source(&source.url, &args.keyword, args.limit)?
            }
            config::SourceType::Local => {
                registry::search_local_source(&source.url, &args.keyword, args.limit)?
            }
        };
        for skill in skills {
            merged.entry(skill.id.clone()).or_insert(skill);
        }
    }

    if merged.is_empty() {
        println!(
            "{} {}",
            style(i18n::t("info")).cyan().bold(),
            i18n::t("no skills found")
        );
        return Ok(());
    }

    let mut results = merged.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .installs
            .cmp(&left.installs)
            .then_with(|| left.name.cmp(&right.name))
    });

    for skill in results.into_iter().take(args.limit) {
        let description = skill.description.unwrap_or_else(|| "-".to_string());
        println!(
            "{}  {}  {}",
            style(skill.name).green(),
            skill.installs,
            description
        );
    }

    Ok(())
}
