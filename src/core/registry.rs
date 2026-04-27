use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

use crate::models::{RegistrySkill, RegistrySkillContent};

const LEADERBOARD_TTL_SECS: u64 = 5 * 60;
const SKILL_CONTENT_TTL_SECS: u64 = 10 * 60;
const GIT_SOURCE_CACHE_TTL_SECS: u64 = 5 * 60;
const APP_USER_AGENT: &str = concat!("skm/", env!("CARGO_PKG_VERSION"));
const SKILL_FILE_NAME: &str = "SKILL.md";

enum SkillContentSource {
    GitHubSource(String),
    GitRepositoryUrl(String),
}

#[derive(Debug, Clone)]
struct CacheEntry {
    skills: Vec<RegistrySkill>,
    fetched_at: u64,
}

#[derive(Debug, Clone)]
struct ContentCacheEntry {
    content: RegistrySkillContent,
    fetched_at: u64,
}

static LEADERBOARD_CACHE: OnceLock<Mutex<HashMap<String, CacheEntry>>> = OnceLock::new();
static SKILL_CONTENT_CACHE: OnceLock<Mutex<HashMap<String, ContentCacheEntry>>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct SearchResponse {
    skills: Option<Vec<RawRegistrySkill>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRegistrySkill {
    id: Option<String>,
    skill_id: Option<String>,
    name: Option<String>,
    source: Option<String>,
    installs: Option<Value>,
    installs_yesterday: Option<Value>,
    change: Option<Value>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitTreeResponse {
    tree: Option<Vec<GitTreeEntry>>,
}

#[derive(Debug, Deserialize)]
struct GitTreeEntry {
    path: Option<String>,
    #[serde(rename = "type")]
    entry_type: Option<String>,
}

fn cache_store() -> &'static Mutex<HashMap<String, CacheEntry>> {
    LEADERBOARD_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn content_cache_store() -> &'static Mutex<HashMap<String, ContentCacheEntry>> {
    SKILL_CONTENT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn build_client() -> Result<Client> {
    Ok(Client::builder().timeout(Duration::from_secs(20)).build()?)
}

fn normalize_base_url(input: &str) -> Result<String> {
    let normalized = input.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        return Err(anyhow!("registry URL cannot be empty"));
    }
    Ok(normalized)
}

fn is_skills_sh_registry(base_url: &str) -> bool {
    base_url.to_lowercase().contains("skills.sh")
}

fn normalize_github_source(input: &str) -> Result<String> {
    let normalized = input.trim().trim_matches('/').to_string();
    let parts: Vec<&str> = normalized.split('/').collect();
    if parts.len() != 2 || parts.iter().any(|item| item.trim().is_empty()) {
        bail!("registry source must be in owner/repo format");
    }
    Ok(normalized)
}

fn normalize_repository_url(input: &str) -> Result<String> {
    let normalized = input.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        bail!("repository URL cannot be empty");
    }
    Ok(normalized)
}

fn parse_owner_repo_from_path(input: &str) -> Option<String> {
    let normalized = input.trim().trim_matches('/');
    if normalized.is_empty() {
        return None;
    }

    let mut parts = normalized.split('/').filter(|item| !item.trim().is_empty());
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim().trim_end_matches(".git").trim();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some(format!("{owner}/{repo}"))
}

fn parse_github_source_from_repository_url(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with("git@") {
        let remaining = trimmed.strip_prefix("git@")?;
        let mut parts = remaining.splitn(2, ':');
        let host = parts.next()?.trim().to_lowercase();
        let path = parts.next()?.trim();
        if host == "github.com" || host == "www.github.com" {
            return parse_owner_repo_from_path(path);
        }
        return None;
    }

    let normalized_lower = trimmed.to_lowercase();
    let prefixes = [
        "https://github.com/",
        "http://github.com/",
        "https://www.github.com/",
        "http://www.github.com/",
        "ssh://git@github.com/",
        "ssh://git@www.github.com/",
        "git://github.com/",
        "git://www.github.com/",
    ];

    for prefix in prefixes {
        if normalized_lower.starts_with(prefix) {
            return parse_owner_repo_from_path(&trimmed[prefix.len()..]);
        }
    }

    None
}

