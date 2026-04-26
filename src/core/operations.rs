use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{agent, config, git, lock, skill};
use crate::i18n;
use crate::models::{
    LinkState, OperationResult, RelinkResult, SkillBackup, SkillSummary, UpdateCheck,
};

const SKILL_FILE_NAME: &str = "SKILL.md";
const BACKUP_ROOT_NAME: &str = ".skm-backups";
const BACKUP_METADATA_FILE: &str = "snapshot.json";
const BACKUP_SKILL_DIR_NAME: &str = "skill";
const DEFAULT_BACKUP_RETENTION: usize = 8;

pub fn install_skill_from_repo(
    repo_url: &str,
    skill_subpath: Option<String>,
    target_agents: &[String],
) -> Result<SkillSummary> {
    install_skill_from_repo_with_progress(repo_url, skill_subpath, target_agents, |_, _, _| {})
}

pub fn install_skill_from_repo_with_progress<P>(
    repo_url: &str,
    skill_subpath: Option<String>,
    target_agents: &[String],
    mut progress: P,
) -> Result<SkillSummary>
where
    P: FnMut(&str, usize, usize),
{
    git::ensure_git_available()?;
    let total = 5;
    let temp = create_temp_dir()?;

    let result = (|| -> Result<SkillSummary> {
        progress("cloning repository", 0, total);
        run_git(
            &["clone", "--depth=1", repo_url, &temp.to_string_lossy()],
            None,
        )
        .context("failed to clone repository")?;
        progress("cloning repository", 1, total);

        progress("scanning repository", 1, total);
        let discovered = discover_skill_directories(&temp, 6)?;
        let discovered_candidates = discovered
            .iter()
            .filter_map(|path| path.strip_prefix(&temp).ok())
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<_>>();

        let chosen_dir = match skill_subpath {
            Some(subpath) if !subpath.trim().is_empty() => {
                let requested = subpath.trim();
                let direct = temp.join(requested);
                if direct.exists() && direct.join(SKILL_FILE_NAME).exists() {
                    direct
                } else if discovered.is_empty() {
                    bail!("no skill directory found in repository; provide skillSubpath");
                } else if let Some(resolved) =
                    resolve_skill_directory_from_selector(&temp, &discovered, requested)?
                {
                    resolved
                } else {
                    bail!(
                        "skillSubpath '{}' not found. Available skills: {}",
                        requested,
                        discovered_candidates.join(", ")
                    );
                }
            }
            _ => {
                if discovered.is_empty() {
                    bail!("no skill directory found in repository; provide skillSubpath");
                }
                if discovered.len() > 1 {
                    bail!(
                        "multiple skills found; provide skillSubpath. Candidates: {}",
                        discovered_candidates.join(", ")
                    );
                }
                discovered[0].clone()
            }
        };
        progress("scanning repository", 2, total);

        progress("copying skill files", 2, total);
        let skill_id = if is_repo_root_path(&temp, &chosen_dir) {
            derive_repo_root_skill_id(&temp, repo_url)?
        } else {
            derive_skill_id(&chosen_dir)?
        };
        let destination = shared_skill_path(&skill_id)?;
        replace_directory(&chosen_dir, &destination)?;
        progress("copying skill files", 3, total);

        progress("creating symlinks", 3, total);
        for agent_id in target_agents {
            assign_skill(&skill_id, agent_id)?;
        }
        progress("creating symlinks", 4, total);

        progress("writing lock file", 4, total);
        let commit_hash = run_git(&["rev-parse", "HEAD"], Some(&temp)).ok();
        let hash = compute_directory_hash(&destination)?;
        let now = lock::now_rfc3339();

        let source = repo_source_display(repo_url);
        let source_type = if is_github_repo_url(repo_url) {
            "github"
        } else {
            "git"
        };
        let relative_skill_dir = chosen_dir.strip_prefix(&temp).unwrap_or(&chosen_dir);
        let relative_skill_path =
            if relative_skill_dir.as_os_str().is_empty() || relative_skill_dir == Path::new(".") {
                SKILL_FILE_NAME.to_string()
            } else {
                relative_skill_dir
                    .join(SKILL_FILE_NAME)
                    .to_string_lossy()
                    .replace('\\', "/")
            };

        let lock_entry = json!({
            "source": source,
            "sourceType": source_type,
            "sourceUrl": repo_url,
            "skillPath": relative_skill_path,
            "skillFolderHash": hash,
            "installedAt": now,
            "updatedAt": lock::now_rfc3339(),
            "skillyCommitHash": commit_hash
        });

        lock::upsert_skill_entry(&skill_id, lock_entry)?;
        progress("writing lock file", 5, total);

        skill::find_skill_summary(&skill_id)?
            .ok_or_else(|| anyhow!("skill installed but scan failed: {skill_id}"))
    })();

    let _cleanup = fs::remove_dir_all(&temp);
    result
}

