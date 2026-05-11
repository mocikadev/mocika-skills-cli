use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{agent, config, lock};
use crate::models::{LinkState, SkillDetail, SkillFixResult, SkillSummary};

#[derive(Debug, Clone)]
struct SkillAggregate {
    id: String,
    display_name: String,
    description: Option<String>,
    canonical_path: String,
    installed_on: BTreeSet<String>,
}

pub fn scan_skills() -> Result<Vec<SkillSummary>> {
    let mut found: BTreeMap<String, SkillAggregate> = BTreeMap::new();

    scan_directory(&agent::shared_skills_dir()?, "shared", &mut found)?;

    for (agent_id, dir) in agent::all_agent_skill_dirs()? {
        scan_directory(&dir, &agent_id, &mut found)?;
    }

    let configured_agents = config::load_agents()?;
    for (agent_id, dir) in agent::agent_skills_dirs_from_config(&configured_agents) {
        scan_directory(&dir, &agent_id, &mut found)?;
    }

    let mut summaries = found
        .into_values()
        .map(|value| SkillSummary {
            id: value.id,
            display_name: value.display_name,
            description: value.description,
            canonical_path: value.canonical_path,
            installed_on: value.installed_on.into_iter().collect(),
        })
        .collect::<Vec<_>>();

    summaries.sort_by(|left, right| {
        left.display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(summaries)
}

pub fn find_skill_summary(skill_id: &str) -> Result<Option<SkillSummary>> {
    Ok(scan_skills()?
        .into_iter()
        .find(|skill| skill.id == skill_id))
}

pub fn read_skill_detail(skill_id: &str) -> Result<Option<SkillDetail>> {
    let Some(summary) = find_skill_summary(skill_id)? else {
        return Ok(None);
    };

    let skill_md = PathBuf::from(&summary.canonical_path).join("SKILL.md");
    let (frontmatter, markdown_body) = parse_skill_md(&skill_md)?;
    let scope = resolve_scope(&summary.canonical_path).to_string();
    let lock_entry = lock::get_skill_entry(&summary.id)?;

    Ok(Some(SkillDetail {
        summary,
        frontmatter,
        markdown_body,
        scope,
        lock_entry,
    }))
}

pub fn check_link_state(skill_id: &str, agent_skills_dir: &Path) -> LinkState {
    let link = agent_skills_dir.join(skill_id);
    let shared = match agent::shared_skills_dir() {
        Ok(dir) => dir,
        Err(_) => return LinkState::Conflict,
    };
    let canonical = shared.join(skill_id);

    if let Ok(meta) = std::fs::symlink_metadata(&link) {
        if meta.file_type().is_symlink() {
            if let Ok(target) = std::fs::read_link(&link) {
                if target == canonical {
                    return LinkState::Linked;
                }
            }
        }
        return LinkState::Conflict;
    }
    LinkState::NotLinked
}

fn scan_directory(
    base_dir: &Path,
    source_id: &str,
    found: &mut BTreeMap<String, SkillAggregate>,
) -> Result<()> {
    let entries = match fs::read_dir(base_dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
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

        let canonical_path = fs::canonicalize(&entry_path).unwrap_or(entry_path.clone());
        let skill_md = canonical_path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let (frontmatter, _) = parse_skill_md(&skill_md)?;
        let fallback_id = entry.file_name().to_string_lossy().to_string();
        let id = canonical_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| fallback_id.clone());

        let display_name = frontmatter
            .get("name")
            .and_then(|value| value.as_str())
            .map(std::string::ToString::to_string)
            .unwrap_or_else(|| fallback_id.clone());

        let description = frontmatter
            .get("description")
            .and_then(|value| value.as_str())
            .map(std::string::ToString::to_string);

        let key = canonical_path.to_string_lossy().to_string();
        let aggregate = found.entry(key.clone()).or_insert_with(|| SkillAggregate {
            id,
            display_name,
            description,
            canonical_path: key,
            installed_on: BTreeSet::new(),
        });
        aggregate.installed_on.insert(source_id.to_string());
    }

    Ok(())
}

fn parse_skill_md(path: &Path) -> Result<(serde_json::Value, String)> {
    let content = fs::read_to_string(path)?;
    if !content.starts_with("---\n") {
        return Ok((serde_json::Value::Object(Default::default()), content));
    }

    let mut yaml_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut lines = content.lines();
    let _first_line = lines.next();
    let mut in_frontmatter = true;

    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            continue;
        }

        if in_frontmatter {
            yaml_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    let yaml = yaml_lines.join("\n");
    let frontmatter = if yaml.trim().is_empty() {
        serde_json::Value::Object(Default::default())
    } else {
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml)?;
        serde_json::to_value(yaml_value)?
    };

    Ok((frontmatter, body_lines.join("\n")))
}

fn resolve_scope(canonical_path: &str) -> &'static str {
    if let Ok(shared_dir) = agent::shared_skills_dir() {
        let shared = shared_dir.to_string_lossy().to_string();
        if canonical_path.starts_with(&shared) {
            return "shared_global";
        }
    }
    "agent_local"
}

