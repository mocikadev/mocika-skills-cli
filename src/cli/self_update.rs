use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::core::updater;
use crate::i18n;

#[derive(clap::Args)]
#[command(about = "Update skm itself to the latest release")]
pub struct SelfUpdateArgs {
    /// Only check for a newer version without downloading
    #[arg(long)]
    pub check: bool,
}

pub fn run(args: SelfUpdateArgs) -> Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap_or_else(|_| ProgressStyle::default_spinner()),
    );
    spinner.set_message(i18n::t("checking for updates"));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    match updater::check_update()? {
        None => {
            spinner.finish_and_clear();
            println!(
                "{} {}",
                style("✓").green().bold(),
                i18n::t("already up to date")
            );
        }
        Some(info) => {
            spinner.finish_and_clear();

            if args.check {
                println!(
                    "{} {}",
                    style(i18n::t("update available")).cyan().bold(),
                    i18n::fmt_update_available(&info.tag)
                );
            } else {
                let spinner2 = ProgressBar::new_spinner();
                spinner2.set_style(
                    ProgressStyle::default_spinner()
                        .template("{spinner:.cyan} {msg}")
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                spinner2.set_message(i18n::t("downloading update"));
                spinner2.enable_steady_tick(std::time::Duration::from_millis(80));

                let new_version = updater::apply_update(&info)?;

                spinner2.finish_and_clear();
                println!(
                    "{} {}",
                    style("✓").green().bold(),
                    i18n::fmt_updated_to(&new_version)
                );
            }
        }
    }

    Ok(())
}