pub fn discover_repo_skill_subpaths(repo_url: &str) -> Result<Vec<String>> {
    git::ensure_git_available()?;
    let temp = create_temp_dir()?;

    let result = (|| -> Result<Vec<String>> {
        run_git(
            &["clone", "--depth=1", repo_url, &temp.to_string_lossy()],
            None,
        )
        .context("failed to clone repository")?;

        let mut candidates = discover_skill_directories(&temp, 6)?
            .iter()
            .filter_map(|path| path.strip_prefix(&temp).ok())
            .map(|path| path.to_string_lossy().replace('\\', "/"))
            .collect::<Vec<String>>();

        candidates.sort();
        candidates.dedup();
        Ok(candidates)
    })();

    let _cleanup = fs::remove_dir_all(&temp);
    result
}

pub fn assign_skill(skill_id: &str, agent_id: &str) -> Result<OperationResult> {
    let source = shared_skill_path(skill_id)?;
    if !source.exists() {
        bail!("skill does not exist in shared directory: {skill_id}");
    }

    let agent_dir = resolve_agent_skills_dir(agent_id)?;

    if let Ok(config_dir) = agent::config_dir_for(agent_id) {
        if !config_dir.exists() {
            bail!(
                "agent '{}' is not installed (directory not found: {})",
                agent_id,
                config_dir.display()
            );
        }
    }

    fs::create_dir_all(&agent_dir)?;

    let link_path = agent_dir.join(skill_id);
    if link_path.exists() || fs::symlink_metadata(&link_path).is_ok() {
        remove_path(&link_path)?;
    }

    create_link(&source, &link_path)?;
    Ok(OperationResult {
        success: true,
        message: format!("assigned {skill_id} to {agent_id}"),
    })
}

pub fn unassign_skill(skill_id: &str, agent_id: &str) -> Result<OperationResult> {
    let link_path = resolve_agent_skills_dir(agent_id)?.join(skill_id);
    if link_path.exists() || fs::symlink_metadata(&link_path).is_ok() {
        remove_path(&link_path)?;
    }

    Ok(OperationResult {
        success: true,
        message: format!("unassigned {skill_id} from {agent_id}"),
    })
}

pub fn delete_skill(skill_id: &str) -> Result<OperationResult> {
    let mut agent_ids = BTreeSet::new();
    for definition in agent::definitions() {
        agent_ids.insert(definition.id.to_string());
    }
    for id in config::load_agents()?.agents.keys() {
        agent_ids.insert(id.clone());
    }

    for agent_id in agent_ids {
        if let Ok(dir) = resolve_agent_skills_dir(&agent_id) {
            let link_path = dir.join(skill_id);
            if link_path.exists() || fs::symlink_metadata(&link_path).is_ok() {
                remove_path(&link_path)?;
            }
        }
    }

    let canonical = shared_skill_path(skill_id)?;
    if canonical.exists() {
        remove_path(&canonical)?;
    }

    lock::remove_skill_entry(skill_id)?;

    Ok(OperationResult {
        success: true,
        message: format!("deleted skill {skill_id}"),
    })
}