fn parse_skill_content_source(input: &str) -> Result<SkillContentSource> {
    let normalized = input.trim();
    if normalized.is_empty() {
        bail!("registry source cannot be empty");
    }

    if let Some(github_source) = parse_github_source_from_repository_url(normalized) {
        return Ok(SkillContentSource::GitHubSource(github_source));
    }

    if normalized.contains("://") || normalized.starts_with("git@") {
        return Ok(SkillContentSource::GitRepositoryUrl(
            normalize_repository_url(normalized)?,
        ));
    }

    Ok(SkillContentSource::GitHubSource(normalize_github_source(
        normalized,
    )?))
}

fn normalize_skill_id(input: &str) -> Result<String> {
    let normalized = input.trim().trim_matches('/').to_string();
    if normalized.is_empty() {
        bail!("skill id cannot be empty");
    }
    Ok(normalized)
}

fn normalize_category(input: &str) -> Result<&'static str> {
    match input.trim() {
        "all_time" | "all-time" | "alltime" => Ok("all_time"),
        "trending" => Ok("trending"),
        "hot" => Ok("hot"),
        _ => Err(anyhow!("unsupported leaderboard category: {input}")),
    }
}

fn category_path(category: &str) -> &'static str {
    match category {
        "trending" => "/trending",
        "hot" => "/hot",
        _ => "/",
    }
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn parse_u64(value: Option<Value>) -> Option<u64> {
    match value {
        Some(Value::Number(number)) => number.as_u64(),
        Some(Value::String(text)) => text.trim().parse::<u64>().ok(),
        _ => None,
    }
}

fn parse_i64(value: Option<Value>) -> Option<i64> {
    match value {
        Some(Value::Number(number)) => number.as_i64(),
        Some(Value::String(text)) => text.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn normalize_skill(raw: RawRegistrySkill) -> Option<RegistrySkill> {
    let skill_id = raw.skill_id?.trim().to_string();
    let source = raw.source?.trim().to_string();
    let name = raw.name?.trim().to_string();
    if skill_id.is_empty() || source.is_empty() || name.is_empty() {
        return None;
    }

    let id = raw
        .id
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .unwrap_or_else(|| format!("{source}/{skill_id}"));

    Some(RegistrySkill {
        id,
        skill_id,
        name,
        source,
        installs: parse_u64(raw.installs).unwrap_or(0),
        installs_yesterday: parse_u64(raw.installs_yesterday),
        change: parse_i64(raw.change),
        description: raw
            .description
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty()),
    })
}

fn decode_candidate(candidate: &str) -> Option<RegistrySkill> {
    let raw: RawRegistrySkill = serde_json::from_str(candidate).ok()?;
    normalize_skill(raw)
}

fn parse_leaderboard_html(html: &str) -> Result<Vec<RegistrySkill>> {
    let mut parsed = Vec::<RegistrySkill>::new();
    let direct_regex =
        Regex::new(r#"\{[^}]*"skillId"\s*:\s*"[^"]+"[^}]*"installs"\s*:\s*\d+[^}]*\}"#)?;

    for matched in direct_regex.find_iter(html) {
        let json_text = matched.as_str();
        if let Some(skill) = decode_candidate(json_text) {
            parsed.push(skill);
            continue;
        }

        let unescaped = json_text.replace("\\\"", "\"").replace("\\\\/", "/");
        if let Some(skill) = decode_candidate(&unescaped) {
            parsed.push(skill);
        }
    }

    if parsed.is_empty() {
        let escaped_regex = Regex::new(r#"\{(?:[^{}]|\\[{}])*\\?"skillId\\?"[^}]*\}"#)?;
        for matched in escaped_regex.find_iter(html) {
            let block = matched
                .as_str()
                .replace("\\\"", "\"")
                .replace("\\\\/", "/")
                .replace("\\\\", "\\");

            if let Some(skill) = decode_candidate(&block) {
                parsed.push(skill);
            }
        }
    }

    let mut seen = HashSet::<String>::new();
    let mut deduped = Vec::<RegistrySkill>::new();
    for skill in parsed {
        if seen.insert(skill.id.clone()) {
            deduped.push(skill);
        }
    }

    deduped.sort_by_key(|s| std::cmp::Reverse(s.installs));
    if deduped.is_empty() {
        bail!("no skills found in leaderboard response");
    }

    Ok(deduped)
}

fn build_raw_url(source: &str, path: &str, branch: &str) -> String {
    let trimmed_path = path.trim_matches('/');
    let file_path = if trimmed_path.is_empty() {
        "SKILL.md".to_string()
    } else {
        format!("{trimmed_path}/SKILL.md")
    };
    format!("https://raw.githubusercontent.com/{source}/{branch}/{file_path}")
}

fn candidate_raw_urls(source: &str, skill_id: &str) -> Vec<String> {
    let paths = vec![
        skill_id.to_string(),
        format!("skills/{skill_id}"),
        format!("plugin/skills/{skill_id}"),
        format!("plugins/skills/{skill_id}"),
        format!(".claude/skills/{skill_id}"),
        String::new(),
    ];
    let branches = ["main", "master"];

    let mut urls = Vec::<String>::new();
    for branch in branches {
        for path in &paths {
            urls.push(build_raw_url(source, path, branch));
        }
    }
    urls
}

fn fetch_skill_md_from_url(client: &Client, url: &str) -> Result<Option<String>> {
    let response = client
        .get(url)
        .header(ACCEPT, "text/plain")
        .header(USER_AGENT, APP_USER_AGENT)
        .send()?;
    let status = response.status().as_u16();

    if status == 404 {
        return Ok(None);
    }
    if status != 200 {
        bail!("registry skill content request failed ({status})");
    }
    Ok(Some(response.text()?))
}

fn fetch_tree_api_paths(client: &Client, source: &str, branch: &str) -> Result<Vec<String>> {
    let api_url = format!("https://api.github.com/repos/{source}/git/trees/{branch}?recursive=1");
    let response = client
        .get(api_url)
        .header(ACCEPT, "application/vnd.github.v3+json")
        .header(USER_AGENT, APP_USER_AGENT)
        .send()?;
    let status = response.status().as_u16();

    if status == 404 {
        return Ok(Vec::new());
    }
    if status != 200 {
        bail!("github tree API request failed ({status})");
    }

    let payload: GitTreeResponse = response.json()?;
    Ok(payload
        .tree
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            if entry.entry_type.as_deref() != Some("blob") {
                return None;
            }
            let path = entry.path?;
            if path.trim().is_empty() {
                return None;
            }
            Some(path)
        })
        .collect())
}

fn content_matches_skill_id(content: &str, skill_id: &str) -> bool {
    let expected = skill_id.trim().to_lowercase();
    if expected.is_empty() {
        return false;
    }

    let mut in_frontmatter = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            }
            break;
        }
        if !in_frontmatter {
            continue;
        }
        let Some((key, value)) = trimmed.split_once(':') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("name") {
            continue;
        }

        let normalized_value = value
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .to_lowercase();
        return normalized_value == expected;
    }
    false
}

