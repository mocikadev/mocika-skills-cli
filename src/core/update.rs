use anyhow::Result;

use super::{lock, operations};
use crate::models::UpdateCheck;

pub fn check_updates(skill_ids: &[String]) -> Result<Vec<UpdateCheck>> {
    if skill_ids.is_empty() {
        return operations::check_all_updates();
    }

    let mut checks = Vec::new();
    for skill_id in skill_ids {
        checks.push(operations::check_skill_update(skill_id)?);
    }
    checks.sort_by(|left, right| left.skill_id.cmp(&right.skill_id));
    Ok(checks)
}

pub fn installed_skill_ids() -> Result<Vec<String>> {
    Ok(lock::list_skill_entries()?
        .into_iter()
        .map(|(id, _)| id)
        .collect())
}
