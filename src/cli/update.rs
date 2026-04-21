use anyhow::{anyhow, Result};
use console::style;

use crate::core::update;
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "Update installed skills to latest version")]
pub struct UpdateArgs {
    /// Skill name to update (omit to update all)
    pub name: Option<String>,
    /// Only check for updates, do not apply them
    #[arg(long)]
    pub check: bool,
}

pub fn run(args: UpdateArgs) -> Result<()> {
    if args.check {
        let ids = args.name.into_iter().collect::<Vec<_>>();
        for item in update::check_updates(&ids)? {
            println!(
                "{} {} {}",
                style(item.skill_id).green(),
                i18n::t(&item.status),
                item.message.unwrap_or_default()
            );
        }
        return Ok(());
    }

    let targets = match args.name {
        Some(name) => vec![name],
        None => update::installed_skill_ids()?,
    };

    let mut errors = Vec::new();
    for target in targets {
        match crate::core::operations::update_skill(&target) {
            Ok(summary) => println!(
                "{} {}",
                style(i18n::t("updated")).green().bold(),
                summary.id
            ),
            Err(error) => errors.push(format!("{target}: {error}")),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(anyhow!(errors.join("\n")))
    }
}