fn tree_candidate_score(path: &str, skill_id: &str) -> i32 {
    let normalized_path = path.to_lowercase();
    let normalized_skill_id = skill_id.to_lowercase();
    let mut score = 0;

    if normalized_path.contains(&normalized_skill_id) {
        score += 4;
    }
    if normalized_path.starts_with(&format!("{normalized_skill_id}/")) {
        score += 2;
    }
    if normalized_path.starts_with("skills/") {
        score += 1;
    }
    if normalized_path.starts_with(".claude/skills/") {
        score += 1;
    }
    if normalized_path == "skill.md" {
        score += 1;
    }

    score
}

fn discover_via_tree_api(
    client: &Client,
    source: &str,
    skill_id: &str,
) -> Result<Option<(String, String)>> {
    let mut best_candidate: Option<(i32, String, String)> = None;

    for branch in ["main", "master"] {
        let mut skill_paths = fetch_tree_api_paths(client, source, branch)?
            .into_iter()
            .filter(|path| path.ends_with("SKILL.md"))
            .collect::<Vec<String>>();
        if skill_paths.is_empty() {
            continue;
        }
        skill_paths.sort();

        for path in skill_paths {
            let dir_path = if path == "SKILL.md" {
                String::new()
            } else {
                path.trim_end_matches("/SKILL.md").to_string()
            };
            let raw_url = build_raw_url(source, &dir_path, branch);

            let Some(content) = fetch_skill_md_from_url(client, &raw_url)? else {
                continue;
            };
            if content_matches_skill_id(&content, skill_id) {
                return Ok(Some((content, raw_url)));
            }

            let score = tree_candidate_score(&path, skill_id);
            match &best_candidate {
                Some((best_score, _, _)) if score <= *best_score => {}
                _ => best_candidate = Some((score, content, raw_url)),
            }
        }
    }

    Ok(best_candidate.map(|(_, content, url)| (content, url)))
}