pub fn check_skill_update(skill_id: &str) -> Result<UpdateCheck> {
    git::ensure_git_available()?;
    let Some(entry) = lock::get_skill_entry(skill_id)? else {
        return Ok(UpdateCheck {
            skill_id: skill_id.to_string(),
            status: "not_tracked".to_string(),
            has_update: false,
            local_commit_hash: None,
            remote_commit_hash: None,
            message: Some(i18n::fmt_no_lock_file_entry()),
        });
    };

    let source_type = entry
        .get("sourceType")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let source_url = entry.get("sourceUrl").and_then(|value| value.as_str());

    if !supports_remote_update_source_type(source_type, source_url) {
        let source_type_display = if source_type.is_empty() {
            "unknown"
        } else {
            source_type
        };
        return Ok(UpdateCheck {
            skill_id: skill_id.to_string(),
            status: "unsupported".to_string(),
            has_update: false,
            local_commit_hash: None,
            remote_commit_hash: None,
            message: Some(i18n::fmt_source_type_no_remote_update_checks(
                source_type_display,
            )),
        });
    }

    let source_url = source_url.ok_or_else(|| anyhow!("lock entry missing sourceUrl"))?;
    let local_commit_hash = entry
        .get("skillyCommitHash")
        .and_then(|value| value.as_str())
        .map(std::string::ToString::to_string);

    let remote_raw = run_git(&["ls-remote", source_url, "HEAD"], None)?;
    let remote_commit_hash = remote_raw
        .split_whitespace()
        .next()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| anyhow!("failed to parse git ls-remote output"))?;

    let has_update = local_commit_hash
        .as_ref()
        .map(|local| local != &remote_commit_hash)
        .unwrap_or(true);

    Ok(UpdateCheck {
        skill_id: skill_id.to_string(),
        status: if has_update {
            "has_update".to_string()
        } else {
            "up_to_date".to_string()
        },
        has_update,
        local_commit_hash,
        remote_commit_hash: Some(remote_commit_hash),
        message: None,
    })
}

pub fn check_all_updates() -> Result<Vec<UpdateCheck>> {
    let mut checks = Vec::new();
    for skill_id in list_shared_skill_ids()? {
        let check = match check_skill_update(&skill_id) {
            Ok(check) => check,
            Err(error) => UpdateCheck {
                skill_id: skill_id.clone(),
                status: "error".to_string(),
                has_update: false,
                local_commit_hash: None,
                remote_commit_hash: None,
                message: Some(error.to_string()),
            },
        };
        checks.push(check);
    }
    checks.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
    Ok(checks)
}

pub fn update_skill(skill_id: &str) -> Result<SkillSummary> {
    git::ensure_git_available()?;
    let entry = lock::get_skill_entry(skill_id)?
        .ok_or_else(|| anyhow!("skill is not tracked in lock file: {skill_id}"))?;

    let source_type = entry
        .get("sourceType")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let source_url = entry.get("sourceUrl").and_then(|value| value.as_str());
    if !supports_remote_update_source_type(source_type, source_url) {
        let source_type_display = if source_type.is_empty() {
            "unknown"
        } else {
            source_type
        };
        bail!(i18n::fmt_source_type_no_remote_updates(source_type_display));
    }

    let source_url = source_url.ok_or_else(|| anyhow!("lock entry missing sourceUrl"))?;
    let skill_path = entry
        .get("skillPath")
        .and_then(|value| value.as_str())
        .ok_or_else(|| anyhow!("lock entry missing skillPath"))?;

    let folder_rel_path = if skill_path == SKILL_FILE_NAME {
        String::new()
    } else {
        skill_path
            .trim_end_matches(&format!("/{SKILL_FILE_NAME}"))
            .to_string()
    };

    let temp = create_temp_dir()?;
    let result = (|| -> Result<SkillSummary> {
        run_git(
            &["clone", "--depth=1", source_url, &temp.to_string_lossy()],
            None,
        )
        .context("failed to clone repository for update")?;

        let source_dir = if folder_rel_path.is_empty() {
            temp.clone()
        } else {
            temp.join(&folder_rel_path)
        };
        if !source_dir.exists() || !source_dir.join(SKILL_FILE_NAME).exists() {
            bail!("updated repository does not include expected skill path: {folder_rel_path}");
        }

        let destination = shared_skill_path(skill_id)?;
        let snapshot = if destination.exists() {
            Some(create_backup_snapshot(skill_id, &destination)?)
        } else {
            None
        };

        if let Err(error) = replace_directory(&source_dir, &destination) {
            if let Some(snapshot) = snapshot.as_ref() {
                let _rollback = restore_backup_snapshot(skill_id, &snapshot.snapshot_id);
            }
            return Err(error).context("failed to replace skill directory");
        }

        let new_commit_hash = run_git(&["rev-parse", "HEAD"], Some(&temp)).ok();
        let new_hash = compute_directory_hash(&destination)?;
        let now = lock::now_rfc3339();

        let mut updated = entry.clone();
        updated["skillFolderHash"] = serde_json::Value::String(new_hash);
        updated["updatedAt"] = serde_json::Value::String(now);
        updated["skillyCommitHash"] = match new_commit_hash {
            Some(value) => serde_json::Value::String(value),
            None => serde_json::Value::Null,
        };

        if let Err(error) = lock::upsert_skill_entry(skill_id, updated) {
            if let Some(snapshot) = snapshot.as_ref() {
                let _rollback = restore_backup_snapshot(skill_id, &snapshot.snapshot_id);
            }
            return Err(error).context("failed to update lock entry after applying update");
        }

        skill::find_skill_summary(skill_id)?
            .ok_or_else(|| anyhow!("skill updated but scan failed: {skill_id}"))
    })();

    let _cleanup = fs::remove_dir_all(&temp);
    result
}

