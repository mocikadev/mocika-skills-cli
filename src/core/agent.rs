use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Result};

use super::config::{expand_tilde, AgentsConfig};
use crate::models::AgentStatus;

#[derive(Debug, Clone)]
pub struct AgentDefinition {
    pub id: &'static str,
    pub display_name: &'static str,
    pub detect_command: &'static str,
}

pub fn definitions() -> Vec<AgentDefinition> {
    vec![
        AgentDefinition {
            id: "claude-code",
            display_name: "Claude Code",
            detect_command: "claude",
        },
        AgentDefinition {
            id: "codex",
            display_name: "Codex",
            detect_command: "codex",
        },
        AgentDefinition {
            id: "gemini-cli",
            display_name: "Gemini CLI",
            detect_command: "gemini",
        },
        AgentDefinition {
            id: "copilot-cli",
            display_name: "Copilot CLI",
            detect_command: "gh",
        },
        AgentDefinition {
            id: "opencode",
            display_name: "OpenCode",
            detect_command: "opencode",
        },
        AgentDefinition {
            id: "antigravity",
            display_name: "Antigravity",
            detect_command: "antigravity",
        },
        AgentDefinition {
            id: "cursor",
            display_name: "Cursor",
            detect_command: "cursor",
        },
        AgentDefinition {
            id: "kiro",
            display_name: "Kiro",
            detect_command: "kiro",
        },
        AgentDefinition {
            id: "codebuddy",
            display_name: "CodeBuddy",
            detect_command: "codebuddy",
        },
        AgentDefinition {
            id: "openclaw",
            display_name: "OpenClaw",
            detect_command: "openclaw",
        },
        AgentDefinition {
            id: "trae",
            display_name: "Trae",
            detect_command: "trae",
        },
        AgentDefinition {
            id: "junie",
            display_name: "Junie",
            detect_command: "junie",
        },
        AgentDefinition {
            id: "qoder",
            display_name: "Qoder",
            detect_command: "qoder",
        },
        AgentDefinition {
            id: "trae-cn",
            display_name: "Trae CN",
            detect_command: "trae-cn",
        },
        AgentDefinition {
            id: "windsurf",
            display_name: "Windsurf",
            detect_command: "windsurf",
        },
        AgentDefinition {
            id: "augment",
            display_name: "Augment",
            detect_command: "augment",
        },
        AgentDefinition {
            id: "kilocode",
            display_name: "KiloCode",
            detect_command: "kilocode",
        },
        AgentDefinition {
            id: "ob1",
            display_name: "OB1",
            detect_command: "ob1",
        },
        AgentDefinition {
            id: "amp",
            display_name: "Amp",
            detect_command: "amp",
        },
        AgentDefinition {
            id: "hermes",
            display_name: "Hermes",
            detect_command: "hermes",
        },
        AgentDefinition {
            id: "factory-droid",
            display_name: "Factory Droid",
            detect_command: "factory-droid",
        },
        AgentDefinition {
            id: "qwen",
            display_name: "Qwen",
            detect_command: "qwen",
        },
    ]
}

pub fn detect_agents() -> Result<Vec<AgentStatus>> {
    let mut statuses = Vec::new();

    for definition in definitions() {
        let skills_dir = skills_dir_for(definition.id)?;
        let config_dir = config_dir_for(definition.id)?;
        let command_detected = detect_command(definition.detect_command);
        let config_exists = config_dir.exists();
        let skills_dir_exists = skills_dir.exists();
        let skill_count = if skills_dir_exists {
            count_skill_dirs(&skills_dir)?
        } else {
            0
        };
        let installed = command_detected || config_exists || skills_dir_exists || skill_count > 0;

        statuses.push(AgentStatus {
            id: definition.id.to_string(),
            display_name: definition.display_name.to_string(),
            skills_dir: skills_dir.to_string_lossy().to_string(),
            installed,
            skill_count,
        });
    }

    Ok(statuses)
}

pub fn is_agent_present(agent_id: &str) -> bool {
    let defs = definitions();
    let Some(def) = defs.iter().find(|d| d.id == agent_id) else {
        return true;
    };

    if detect_command(def.detect_command) {
        return true;
    }

    // Skills dir is intentionally excluded: skm creates it, so it can't prove the agent is installed.
    config_dir_for(agent_id)
        .map(|dir| dir.exists())
        .unwrap_or(false)
}

pub fn all_installed_agents() -> Result<Vec<AgentStatus>> {
    Ok(detect_agents()?
        .into_iter()
        .filter(|item| item.installed)
        .collect())
}