pub fn fix_skill_frontmatter(skill_id: &str, dry_run: bool) -> Result<SkillFixResult> {
    let skill_dir = agent::shared_skills_dir()?.join(skill_id);
    let skill_md_path = skill_dir.join("SKILL.md");

    if !skill_md_path.exists() {
        return Ok(SkillFixResult {
            skill_id: skill_id.to_string(),
            fixed: vec![],
            warnings: vec!["SKILL.md not found".to_string()],
            applied: false,
        });
    }

    let content = fs::read_to_string(&skill_md_path)?;
    let (new_content, fixed, warnings) = analyze_and_fix(skill_id, &content);

    let has_fixes = !fixed.is_empty();
    let applied = if has_fixes && !dry_run {
        fs::write(&skill_md_path, new_content.as_bytes())?;
        true
    } else {
        false
    };

    Ok(SkillFixResult {
        skill_id: skill_id.to_string(),
        fixed,
        warnings,
        applied,
    })
}

fn analyze_and_fix(skill_id: &str, content: &str) -> (String, Vec<String>, Vec<String>) {
    let mut fixed = Vec::new();
    let mut warnings = Vec::new();

    if !content.starts_with("---\n") {
        fixed.push("added frontmatter with name field".to_string());
        return (
            format!("---\nname: {skill_id}\n---\n\n{content}"),
            fixed,
            warnings,
        );
    }

    let rest = &content[4..];
    let (end_idx, sep_len) = if let Some(idx) = rest.find("\n---\n") {
        (idx, 5usize)
    } else if rest.ends_with("\n---") {
        (rest.len() - 4, 4usize)
    } else {
        warnings.push("malformed frontmatter: missing closing ---".to_string());
        return (content.to_string(), fixed, warnings);
    };

    let yaml_str = &rest[..end_idx];
    let after_fm = &rest[end_idx + sep_len..];

    match serde_yaml::from_str::<serde_yaml::Value>(yaml_str) {
        Err(e) => {
            warnings.push(format!("YAML parse error (skipped): {e}"));
            (content.to_string(), fixed, warnings)
        }
        Ok(yaml_val) => {
            let mut yaml_lines: Vec<String> = yaml_str.lines().map(|l| l.to_string()).collect();
            let name_opt = yaml_val.get("name").and_then(|v| v.as_str());

            match name_opt {
                None => {
                    yaml_lines.insert(0, format!("name: {skill_id}"));
                    fixed.push("added missing name field".to_string());
                }
                Some(name) if name != skill_id => {
                    for line in yaml_lines.iter_mut() {
                        if line.trim_start().starts_with("name:") {
                            *line = format!("name: {skill_id}");
                            break;
                        }
                    }
                    fixed.push(format!("corrected name: \"{name}\" → \"{skill_id}\""));
                }
                _ => {}
            }

            let new_yaml = yaml_lines.join("\n");
            (format!("---\n{new_yaml}\n---\n{after_fm}"), fixed, warnings)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::analyze_and_fix;

    #[test]
    fn no_frontmatter_prepends_block_with_name() {
        let content = "# Hello\nsome content\n";
        let (new_content, fixed, warnings) = analyze_and_fix("my-skill", content);
        assert!(warnings.is_empty());
        assert_eq!(fixed, vec!["added frontmatter with name field"]);
        assert!(new_content.starts_with("---\nname: my-skill\n---\n"));
        assert!(new_content.contains("# Hello"));
    }

    #[test]
    fn missing_name_field_gets_added() {
        let content = "---\ndescription: a skill\n---\n# Body\n";
        let (new_content, fixed, warnings) = analyze_and_fix("my-skill", content);
        assert!(warnings.is_empty());
        assert_eq!(fixed, vec!["added missing name field"]);
        assert!(new_content.contains("name: my-skill"));
        assert!(new_content.contains("description: a skill"));
    }

    #[test]
    fn wrong_name_gets_corrected() {
        let content = "---\nname: old-name\ndescription: a skill\n---\n# Body\n";
        let (new_content, fixed, warnings) = analyze_and_fix("correct-name", content);
        assert!(warnings.is_empty());
        assert_eq!(
            fixed,
            vec!["corrected name: \"old-name\" → \"correct-name\""]
        );
        assert!(new_content.contains("name: correct-name"));
        assert!(!new_content.contains("name: old-name"));
    }

    #[test]
    fn correct_name_produces_no_changes() {
        let content = "---\nname: my-skill\ndescription: a skill\n---\n# Body\n";
        let (_, fixed, warnings) = analyze_and_fix("my-skill", content);
        assert!(fixed.is_empty());
        assert!(warnings.is_empty());
    }

    #[test]
    fn missing_closing_delimiter_emits_warning() {
        let content = "---\nname: my-skill\n";
        let (returned_content, fixed, warnings) = analyze_and_fix("my-skill", content);
        assert!(fixed.is_empty());
        assert!(!warnings.is_empty());
        assert_eq!(returned_content, content);
    }
}
