use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSummary {
    pub id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub canonical_path: String,
    pub installed_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub summary: SkillSummary,
    pub frontmatter: serde_json::Value,
    pub markdown_body: String,
    pub scope: String,
    pub lock_entry: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub id: String,
    pub display_name: String,
    pub skills_dir: String,
    pub installed: bool,
    pub skill_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySkill {
    pub id: String,
    pub skill_id: String,
    pub name: String,
    pub source: String,
    pub installs: u64,
    pub installs_yesterday: Option<u64>,
    pub change: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySkillContent {
    pub source: String,
    pub skill_id: String,
    pub frontmatter: serde_json::Value,
    pub markdown_body: String,
    pub fetched_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheck {
    pub skill_id: String,
    pub status: String,
    pub has_update: bool,
    pub local_commit_hash: Option<String>,
    pub remote_commit_hash: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBackup {
    pub skill_id: String,
    pub snapshot_id: String,
    pub created_at: String,
    pub backup_path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LinkState {
    Linked,
    Conflict,
    NotLinked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelinkResult {
    pub linked: usize,
    pub skipped: usize,
    pub conflicts: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
}
