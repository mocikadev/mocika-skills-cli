use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Bundle {
    #[serde(default)]
    pub skills: Vec<BundleEntry>,
}

pub fn write_bundle(path: &Path, bundle: &Bundle) -> Result<()> {
    let content = toml::to_string_pretty(bundle)?;
    fs::write(path, content.as_bytes())?;
    Ok(())
}

pub fn read_bundle(path: &Path) -> Result<Bundle> {
    if !path.exists() {
        bail!("bundle file not found: {}", path.display());
    }
    let content = fs::read_to_string(path)?;
    let bundle: Bundle = toml::from_str(&content)?;
    Ok(bundle)
}

pub fn bundle_source_from_lock(entry: &serde_json::Value) -> Option<String> {
    const SKILL_MD: &str = "SKILL.md";
    let source_type = entry
        .get("sourceType")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if source_type == "local" {
        return None;
    }
    let url = entry
        .get("sourceUrl")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            entry
                .get("source")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
        })?;
    let skill_path = entry
        .get("skillPath")
        .and_then(|v| v.as_str())
        .unwrap_or(SKILL_MD);
    if skill_path == SKILL_MD {
        Some(url.to_string())
    } else {
        let subpath = skill_path.trim_end_matches(&format!("/{SKILL_MD}"));
        Some(format!("{url}#{subpath}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn roundtrip_empty_bundle() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.bundle");
        let bundle = Bundle::default();
        write_bundle(&path, &bundle).unwrap();

        let loaded = read_bundle(&path).unwrap();
        assert_eq!(loaded.skills.len(), 0);
    }

    #[test]
    fn roundtrip_with_entries() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("skills.bundle");
        let bundle = Bundle {
            skills: vec![
                BundleEntry {
                    name: "android-cli".to_string(),
                    source: "https://github.com/foo/bar".to_string(),
                },
                BundleEntry {
                    name: "rust-skills".to_string(),
                    source: "owner/repo#skills/rust-skills".to_string(),
                },
            ],
        };
        write_bundle(&path, &bundle).unwrap();

        let loaded = read_bundle(&path).unwrap();
        assert_eq!(loaded.skills.len(), 2);
        assert_eq!(loaded.skills[0].name, "android-cli");
        assert_eq!(loaded.skills[0].source, "https://github.com/foo/bar");
        assert_eq!(loaded.skills[1].name, "rust-skills");
        assert_eq!(loaded.skills[1].source, "owner/repo#skills/rust-skills");
    }

    #[test]
    fn read_nonexistent_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does_not_exist.bundle");
        assert!(read_bundle(&path).is_err());
    }

    #[test]
    fn read_invalid_toml_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bad.bundle");
        fs::write(&path, b"this is not valid toml ][").unwrap();
        assert!(read_bundle(&path).is_err());
    }

    #[test]
    fn source_from_lock_local_skipped() {
        let entry = serde_json::json!({
            "sourceType": "local",
            "sourceUrl": "/home/user/myskill"
        });
        assert_eq!(bundle_source_from_lock(&entry), None);
    }

    #[test]
    fn source_from_lock_github_root_skill() {
        let entry = serde_json::json!({
            "sourceType": "github",
            "sourceUrl": "https://github.com/foo/bar",
            "skillPath": "SKILL.md"
        });
        assert_eq!(
            bundle_source_from_lock(&entry),
            Some("https://github.com/foo/bar".to_string())
        );
    }

    #[test]
    fn source_from_lock_github_with_subpath() {
        let entry = serde_json::json!({
            "sourceType": "github",
            "sourceUrl": "https://github.com/foo/bar",
            "skillPath": "skills/android-cli/SKILL.md"
        });
        assert_eq!(
            bundle_source_from_lock(&entry),
            Some("https://github.com/foo/bar#skills/android-cli".to_string())
        );
    }

    #[test]
    fn source_from_lock_missing_url_returns_none() {
        let entry = serde_json::json!({ "sourceType": "github" });
        assert_eq!(bundle_source_from_lock(&entry), None);
    }

    #[test]
    fn source_from_lock_fallback_to_source_field() {
        let entry = serde_json::json!({
            "sourceType": "git",
            "source": "https://gitlab.com/org/repo",
            "skillPath": "SKILL.md"
        });
        assert_eq!(
            bundle_source_from_lock(&entry),
            Some("https://gitlab.com/org/repo".to_string())
        );
    }
}