pub fn list_all_backups() -> Result<Vec<SkillBackup>> {
    let root = backup_root_dir()?;
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };

    let mut all_backups = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let skill_id = entry.file_name().to_string_lossy().to_string();
        all_backups.extend(list_skill_backups(&skill_id)?);
    }

    all_backups.sort_by(|a, b| {
        a.skill_id
            .cmp(&b.skill_id)
            .then_with(|| b.snapshot_id.cmp(&a.snapshot_id))
    });
    Ok(all_backups)
}

pub fn list_skill_backups(skill_id: &str) -> Result<Vec<SkillBackup>> {
    let root = backup_skill_dir(skill_id)?;
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };

    let mut items = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let snapshot_id = entry.file_name().to_string_lossy().to_string();
        let metadata_path = entry.path().join(BACKUP_METADATA_FILE);
        let backup = match fs::read_to_string(&metadata_path) {
            Ok(text) => match serde_json::from_str::<SkillBackup>(&text) {
                Ok(item) => item,
                Err(_) => SkillBackup {
                    skill_id: skill_id.to_string(),
                    snapshot_id: snapshot_id.clone(),
                    created_at: lock::now_rfc3339(),
                    backup_path: entry.path().to_string_lossy().to_string(),
                },
            },
            Err(_) => SkillBackup {
                skill_id: skill_id.to_string(),
                snapshot_id: snapshot_id.clone(),
                created_at: lock::now_rfc3339(),
                backup_path: entry.path().to_string_lossy().to_string(),
            },
        };

        items.push(backup);
    }

    items.sort_by(|left, right| compare_snapshot_ids_desc(&left.snapshot_id, &right.snapshot_id));
    Ok(items)
}

pub fn restore_skill_backup(skill_id: &str, snapshot_id: Option<String>) -> Result<SkillSummary> {
    let chosen_snapshot = if let Some(snapshot_id) = snapshot_id {
        if snapshot_id.trim().is_empty() {
            latest_backup_snapshot(skill_id)?
        } else {
            snapshot_id.trim().to_string()
        }
    } else {
        latest_backup_snapshot(skill_id)?
    };

    restore_backup_snapshot(skill_id, &chosen_snapshot)?;
    skill::find_skill_summary(skill_id)?
        .ok_or_else(|| anyhow!("skill restored but scan failed: {skill_id}"))
}

pub fn delete_skill_backup(skill_id: &str, snapshot_id: String) -> Result<OperationResult> {
    let normalized = snapshot_id.trim();
    if normalized.is_empty() {
        bail!("snapshotId cannot be empty");
    }

    let snapshot_root = backup_skill_dir(skill_id)?.join(normalized);
    if !snapshot_root.exists() {
        bail!("backup snapshot not found: {normalized}");
    }
    fs::remove_dir_all(&snapshot_root)?;

    Ok(OperationResult {
        success: true,
        message: format!("deleted backup snapshot {normalized} for {skill_id}"),
    })
}

pub fn relink_all(force: bool, backup: bool, dry_run: bool) -> Result<RelinkResult> {
    relink_selected(None, None, force, backup, dry_run)
}

pub fn relink_agent(
    agent_id: &str,
    force: bool,
    backup: bool,
    dry_run: bool,
) -> Result<RelinkResult> {
    relink_selected(Some(agent_id), None, force, backup, dry_run)
}

