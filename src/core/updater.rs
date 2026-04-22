use anyhow::{bail, Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::time::Duration;

const REPO: &str = "mocikadev/mocika-skills-cli";
const GITHUB_API: &str = "https://api.github.com";

const API_TIMEOUT: Duration = Duration::from_secs(30);
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(300);

/// GitHub release metadata returned from the API.
#[derive(Debug)]
pub struct ReleaseInfo {
    /// e.g. "v0.2.0"
    pub tag: String,
    /// e.g. "0.2.0" (tag stripped of leading "v")
    pub version: String,
    /// Download URL for this platform's binary
    pub binary_url: String,
    /// Download URL for SHA256SUMS.txt
    pub checksum_url: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Returns the user-friendly platform identifier used in release asset names,
/// or `None` for unsupported platforms.
///
/// Matches the naming convention in the release workflow:
/// `linux-amd64`, `linux-arm64`, `macos-amd64`, `macos-arm64`.
pub fn current_target() -> Option<&'static str> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("linux-amd64")
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        Some("linux-arm64")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("macos-amd64")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("macos-arm64")
    } else {
        None
    }
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.trim_start_matches('v');
    let parts: Vec<&str> = v.splitn(3, '.').collect();
    Some((
        parts.first()?.parse().ok()?,
        parts.get(1)?.parse().ok()?,
        parts.get(2)?.parse().ok()?,
    ))
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// Check GitHub for a newer release.
///
/// Returns `Ok(None)` when the installed version is already up-to-date.
///
/// # Errors
///
/// Propagates network errors or JSON parse failures, and bails when the
/// running platform is not supported or required release assets are missing.
pub fn check_update() -> Result<Option<ReleaseInfo>> {
    let target = current_target().context("unsupported platform for self-update")?;

    let client = Client::builder()
        .user_agent(format!("skm/{}", env!("CARGO_PKG_VERSION")))
        .timeout(API_TIMEOUT)
        .build()
        .context("failed to build HTTP client")?;

    let url = format!("{GITHUB_API}/repos/{REPO}/releases/latest");
    let release: Release = client
        .get(&url)
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .with_context(|| format!("failed to reach GitHub API: {url}"))?
        .error_for_status()
        .context("GitHub API returned error status")?
        .json()
        .context("failed to deserialise GitHub release JSON")?;

    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version = env!("CARGO_PKG_VERSION");

    if !is_newer(latest_version, current_version) {
        return Ok(None);
    }

    let binary_name = format!("skm-{target}");
    let binary_url = release
        .assets
        .iter()
        .find(|a| a.name == binary_name)
        .map(|a| a.browser_download_url.clone())
        .with_context(|| {
            format!(
                "release {} does not contain asset '{binary_name}'",
                release.tag_name
            )
        })?;

    let checksum_url = release
        .assets
        .iter()
        .find(|a| a.name == "SHA256SUMS.txt")
        .map(|a| a.browser_download_url.clone())
        .with_context(|| {
            format!(
                "release {} does not contain asset 'SHA256SUMS.txt'",
                release.tag_name
            )
        })?;

    Ok(Some(ReleaseInfo {
        tag: release.tag_name.clone(),
        version: latest_version.to_owned(),
        binary_url,
        checksum_url,
    }))
}

/// Download, verify, and atomically replace the running binary.
///
/// Returns the new version string on success.
///
/// # Errors
///
/// Propagates network, I/O, or checksum-mismatch errors.
pub fn apply_update(info: &ReleaseInfo) -> Result<String> {
    let target = current_target().context("unsupported platform for self-update")?;

    let client = Client::builder()
        .user_agent(format!("skm/{}", env!("CARGO_PKG_VERSION")))
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .context("failed to build HTTP client")?;

    // 1. Download binary bytes
    let binary_bytes = client
        .get(&info.binary_url)
        .send()
        .with_context(|| format!("failed to download binary from {}", info.binary_url))?
        .error_for_status()
        .context("binary download returned error status")?
        .bytes()
        .context("failed to read binary response body")?;

    // 2. Download SHA256SUMS.txt
    let checksum_text = client
        .get(&info.checksum_url)
        .send()
        .with_context(|| format!("failed to download SHA256SUMS from {}", info.checksum_url))?
        .error_for_status()
        .context("SHA256SUMS download returned error status")?
        .text()
        .context("failed to read SHA256SUMS response body")?;

    // 3. Parse SHA256SUMS — format: `<hex>  <filename>` (two spaces)
    let binary_name = format!("skm-{target}");
    let expected_hash = checksum_text
        .lines()
        .find_map(|line| {
            let mut parts = line.splitn(2, "  ");
            let hash = parts.next()?.trim();
            let name = parts.next()?.trim();
            if name == binary_name {
                Some(hash.to_owned())
            } else {
                None
            }
        })
        .with_context(|| format!("SHA256SUMS.txt does not contain an entry for '{binary_name}'"))?;

    // 4. Compute SHA256 of downloaded bytes
    let mut hasher = Sha256::new();
    hasher.update(&binary_bytes);
    let computed_hash = format!("{:x}", hasher.finalize());

    // 5. Compare — case-insensitive hex comparison
    if computed_hash.to_lowercase() != expected_hash.to_lowercase() {
        bail!(
            "SHA256 mismatch for '{binary_name}':\n  expected: {expected_hash}\n  computed: {computed_hash}"
        );
    }

    // 6. Locate running executable
    let exe_path = std::env::current_exe()
        .context("failed to determine current executable path")?
        .canonicalize()
        .context("failed to canonicalize current executable path")?;

    // 7. Write to a sibling temp file
    let tmp_path = {
        let mut p = exe_path.clone();
        let mut name = p
            .file_name()
            .context("executable path has no file name")?
            .to_os_string();
        name.push(".skm-update-tmp");
        p.set_file_name(name);
        p
    };

    fs::write(&tmp_path, &binary_bytes)
        .with_context(|| format!("failed to write temp binary to {}", tmp_path.display()))?;

    // 8. Set executable permission (Unix only — Windows not supported)
    // 9. Atomic rename: tmp → exe
    let install_result = (|| -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&tmp_path, perms)
                .with_context(|| format!("failed to chmod {}", tmp_path.display()))?;
        }
        fs::rename(&tmp_path, &exe_path).with_context(|| {
            format!(
                "failed to replace {} with {}",
                exe_path.display(),
                tmp_path.display()
            )
        })?;
        Ok(())
    })();
    if install_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    install_result?;

    Ok(info.version.clone())
}