fn parse_skill_md_content(content: &str) -> Result<(serde_json::Value, String)> {
    if !content.starts_with("---\n") {
        return Ok((
            serde_json::Value::Object(Default::default()),
            content.to_string(),
        ));
    }

    let mut yaml_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut lines = content.lines();
    let _first_line = lines.next();

    let mut in_frontmatter = true;
    let mut found_end_marker = false;
    for line in lines {
        if in_frontmatter && line.trim() == "---" {
            in_frontmatter = false;
            found_end_marker = true;
            continue;
        }

        if in_frontmatter {
            yaml_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    if !found_end_marker {
        return Ok((
            serde_json::Value::Object(Default::default()),
            content.to_string(),
        ));
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

fn create_temp_dir() -> Result<PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| anyhow!("system time error: {error}"))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("skm-registry-{nanos}"));
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
        if stderr.is_empty() {
            bail!("git {} failed", args.join(" "));
        }
        bail!("git {} failed: {stderr}", args.join(" "));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn normalize_repo_subpath(input: &str) -> String {
    input
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
}

fn candidate_repo_subpaths(skill_id: &str) -> Vec<String> {
    let normalized = normalize_repo_subpath(skill_id);
    let mut seen = HashSet::<String>::new();
    let candidates = vec![
        normalized.clone(),
        format!("skills/{normalized}"),
        format!("plugin/skills/{normalized}"),
        format!("plugins/skills/{normalized}"),
        format!(".claude/skills/{normalized}"),
        String::new(),
    ];

    let mut deduped = Vec::<String>::new();
    for candidate in candidates {
        let normalized_candidate = normalize_repo_subpath(&candidate);
        if seen.insert(normalized_candidate.clone()) {
            deduped.push(normalized_candidate);
        }
    }
    deduped
}

fn build_repo_dir_path(repo_root: &Path, subpath: &str) -> Option<PathBuf> {
    let mut path = repo_root.to_path_buf();
    let normalized = normalize_repo_subpath(subpath);
    for segment in normalized.split('/') {
        let trimmed = segment.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "." || trimmed == ".." {
            return None;
        }
        path.push(trimmed);
    }
    Some(path)
}

fn read_skill_md_from_repo_subpath(
    repo_root: &Path,
    subpath: &str,
) -> Result<Option<(String, String)>> {
    let Some(skill_dir) = build_repo_dir_path(repo_root, subpath) else {
        return Ok(None);
    };
    let skill_md = skill_dir.join(SKILL_FILE_NAME);
    if !skill_md.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&skill_md)?;
    let relative_dir = normalize_repo_subpath(subpath);
    let relative_path = if relative_dir.is_empty() {
        SKILL_FILE_NAME.to_string()
    } else {
        format!("{relative_dir}/{SKILL_FILE_NAME}")
    };
    Ok(Some((content, relative_path)))
}

fn discover_skill_md_paths(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut found = Vec::<PathBuf>::new();
    discover_skill_md_paths_inner(root, root, 0, max_depth, &mut found)?;
    Ok(found)
}

fn discover_skill_md_paths_inner(
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
    if skill_md.is_file() {
        found.push(skill_md);
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
        discover_skill_md_paths_inner(root, &path, depth + 1, max_depth, found)?;
    }
    Ok(())
}

fn discover_skill_md_from_repo(
    repo_root: &Path,
    skill_id: &str,
) -> Result<Option<(String, String)>> {
    let mut skill_files = discover_skill_md_paths(repo_root, 8)?;
    if skill_files.is_empty() {
        return Ok(None);
    }
    skill_files.sort_by(|left, right| left.to_string_lossy().cmp(&right.to_string_lossy()));

    let mut best_candidate: Option<(i32, String, String)> = None;
    for skill_file in skill_files {
        if !skill_file.is_file() {
            continue;
        }

        let relative_path = skill_file
            .strip_prefix(repo_root)
            .unwrap_or(&skill_file)
            .to_string_lossy()
            .replace('\\', "/");
        let content = match fs::read_to_string(&skill_file) {
            Ok(content) => content,
            Err(_) => continue,
        };

        if content_matches_skill_id(&content, skill_id) {
            return Ok(Some((content, relative_path)));
        }

        let score = tree_candidate_score(&relative_path, skill_id);
        match &best_candidate {
            Some((best_score, _, _)) if score <= *best_score => {}
            _ => best_candidate = Some((score, content, relative_path)),
        }
    }

    Ok(best_candidate.map(|(_, content, path)| (content, path)))
}

fn fetch_skill_content_from_repository_url(
    repo_url: &str,
    skill_id: &str,
) -> Result<Option<(String, String)>> {
    let temp = create_temp_dir()?;
    let temp_path = temp.to_string_lossy().to_string();

    let result = (|| -> Result<Option<(String, String)>> {
        run_git(&["clone", "--depth=1", repo_url, &temp_path], None)
            .context("failed to clone repository")?;

        for subpath in candidate_repo_subpaths(skill_id) {
            if let Some((content, relative_path)) =
                read_skill_md_from_repo_subpath(&temp, &subpath)?
            {
                let fetched_from = format!("{repo_url}#{relative_path}");
                return Ok(Some((content, fetched_from)));
            }
        }

        if let Some((content, relative_path)) = discover_skill_md_from_repo(&temp, skill_id)? {
            let fetched_from = format!("{repo_url}#{relative_path}");
            return Ok(Some((content, fetched_from)));
        }

        Ok(None)
    })();

    let _cleanup = fs::remove_dir_all(&temp);
    result
}

fn skill_matches_keyword(content: &str, dir_path: &str, keyword: &str) -> bool {
    let kw = keyword.trim().to_lowercase();
    if kw.is_empty() {
        return true;
    }

    let dir_name = dir_path
        .split('/')
        .next_back()
        .unwrap_or(dir_path)
        .to_lowercase();
    if dir_name.contains(&kw) {
        return true;
    }

    let mut in_frontmatter = false;
    let mut found_start = false;
    for line in content.lines().take(60) {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !found_start {
                found_start = true;
                in_frontmatter = true;
                continue;
            } else {
                break;
            }
        }
        if !in_frontmatter {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let k = key.trim().to_lowercase();
            if matches!(k.as_str(), "name" | "description" | "tags")
                && value.trim().to_lowercase().contains(&kw)
            {
                return true;
            }
        }
    }
    false
}

