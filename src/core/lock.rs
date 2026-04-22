use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::{json, Value};

const LOCK_FILE_NAME: &str = ".skill-lock.json";

pub fn lock_file_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
    Ok(home.join(".agents").join(LOCK_FILE_NAME))
}

pub fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub fn ensure_exists() -> Result<()> {
    let path = lock_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        write_json(&json!({ "skills": {} }))?;
    }

    Ok(())
}

pub fn read_json() -> Result<Value> {
    ensure_exists()?;
    let text = fs::read_to_string(lock_file_path()?)?;
    Ok(serde_json::from_str(&text)?)
}

pub fn get_skill_entry(skill_id: &str) -> Result<Option<Value>> {
    let root = read_json()?;
    Ok(root
        .get("skills")
        .and_then(|skills| skills.get(skill_id))
        .cloned())
}

pub fn list_skill_entries() -> Result<Vec<(String, Value)>> {
    let root = read_json()?;
    let Some(skills) = root.get("skills").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };

    Ok(skills
        .iter()
        .map(|(skill_id, entry)| (skill_id.clone(), entry.clone()))
        .collect())
}

pub fn upsert_skill_entry(skill_id: &str, entry: Value) -> Result<()> {
    let mut root = read_json()?;

    if root.get("skills").is_none() {
        root["skills"] = json!({});
    }

    let skills = root
        .get_mut("skills")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| anyhow!("invalid lock file: 'skills' must be an object"))?;
    skills.insert(skill_id.to_string(), entry);

    write_json(&root)
}

pub fn remove_skill_entry(skill_id: &str) -> Result<()> {
    let mut root = read_json()?;
    if let Some(skills) = root.get_mut("skills").and_then(Value::as_object_mut) {
        skills.remove(skill_id);
    }
    write_json(&root)
}

fn write_json(value: &Value) -> Result<()> {
    let path = lock_file_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let tmp = path.with_extension("json.tmp");
    let payload = serde_json::to_string_pretty(value)?;
    let result = (|| -> Result<()> {
        fs::write(&tmp, &payload)?;
        fs::rename(&tmp, &path)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