pub fn all_agent_skill_dirs() -> Result<Vec<(String, PathBuf)>> {
    let mut result = Vec::new();
    for definition in definitions() {
        result.push((definition.id.to_string(), skills_dir_for(definition.id)?));
    }
    Ok(result)
}

pub fn shared_skills_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
    Ok(home.join(".agents").join("skills"))
}

pub fn skills_dir_for(agent_id: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
    let path = match agent_id {
        "claude-code" => home.join(".claude").join("skills"),
        "codex" => home.join(".codex").join("skills"),
        "gemini-cli" => home.join(".gemini").join("skills"),
        "copilot-cli" => home.join(".copilot").join("skills"),
        "opencode" => opencode_dir()?.join("skills"),
        "antigravity" => home.join(".gemini").join("antigravity").join("skills"),
        "cursor" => home.join(".cursor").join("skills"),
        "kiro" => home.join(".kiro").join("skills"),
        "codebuddy" => home.join(".codebuddy").join("skills"),
        "openclaw" => home.join(".openclaw").join("skills"),
        "trae" => home.join(".trae").join("skills"),
        "junie" => home.join(".junie").join("skills"),
        "qoder" => home.join(".qoder").join("skills"),
        "trae-cn" => home.join(".trae-cn").join("skills"),
        "windsurf" => home.join(".windsurf").join("skills"),
        "augment" => home.join(".augment").join("skills"),
        "kilocode" => home.join(".kilocode").join("skills"),
        "ob1" => home.join(".ob1").join("skills"),
        "amp" => home.join(".amp").join("skills"),
        "hermes" => home.join(".hermes").join("skills"),
        "factory-droid" => home.join(".factory").join("skills"),
        "qwen" => home.join(".qwen").join("skills"),
        _ => return Err(anyhow!("unsupported agent id: {agent_id}")),
    };
    Ok(path)
}

pub fn config_dir_for(agent_id: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
    let path = match agent_id {
        "claude-code" => home.join(".claude"),
        "codex" => home.join(".codex"),
        "gemini-cli" => home.join(".gemini"),
        "copilot-cli" => home.join(".copilot"),
        "opencode" => opencode_dir()?,
        "antigravity" => home.join(".gemini").join("antigravity"),
        "cursor" => home.join(".cursor"),
        "kiro" => home.join(".kiro"),
        "codebuddy" => home.join(".codebuddy"),
        "openclaw" => home.join(".openclaw"),
        "trae" => home.join(".trae"),
        "junie" => home.join(".junie"),
        "qoder" => home.join(".qoder"),
        "trae-cn" => home.join(".trae-cn"),
        "windsurf" => home.join(".windsurf"),
        "augment" => home.join(".augment"),
        "kilocode" => home.join(".kilocode"),
        "ob1" => home.join(".ob1"),
        "amp" => home.join(".amp"),
        "hermes" => home.join(".hermes"),
        "factory-droid" => home.join(".factory"),
        "qwen" => home.join(".qwen"),
        _ => return Err(anyhow!("unsupported agent id: {agent_id}")),
    };
    Ok(path)
}

pub fn agent_skills_dirs_from_config(config: &AgentsConfig) -> Vec<(String, PathBuf)> {
    config
        .agents
        .iter()
        .map(|(id, path)| (id.clone(), PathBuf::from(expand_tilde(path))))
        .collect()
}

fn opencode_dir() -> Result<PathBuf> {
    if cfg!(target_os = "windows") {
        let config_base =
            dirs::config_dir().ok_or_else(|| anyhow!("cannot resolve config directory"))?;
        Ok(config_base.join("opencode"))
    } else {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
        Ok(home.join(".config").join("opencode"))
    }
}

fn count_skill_dirs(path: &Path) -> Result<usize> {
    let mut count = 0;
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Ok(0),
    };

    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = match fs::metadata(&entry_path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if !metadata.is_dir() {
            continue;
        }
        if entry_path.join("SKILL.md").exists() {
            count += 1;
        }
    }

    Ok(count)
}

fn detect_command(command: &str) -> bool {
    if which::which(command).is_ok() {
        return true;
    }
    detect_command_via_shell(command)
}

#[cfg(target_os = "windows")]
fn detect_command_via_shell(command: &str) -> bool {
    Command::new("cmd")
        .args(["/C", "where", command])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "windows"))]
fn detect_command_via_shell(command: &str) -> bool {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let escaped = command.replace('\'', r"'\''");
    let probe = format!("command -v '{escaped}' >/dev/null 2>&1");

    Command::new(shell)
        .args(["-lc", &probe])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}
