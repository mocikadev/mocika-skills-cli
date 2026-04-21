use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::core::config::agents_dir_path;
use crate::i18n::Lang;

const SKM_CONFIG_FILE: &str = "skm.toml";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SkmConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

pub fn load() -> Result<SkmConfig> {
    let path = skm_config_file_path()?;
    if !path.exists() {
        return Ok(SkmConfig::default());
    }

    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(SkmConfig::default());
    }

    Ok(toml::from_str(&content)?)
}

pub fn save(config: &SkmConfig) -> Result<()> {
    let payload = toml::to_string_pretty(config)?;
    atomic_write(&skm_config_file_path()?, &payload)
}

pub fn load_lang() -> Option<Lang> {
    load().ok()?.lang.as_deref().and_then(Lang::from_code)
}

pub fn set_lang(lang: &Lang) -> Result<()> {
    let mut config = load()?;
    config.lang = Some(lang.code().to_string());
    save(&config)
}

fn skm_config_file_path() -> Result<PathBuf> {
    Ok(agents_dir_path()?.join(SKM_CONFIG_FILE))
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)?;
    Ok(())
}