pub fn relink_selected(
    agent_id: Option<&str>,
    skill_id: Option<&str>,
    force: bool,
    backup: bool,
    dry_run: bool,
) -> Result<RelinkResult> {
    let targets = resolve_relink_targets(agent_id)?;
    let skills = resolve_relink_skills(skill_id)?;
    let shared_dir = agent::shared_skills_dir()?;

    let mut result = RelinkResult {
        linked: 0,
        skipped: 0,
        conflicts: 0,
        errors: Vec::new(),
    };

    for (target_agent_id, skills_dir) in targets {
        for current_skill_id in &skills {
            let state = skill::check_link_state(current_skill_id, &skills_dir);
            match state {
                LinkState::Linked => {
                    result.skipped += 1;
                }
                LinkState::NotLinked => {
                    if !dry_run {
                        let operation = (|| -> Result<()> {
                            fs::create_dir_all(&skills_dir)?;
                            create_link(
                                &shared_dir.join(current_skill_id),
                                &skills_dir.join(current_skill_id),
                            )
                        })();
                        if let Err(error) = operation {
                            result.errors.push(format!(
                                "{} -> {}: {}",
                                current_skill_id, target_agent_id, error
                            ));
                            continue;
                        }
                    }
                    result.linked += 1;
                }
                LinkState::Conflict => {
                    if !force {
                        result.conflicts += 1;
                        continue;
                    }

                    let destination = skills_dir.join(current_skill_id);
                    if backup && !dry_run {
                        if let Err(error) = backup_conflicting_path(
                            &target_agent_id,
                            current_skill_id,
                            &destination,
                        ) {
                            result.errors.push(format!(
                                "backup {} -> {} failed: {}",
                                current_skill_id, target_agent_id, error
                            ));
                            continue;
                        }
                    }

                    if !dry_run {
                        let op = remove_path(&destination).and_then(|_| {
                            fs::create_dir_all(&skills_dir)?;
                            create_link(&shared_dir.join(current_skill_id), &destination)
                        });
                        if let Err(error) = op {
                            result.errors.push(format!(
                                "{} -> {}: {}",
                                current_skill_id, target_agent_id, error
                            ));
                            continue;
                        }
                    }
                    result.linked += 1;
                }
            }
        }
    }

    Ok(result)
}

fn shared_skill_path(skill_id: &str) -> Result<PathBuf> {
    Ok(agent::shared_skills_dir()?.join(skill_id))
}

fn backup_root_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve user home directory"))?;
    Ok(home.join(".agents").join(BACKUP_ROOT_NAME))
}

fn backup_skill_dir(skill_id: &str) -> Result<PathBuf> {
    Ok(backup_root_dir()?.join(skill_id))
}

fn create_backup_snapshot(skill_id: &str, source_dir: &Path) -> Result<SkillBackup> {
    let snapshot_id = current_epoch_millis().to_string();
    let created_at = lock::now_rfc3339();
    let backup_dir = backup_skill_dir(skill_id)?.join(&snapshot_id);
    let skill_backup_dir = backup_dir.join(BACKUP_SKILL_DIR_NAME);

    if let Some(parent) = backup_dir.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::create_dir_all(&backup_dir)?;
    copy_directory(source_dir, &skill_backup_dir)?;

    let backup = SkillBackup {
        skill_id: skill_id.to_string(),
        snapshot_id: snapshot_id.clone(),
        created_at,
        backup_path: backup_dir.to_string_lossy().to_string(),
    };
    fs::write(
        backup_dir.join(BACKUP_METADATA_FILE),
        serde_json::to_string_pretty(&backup)?,
    )?;
    prune_backup_snapshots(skill_id, DEFAULT_BACKUP_RETENTION)?;
    Ok(backup)
}

