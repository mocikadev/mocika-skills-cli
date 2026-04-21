use anyhow::{anyhow, Result};
use clap::Subcommand;

use crate::core::skm_config;
use crate::i18n::{self, Lang};

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show or set the UI language
    Lang(LangArgs),
}

#[derive(clap::Args)]
pub struct LangArgs {
    /// Language code (en or zh)
    pub lang: Option<String>,
    /// Reset language to auto-detect from environment
    #[arg(long, conflicts_with = "lang")]
    pub reset: bool,
}

pub fn run(cmd: ConfigCommands) -> Result<()> {
    match cmd {
        ConfigCommands::Lang(args) => run_lang(args),
    }
}

fn run_lang(args: LangArgs) -> Result<()> {
    if args.reset {
        let mut config = skm_config::load()?;
        config.lang = None;
        skm_config::save(&config)?;
        println!("{}", i18n::fmt_lang_reset_to_auto_detect());
        return Ok(());
    }

    if let Some(lang_code) = args.lang.as_deref() {
        let lang =
            Lang::from_code(lang_code).ok_or_else(|| anyhow!(i18n::fmt_invalid_lang(lang_code)))?;
        skm_config::set_lang(&lang)?;
        println!("{}", i18n::fmt_lang_set(lang.code()));
        return Ok(());
    }

    let configured_lang = skm_config::load()?
        .lang
        .and_then(|value| Lang::from_code(&value));
    let active_lang = configured_lang.unwrap_or(*i18n::current());
    println!(
        "{}",
        i18n::fmt_lang_status(active_lang.code(), configured_lang.is_some())
    );
    Ok(())
}
