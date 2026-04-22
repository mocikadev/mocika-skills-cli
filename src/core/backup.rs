use anyhow::Result;

use super::operations;
use crate::models::SkillBackup;

pub fn list_backups(skill_id: &str) -> Result<Vec<SkillBackup>> {
    operations::list_skill_backups(skill_id)
}

pub fn list_all_backups() -> Result<Vec<SkillBackup>> {
    operations::list_all_backups()
}

pub fn restore_backup(skill_id: &str, snapshot_id: Option<String>) -> Result<()> {
    operations::restore_skill_backup(skill_id, snapshot_id).map(|_| ())
}

pub fn delete_backup(skill_id: &str, snapshot_id: &str) -> Result<()> {
    operations::delete_skill_backup(skill_id, snapshot_id.to_string()).map(|_| ())
}