fn skill_from_git_content(content: &str, source: &str, dir_path: &str) -> Option<RegistrySkill> {
    let (frontmatter, _) = parse_skill_md_content(content).ok()?;

    let name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            dir_path
                .split('/')
                .next_back()
                .unwrap_or(dir_path)
                .to_string()
        });

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let skill_id = if dir_path.is_empty() {
        name.to_lowercase().replace(' ', "-")
    } else {
        dir_path.to_string()
    };

    Some(RegistrySkill {
        id: format!("{source}/{skill_id}"),
        skill_id,
        name,
        source: source.to_string(),
        installs: 0,
        installs_yesterday: None,
        change: None,
        description,
    })
}

fn sanitize_url_for_cache_key(url: &str) -> String {
    let s = url
        .trim()
        .trim_start_matches("ssh://git@")
        .trim_start_matches("git@")
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("git://");
    let s = s.trim_end_matches(".git").trim_end_matches('/');

    let mut result = String::with_capacity(s.len());
    let mut prev_dash = false;
    for c in s.chars() {
        let safe = if c.is_alphanumeric() || c == '.' {
            c
        } else {
            '-'
        };
        if safe == '-' {
            if !prev_dash {
                result.push('-');
            }
            prev_dash = true;
        } else {
            result.push(safe);
            prev_dash = false;
        }
    }
    result.trim_matches('-').to_string()
}

fn git_source_cache_dir(source_url: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("cannot resolve home directory"))?;
    let key = sanitize_url_for_cache_key(source_url);
    Ok(home.join(".agents").join(".skm-source-cache").join(key))
}

