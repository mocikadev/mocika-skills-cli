use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

const SOURCES_FILE_NAME: &str = "sources.toml";
const AGENTS_FILE_NAME: &str = "agents.toml";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SourceType {
    #[serde(rename = "skills_sh")]
    #[default]
    SkillsSh,
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "git")]
    Git,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::SkillsSh => write!(f, "skills.sh"),
            SourceType::GitHub => write!(f, "github"),
            SourceType::Git => write!(f, "git"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceEntry {
    pub name: String,
    pub url: String,
    pub enabled: bool,
    #[serde(default)]
    pub source_type: SourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcesConfig {
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    pub agents: HashMap<String, String>,
}

pub fn load_sources() -> Result<SourcesConfig> {
    let path = sources_file_path()?;
    if !path.exists() {
        return Ok(default_sources());
    }

    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(default_sources());
    }

    let config: SourcesConfig = toml::from_str(&content)?;
    if config.sources.is_empty() {
        Ok(default_sources())
    } else {
        Ok(config)
    }
}

pub fn save_sources(config: &SourcesConfig) -> Result<()> {
    let payload = toml::to_string_pretty(config)?;
    atomic_write(&sources_file_path()?, &payload)
}

pub fn add_source(name: &str, url: &str) -> Result<()> {
    let mut config = load_sources()?;
    let normalized_name = name.trim();
    let normalized_url = url.trim().trim_end_matches('/');
    if normalized_name.is_empty() {
        return Err(anyhow!("source name cannot be empty"));
    }
    if normalized_url.is_empty() {
        return Err(anyhow!("source URL cannot be empty"));
    }

    let source_type = detect_source_type(normalized_url);

    if let Some(entry) = config
        .sources
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(normalized_name))
    {
        entry.name = normalized_name.to_string();
        entry.url = normalized_url.to_string();
        entry.enabled = true;
        entry.source_type = source_type;
    } else {
        config.sources.push(SourceEntry {
            name: normalized_name.to_string(),
            url: normalized_url.to_string(),
            enabled: true,
            source_type,
        });
    }

    save_sources(&config)
}

pub fn remove_source(name: &str) -> Result<()> {
    let mut config = load_sources()?;
    let before = config.sources.len();
    config
        .sources
        .retain(|entry| !entry.name.eq_ignore_ascii_case(name.trim()));
    if before == config.sources.len() {
        return Err(anyhow!("source not found: {}", name.trim()));
    }
    save_sources(&config)
}

pub fn load_agents() -> Result<AgentsConfig> {
    let path = agents_file_path()?;
    if !path.exists() {
        return Ok(AgentsConfig::default());
    }

    let content = fs::read_to_string(path)?;
    if content.trim().is_empty() {
        return Ok(AgentsConfig::default());
    }

    Ok(toml::from_str(&content)?)
}

pub fn save_agents(config: &AgentsConfig) -> Result<()> {
    let ordered = config
        .agents
        .iter()
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<BTreeMap<String, String>>();
    let payload = toml::to_string_pretty(&AgentsConfigFile { agents: ordered })?;
    atomic_write(&agents_file_path()?, &payload)
}

pub fn remove_agent_entry(id: &str) -> Result<()> {
    let normalized_id = id.trim();
    if normalized_id.is_empty() {
        return Err(anyhow!("agent id cannot be empty"));
    }
    let mut config = load_agents()?;
    config.agents.remove(normalized_id);
    save_agents(&config)
}

pub fn add_agent_entry(id: &str, path: &str) -> Result<()> {
    let normalized_id = id.trim();
    let normalized_path = path.trim();
    if normalized_id.is_empty() {
        return Err(anyhow!("agent id cannot be empty"));
    }
    if normalized_path.is_empty() {
        return Err(anyhow!("agent path cannot be empty"));
    }

    let mut config = load_agents()?;
    let stored = compact_home_path(Path::new(normalized_path));
    config.agents.insert(normalized_id.to_string(), stored);
    save_agents(&config)
}

pub fn agents_dir_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
    Ok(home.join(".agents"))
}

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home
                .join(path.strip_prefix("~/").unwrap_or(path))
                .to_string_lossy()
                .to_string();
        }
    }
    path.to_string()
}

pub fn compact_home_path(path: &Path) -> String {
    let raw = path.to_string_lossy().to_string();
    if let Some(home) = dirs::home_dir() {
        let home_text = home.to_string_lossy().to_string();
        if raw == home_text {
            return "~".to_string();
        }
        if let Some(stripped) = raw.strip_prefix(&(home_text + "/")) {
            return format!("~/{}", stripped);
        }
    }
    raw
}

fn default_sources() -> SourcesConfig {
    SourcesConfig {
        sources: vec![SourceEntry {
            name: "skills.sh".to_string(),
            url: "https://skills.sh".to_string(),
            enabled: true,
            source_type: SourceType::SkillsSh,
        }],
    }
}

fn detect_source_type(url: &str) -> SourceType {
    if url.to_lowercase().contains("skills.sh") {
        return SourceType::SkillsSh;
    }

    let lower = url.to_lowercase();
    if lower.starts_with("https://github.com/")
        || lower.starts_with("http://github.com/")
        || lower.starts_with("git@github.com:")
        || lower.starts_with("ssh://git@github.com/")
        || lower.starts_with("git://github.com/")
    {
        return SourceType::GitHub;
    }

    if url.contains("://") || url.starts_with("git@") {
        return SourceType::Git;
    }

    let is_owner_repo_shorthand = url.chars().filter(|ch| *ch == '/').count() == 1;
    if is_owner_repo_shorthand {
        return SourceType::GitHub;
    }

    SourceType::SkillsSh
}

fn sources_file_path() -> Result<PathBuf> {
    Ok(agents_dir_path()?.join(SOURCES_FILE_NAME))
}

fn agents_file_path() -> Result<PathBuf> {
    Ok(agents_dir_path()?.join(AGENTS_FILE_NAME))
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

#[derive(Debug, Serialize)]
struct AgentsConfigFile {
    agents: BTreeMap<String, String>,
}