fn prune_backup_snapshots(skill_id: &str, keep: usize) -> Result<()> {
    if keep == 0 {
        return Ok(());
    }

    let root = backup_skill_dir(skill_id)?;
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    let mut snapshots = Vec::<(String, PathBuf)>::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        snapshots.push((
            entry.file_name().to_string_lossy().to_string(),
            entry.path(),
        ));
    }

    snapshots.sort_by(|left, right| compare_snapshot_ids_desc(&left.0, &right.0));
    for (index, (_, path)) in snapshots.into_iter().enumerate() {
        if index >= keep {
            fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}

fn latest_backup_snapshot(skill_id: &str) -> Result<String> {
    list_skill_backups(skill_id)?
        .first()
        .map(|item| item.snapshot_id.clone())
        .ok_or_else(|| anyhow!("no backups found for skill: {skill_id}"))
}

fn compare_snapshot_ids_desc(left: &str, right: &str) -> Ordering {
    let left_number = left.parse::<u128>();
    let right_number = right.parse::<u128>();

    match (left_number, right_number) {
        (Ok(left_value), Ok(right_value)) => {
            right_value.cmp(&left_value).then_with(|| right.cmp(left))
        }
        _ => right.cmp(left),
    }
}

fn restore_backup_snapshot(skill_id: &str, snapshot_id: &str) -> Result<()> {
    let snapshot_root = backup_skill_dir(skill_id)?.join(snapshot_id);
    let backup_skill_path = snapshot_root.join(BACKUP_SKILL_DIR_NAME);
    if !backup_skill_path.exists() || !backup_skill_path.join(SKILL_FILE_NAME).exists() {
        bail!("backup snapshot is invalid or missing SKILL.md: {snapshot_id}");
    }

    let destination = shared_skill_path(skill_id)?;
    replace_directory(&backup_skill_path, &destination)?;

    let new_hash = compute_directory_hash(&destination)?;
    let now = lock::now_rfc3339();
    if let Some(mut entry) = lock::get_skill_entry(skill_id)? {
        entry["skillFolderHash"] = serde_json::Value::String(new_hash);
        entry["updatedAt"] = serde_json::Value::String(now);
        entry["skillyCommitHash"] = serde_json::Value::Null;
        lock::upsert_skill_entry(skill_id, entry)?;
    }
    Ok(())
}

fn derive_skill_id(path: &Path) -> Result<String> {
    let raw = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or_else(|| anyhow!("cannot derive skill id from path"))?;

    let normalized = raw
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if normalized.is_empty() {
        bail!(
            "cannot derive valid skill id from path: {}",
            path.to_string_lossy()
        );
    }

    Ok(normalized)
}

fn is_repo_root_path(repo_root: &Path, candidate: &Path) -> bool {
    candidate
        .strip_prefix(repo_root)
        .map(|relative| relative.as_os_str().is_empty() || relative == Path::new("."))
        .unwrap_or(false)
}

fn derive_repo_root_skill_id(repo_root: &Path, repo_url: &str) -> Result<String> {
    if let Some(frontmatter_name) = frontmatter_name_from_skill_md(repo_root) {
        let normalized = normalize_skill_token(&frontmatter_name);
        if !normalized.is_empty() {
            return Ok(normalized);
        }
    }

    let repo_name = repo_source_display(repo_url)
        .split('/')
        .next_back()
        .map(normalize_skill_token)
        .unwrap_or_default();
    if !repo_name.is_empty() {
        return Ok(repo_name);
    }

    derive_skill_id(repo_root)
}

fn replace_directory(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() || fs::symlink_metadata(destination).is_ok() {
        remove_path(destination)?;
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_directory(source, destination)
}

fn copy_directory(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let entry_path = entry.path();
        let target_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&entry_path)?;

        if metadata.is_dir() {
            copy_directory(&entry_path, &target_path)?;
        } else if metadata.is_file() {
            fs::copy(&entry_path, &target_path)?;
        }
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path)?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn create_link(source: &Path, target: &Path) -> Result<()> {
    #[cfg(target_family = "unix")]
    {
        std::os::unix::fs::symlink(source, target)?;
        Ok(())
    }

    #[cfg(target_family = "windows")]
    {
        std::os::windows::fs::symlink_dir(source, target).map_err(|e| {
            // ERROR_PRIVILEGE_NOT_HELD = 1314
            if e.raw_os_error() == Some(1314) {
                anyhow::anyhow!(
                    "cannot create symlink: Windows requires Developer Mode or elevated privileges.\n\
                     Enable Developer Mode: Settings → System → For developers → Developer Mode,\n\
                     or run skm as Administrator."
                )
            } else {
                anyhow::anyhow!("cannot create symlink: {e}")
            }
        })?;
        Ok(())
    }
}

fn compute_directory_hash(path: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_files(path, path, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for relative in files {
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        let content = fs::read(path.join(&relative))?;
        hasher.update(&content);
        hasher.update([0]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_files(root: &Path, current: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = fs::symlink_metadata(&entry_path)?;
        if metadata.is_dir() {
            collect_files(root, &entry_path, out)?;
        } else if metadata.is_file() {
            out.push(
                entry_path
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    Ok(())
}

fn create_temp_dir() -> Result<PathBuf> {
    let path = std::env::temp_dir().join(format!("skm-{}", current_epoch_millis()));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn run_git(args: &[&str], cwd: Option<&Path>) -> Result<String> {
    let mut command = Command::new("git");
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = command
        .output()
        .with_context(|| format!("failed to execute git command: git {}", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("git {} failed", args.join(" "))
        } else {
            format!("git {} failed: {stderr}", args.join(" "))
        };
        bail!(message);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn discover_skill_directories(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    discover_skill_directories_inner(root, root, 0, max_depth, &mut found)?;
    Ok(found)
}

fn discover_skill_directories_inner(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    found: &mut Vec<PathBuf>,
) -> Result<()> {
    if depth > max_depth {
        return Ok(());
    }

    let skill_md = current.join(SKILL_FILE_NAME);
    if skill_md.exists() {
        found.push(current.to_path_buf());
        if current != root {
            return Ok(());
        }
    }

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let path = entry.path();
        if path
            .file_name()
            .map(|name| name.to_string_lossy() == ".git")
            .unwrap_or(false)
        {
            continue;
        }

        discover_skill_directories_inner(root, &path, depth + 1, max_depth, found)?;
    }

    Ok(())
}

fn normalize_selector_path(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_lowercase()
}

fn normalize_skill_token(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn frontmatter_name_from_skill_md(skill_dir: &Path) -> Option<String> {
    let path = skill_dir.join(SKILL_FILE_NAME);
    let content = fs::read_to_string(path).ok()?;

    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed == "---" {
            break;
        }
        let (key, value) = trimmed.split_once(':')?;
        if key.trim().eq_ignore_ascii_case("name") {
            return Some(
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string(),
            );
        }
    }

    None
}

fn resolve_skill_directory_from_selector(
    root: &Path,
    discovered: &[PathBuf],
    selector: &str,
) -> Result<Option<PathBuf>> {
    let selector_path = normalize_selector_path(selector);
    let selector_token = normalize_skill_token(selector);
    if selector_path.is_empty() && selector_token.is_empty() {
        return Ok(None);
    }

    let mut best: Option<(i32, PathBuf)> = None;
    for dir in discovered {
        let relative = dir
            .strip_prefix(root)
            .unwrap_or(dir)
            .to_string_lossy()
            .replace('\\', "/");
        let relative_norm = normalize_selector_path(&relative);
        let basename_norm = dir
            .file_name()
            .map(|name| normalize_selector_path(&name.to_string_lossy()))
            .unwrap_or_default();
        let derived_id = derive_skill_id(dir).unwrap_or_default();
        let derived_norm = normalize_skill_token(&derived_id);
        let frontmatter_name = frontmatter_name_from_skill_md(dir).unwrap_or_default();
        let frontmatter_norm = normalize_skill_token(&frontmatter_name);

        let mut score = 0;
        if !selector_path.is_empty() && relative_norm == selector_path {
            score += 120;
        }
        if !selector_path.is_empty() && basename_norm == selector_path {
            score += 95;
        }
        if !selector_token.is_empty() && frontmatter_norm == selector_token {
            score += 90;
        }
        if !selector_token.is_empty() && derived_norm == selector_token {
            score += 80;
        }
        if !selector_path.is_empty() && relative_norm.ends_with(&format!("/{selector_path}")) {
            score += 50;
        }
        if !selector_path.is_empty() && relative_norm.contains(&selector_path) {
            score += 25;
        }
        if score <= 0 {
            continue;
        }

        match &best {
            Some((best_score, _)) if score <= *best_score => {}
            _ => best = Some((score, dir.clone())),
        }
    }

    Ok(best.map(|(_, path)| path))
}

fn repo_source_display(repo_url: &str) -> String {
    let normalized = repo_url
        .trim()
        .trim_end_matches(".git")
        .trim_end_matches('/');
    if normalized.is_empty() {
        return repo_url.trim().to_string();
    }

    if let Some(remaining) = normalized.strip_prefix("git@") {
        let mut parts = remaining.splitn(2, ':');
        let _host = parts.next();
        if let Some(path) = parts.next() {
            let trimmed = path.trim_matches('/');
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    if let Some(protocol_index) = normalized.find("://") {
        let after_scheme = &normalized[(protocol_index + 3)..];
        if let Some(path_index) = after_scheme.find('/') {
            let path = after_scheme[(path_index + 1)..].trim_matches('/');
            if !path.is_empty() {
                return path.to_string();
            }
        }
    }

    normalized
        .rsplit(':')
        .next()
        .unwrap_or(normalized)
        .rsplit('/')
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/")
}

fn is_github_repo_url(repo_url: &str) -> bool {
    let normalized = repo_url.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }

    if normalized.starts_with("git@github.com:") || normalized.starts_with("git@www.github.com:") {
        return true;
    }

    [
        "https://github.com/",
        "http://github.com/",
        "https://www.github.com/",
        "http://www.github.com/",
        "ssh://git@github.com/",
        "ssh://git@www.github.com/",
        "git://github.com/",
        "git://www.github.com/",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn is_likely_remote_git_url(source_url: &str) -> bool {
    let normalized = source_url.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }

    normalized.starts_with("git@")
        || normalized.starts_with("http://")
        || normalized.starts_with("https://")
        || normalized.starts_with("ssh://")
        || normalized.starts_with("git://")
}

fn supports_remote_update_source_type(source_type: &str, source_url: Option<&str>) -> bool {
    if source_type.eq_ignore_ascii_case("github") || source_type.eq_ignore_ascii_case("git") {
        return true;
    }
    if source_type.trim().is_empty() {
        return source_url.map(is_likely_remote_git_url).unwrap_or(false);
    }
    false
}

fn resolve_agent_skills_dir(agent_id: &str) -> Result<PathBuf> {
    if let Ok(path) = agent::skills_dir_for(agent_id) {
        return Ok(path);
    }

    let config = config::load_agents()?;
    let Some(path) = config.agents.get(agent_id) else {
        return Err(anyhow!("unsupported or unregistered agent id: {agent_id}"));
    };
    Ok(PathBuf::from(config::expand_tilde(path)))
}

fn list_shared_skill_ids() -> Result<Vec<String>> {
    let shared = agent::shared_skills_dir()?;
    let entries = match fs::read_dir(&shared) {
        Ok(entries) => entries,
        Err(_) => return Ok(Vec::new()),
    };

    let mut ids = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        if metadata.is_dir() && path.join(SKILL_FILE_NAME).exists() {
            ids.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    ids.sort();
    Ok(ids)
}

fn resolve_relink_targets(agent_id: Option<&str>) -> Result<Vec<(String, PathBuf)>> {
    if let Some(agent_id) = agent_id {
        return Ok(vec![(
            agent_id.to_string(),
            resolve_agent_skills_dir(agent_id)?,
        )]);
    }

    let configured = config::load_agents()?;
    let mut targets = agent::agent_skills_dirs_from_config(&configured)
        .into_iter()
        .filter(|(id, _)| agent::is_agent_present(id))
        .collect::<Vec<_>>();
    if targets.is_empty() {
        targets = agent::all_installed_agents()?
            .into_iter()
            .map(|item| (item.id, PathBuf::from(item.skills_dir)))
            .collect();
    }

    let mut dedup = HashSet::new();
    Ok(targets
        .into_iter()
        .filter(|(id, _)| dedup.insert(id.clone()))
        .collect())
}

fn resolve_relink_skills(skill_id: Option<&str>) -> Result<Vec<String>> {
    match skill_id {
        Some(skill_id) => {
            let canonical = shared_skill_path(skill_id)?;
            if !canonical.exists() || !canonical.join(SKILL_FILE_NAME).exists() {
                bail!("skill not found in shared directory: {skill_id}");
            }
            Ok(vec![skill_id.to_string()])
        }
        None => list_shared_skill_ids(),
    }
}

fn backup_conflicting_path(agent_id: &str, skill_id: &str, path: &Path) -> Result<()> {
    if !path.exists() && fs::symlink_metadata(path).is_err() {
        return Ok(());
    }

    let backup_root = backup_root_dir()?
        .join("_relink-conflicts")
        .join(agent_id)
        .join(skill_id)
        .join(current_epoch_millis().to_string());
    fs::create_dir_all(&backup_root)?;

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        let target = backup_root.join(
            path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "conflict".to_string()),
        );
        fs::copy(path, target)?;
    } else if metadata.is_dir() {
        copy_directory(path, &backup_root.join(BACKUP_SKILL_DIR_NAME))?;
    }
    Ok(())
}

fn current_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}