fn is_cache_fresh(cache_dir: &Path) -> bool {
    let fetch_head = cache_dir.join(".git").join("FETCH_HEAD");
    let head = cache_dir.join(".git").join("HEAD");
    let marker = if fetch_head.exists() {
        fetch_head
    } else {
        head
    };
    fs::metadata(&marker)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .map(|age| age.as_secs() < GIT_SOURCE_CACHE_TTL_SECS)
        .unwrap_or(false)
}

fn ensure_git_source_cache(clone_url: &str, cache_dir: &Path) -> Result<()> {
    if cache_dir.join(".git").is_dir() {
        if !is_cache_fresh(cache_dir) {
            let _ = run_git(&["fetch", "--depth=1", "origin"], Some(cache_dir));
            let _ = run_git(&["reset", "--hard", "FETCH_HEAD"], Some(cache_dir));
        }
        return Ok(());
    }

    if cache_dir.exists() {
        let _ = fs::remove_dir_all(cache_dir);
    }
    fs::create_dir_all(cache_dir)?;
    run_git(
        &[
            "clone",
            "--depth=1",
            clone_url,
            &cache_dir.to_string_lossy(),
        ],
        None,
    )
    .context("failed to clone repository")?;
    Ok(())
}

fn search_in_dir(
    repo_dir: &Path,
    source_label: &str,
    keyword: &str,
    limit: usize,
) -> Result<Vec<RegistrySkill>> {
    let skill_files = discover_skill_md_paths(repo_dir, 8)?;
    let mut results = Vec::new();

    for skill_file in skill_files {
        let raw_relative = skill_file
            .strip_prefix(repo_dir)
            .unwrap_or(&skill_file)
            .to_string_lossy()
            .replace('\\', "/");
        let relative_path = raw_relative.trim_start_matches('/');

        let dir_path = if relative_path.eq_ignore_ascii_case("SKILL.md") {
            String::new()
        } else {
            relative_path
                .strip_suffix("/SKILL.md")
                .unwrap_or(relative_path)
                .to_string()
        };

        let content = match fs::read_to_string(&skill_file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if !skill_matches_keyword(&content, &dir_path, keyword) {
            continue;
        }

        if let Some(skill) = skill_from_git_content(&content, source_label, &dir_path) {
            results.push(skill);
        }

        if results.len() >= limit {
            break;
        }
    }

    Ok(results)
}

pub fn search_local_source(
    source_path: &str,
    keyword: &str,
    limit: usize,
) -> Result<Vec<RegistrySkill>> {
    use crate::core::config::expand_tilde;
    let expanded = expand_tilde(source_path.trim());
    let dir = std::path::Path::new(&expanded);
    if !dir.exists() {
        anyhow::bail!("local source path does not exist: {expanded}");
    }
    if !dir.is_dir() {
        anyhow::bail!("local source path is not a directory: {expanded}");
    }
    search_in_dir(dir, source_path.trim(), keyword, limit)
}

pub fn search_git_source(
    source_url: &str,
    keyword: &str,
    limit: usize,
) -> Result<Vec<RegistrySkill>> {
    let trimmed = source_url.trim();
    let parsed = parse_skill_content_source(trimmed)?;

    let clone_url = match &parsed {
        SkillContentSource::GitHubSource(owner_repo) => {
            if trimmed.contains("://") || trimmed.starts_with("git@") {
                trimmed.to_string()
            } else {
                format!("https://github.com/{owner_repo}.git")
            }
        }
        SkillContentSource::GitRepositoryUrl(url) => url.clone(),
    };

    let cache_dir = git_source_cache_dir(trimmed)?;
    ensure_git_source_cache(&clone_url, &cache_dir)?;
    search_in_dir(&cache_dir, trimmed, keyword, limit)
}

pub fn search_skills(registry_url: &str, query: &str, limit: usize) -> Result<Vec<RegistrySkill>> {
    let base = normalize_base_url(registry_url)?;
    let url = format!("{base}/api/search");
    let response = build_client()?
        .get(url)
        .query(&[("q", query), ("limit", &limit.to_string())])
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, APP_USER_AGENT)
        .send()?;

    if !response.status().is_success() {
        bail!(
            "registry search request failed ({})",
            response.status().as_u16()
        );
    }

    let payload: SearchResponse = response.json()?;
    let mut skills = payload
        .skills
        .unwrap_or_default()
        .into_iter()
        .filter_map(normalize_skill)
        .collect::<Vec<RegistrySkill>>();

    if is_skills_sh_registry(&base) {
        skills.sort_by(|left, right| {
            right
                .installs
                .cmp(&left.installs)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
    }
    Ok(skills)
}

pub fn fetch_leaderboard(
    registry_url: &str,
    category: &str,
    force_refresh: bool,
) -> Result<Vec<RegistrySkill>> {
    let base = normalize_base_url(registry_url)?;
    let normalized_category = normalize_category(category)?;
    let cache_key = format!("{base}::{normalized_category}");

    if !force_refresh {
        let now = now_unix_secs();
        if let Ok(guard) = cache_store().lock() {
            if let Some(entry) = guard.get(&cache_key) {
                if now.saturating_sub(entry.fetched_at) < LEADERBOARD_TTL_SECS {
                    return Ok(entry.skills.clone());
                }
            }
        }
    }

    let page_url = format!("{base}{}", category_path(normalized_category));
    let response = build_client()?
        .get(page_url)
        .header(ACCEPT, "text/html,application/xhtml+xml")
        .header(USER_AGENT, APP_USER_AGENT)
        .send()?;

    if !response.status().is_success() {
        bail!(
            "registry leaderboard request failed ({})",
            response.status().as_u16()
        );
    }

    let html = response.text()?;
    let skills = parse_leaderboard_html(&html)?;
    if let Ok(mut guard) = cache_store().lock() {
        guard.insert(
            cache_key,
            CacheEntry {
                skills: skills.clone(),
                fetched_at: now_unix_secs(),
            },
        );
    }
    Ok(skills)
}

pub fn fetch_skill_content(
    source: &str,
    skill_id: &str,
    force_refresh: bool,
) -> Result<RegistrySkillContent> {
    let source = parse_skill_content_source(source)?;
    let skill_id = normalize_skill_id(skill_id)?;
    let source_cache_key = match &source {
        SkillContentSource::GitHubSource(repo) => repo.clone(),
        SkillContentSource::GitRepositoryUrl(repo_url) => repo_url.clone(),
    };
    let cache_key = format!("{source_cache_key}/{skill_id}");

    if !force_refresh {
        let now = now_unix_secs();
        if let Ok(guard) = content_cache_store().lock() {
            if let Some(entry) = guard.get(&cache_key) {
                if now.saturating_sub(entry.fetched_at) < SKILL_CONTENT_TTL_SECS {
                    return Ok(entry.content.clone());
                }
            }
        }
    }

    let (resolved_source, raw_content, fetched_from) = match &source {
        SkillContentSource::GitHubSource(repo_source) => {
            let client = build_client()?;
            let mut raw_content: Option<String> = None;
            let mut fetched_from: Option<String> = None;

            for url in candidate_raw_urls(repo_source, &skill_id) {
                if let Some(content) = fetch_skill_md_from_url(&client, &url)? {
                    raw_content = Some(content);
                    fetched_from = Some(url);
                    break;
                }
            }

            if raw_content.is_none() {
                if let Some((content, url)) =
                    discover_via_tree_api(&client, repo_source, &skill_id)?
                {
                    raw_content = Some(content);
                    fetched_from = Some(url);
                }
            }

            let raw_content =
                raw_content.ok_or_else(|| anyhow!("SKILL.md not found in repository"))?;
            (repo_source.clone(), raw_content, fetched_from)
        }
        SkillContentSource::GitRepositoryUrl(repo_url) => {
            let fetched = fetch_skill_content_from_repository_url(repo_url, &skill_id)?
                .ok_or_else(|| anyhow!("SKILL.md not found in repository"))?;
            (repo_url.clone(), fetched.0, Some(fetched.1))
        }
    };

    let (frontmatter, markdown_body) = match parse_skill_md_content(&raw_content) {
        Ok(parsed) => parsed,
        Err(_) => (
            serde_json::Value::Object(Default::default()),
            raw_content.trim().to_string(),
        ),
    };

    let content = RegistrySkillContent {
        source: resolved_source,
        skill_id,
        frontmatter,
        markdown_body,
        fetched_from,
    };

    if let Ok(mut guard) = content_cache_store().lock() {
        guard.insert(
            cache_key,
            ContentCacheEntry {
                content: content.clone(),
                fetched_at: now_unix_secs(),
            },
        );
    }

    Ok(content)
}
