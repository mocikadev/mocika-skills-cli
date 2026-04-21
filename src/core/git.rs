use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};

pub fn ensure_git_available() -> Result<()> {
    if which::which("git").is_ok() {
        return Ok(());
    }

    Err(anyhow!(
        "git is required but was not found in PATH; please install git first"
    ))
}

pub fn clone(url: &str, dest: &Path) -> Result<()> {
    ensure_git_available()?;
    let dest_text = dest.to_string_lossy().to_string();
    run_git(&["clone", "--depth=1", url, &dest_text], None).map(|_| ())
}

pub fn pull(repo_dir: &Path) -> Result<()> {
    ensure_git_available()?;
    run_git(&["pull"], Some(repo_dir)).map(|_| ())
}

pub fn current_commit(repo_dir: &Path) -> Result<String> {
    ensure_git_available()?;
    run_git(&["rev-parse", "HEAD"], Some(repo_dir))
}

pub fn remote_commit(url: &str) -> Result<String> {
    ensure_git_available()?;
    let output = run_git(&["ls-remote", url, "HEAD"], None)?;
    output
        .split_whitespace()
        .next()
        .map(std::string::ToString::to_string)
        .ok_or_else(|| anyhow!("failed to parse remote HEAD for {url}"))
}

pub fn is_git_repo(dir: &Path) -> bool {
    dir.join(".git").exists()
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
