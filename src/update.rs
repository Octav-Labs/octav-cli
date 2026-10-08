use std::ffi::OsStr;
use std::fs;
use std::io::{ErrorKind, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config;
use crate::error::OctavError;

const REPO: &str = "Octav-Labs/octav-cli";
const INSTALL_SCRIPT: &str =
    "curl -sSf https://raw.githubusercontent.com/Octav-Labs/octav-cli/main/install.sh | sh";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CHECK_INTERVAL_SECS: u64 = 24 * 60 * 60;

// Separate from OctavClient so the Octav API key is never sent to GitHub
fn http_client(timeout: Duration) -> Result<Client, OctavError> {
    Client::builder()
        .user_agent(concat!("octav-cli/", env!("CARGO_PKG_VERSION")))
        .timeout(timeout)
        .build()
        .map_err(|e| OctavError::Network(format!("Failed to create HTTP client: {}", e)))
}

/// Tag of the latest GitHub release, e.g. "v0.3.0"
fn latest_tag(timeout: Duration) -> Result<String, OctavError> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let response = http_client(timeout)?
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| OctavError::Network(format!("Failed to check for updates: {}", e)))?;
    if !response.status().is_success() {
        return Err(OctavError::Network(format!(
            "Failed to check for updates: GitHub returned {}",
            response.status()
        )));
    }
    let release: Value = response
        .json()
        .map_err(|e| OctavError::Network(format!("Failed to read release info: {}", e)))?;
    release["tag_name"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| OctavError::Network("Release info has no tag_name".to_string()))
}

/// Parse "v1.2.3" or "1.2.3", ignoring any pre-release or build suffix
fn parse_version(version: &str) -> Option<(u64, u64, u64)> {
    let core = version
        .trim()
        .trim_start_matches('v')
        .split(['-', '+'])
        .next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
    let parsed = (parts.next()??, parts.next()??, parts.next()??);
    if parts.next().is_some() {
        return None;
    }
    Some(parsed)
}

fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

/// Release asset target for a platform, matching the names install.sh downloads
fn target_for(os: &str, arch: &str) -> Option<String> {
    let os = match os {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        _ => return None,
    };
    match arch {
        "x86_64" | "aarch64" => Some(format!("{}-{}", arch, os)),
        _ => None,
    }
}

/// If the binary is managed by something other than `octav update`, what to run instead
fn other_install_method(exe: &Path, cargo_bin: Option<&Path>) -> Option<String> {
    if cargo_bin.is_some_and(|bin| exe.starts_with(bin)) {
        return Some("octav was installed with cargo. Run: cargo install octav".to_string());
    }
    let path = exe.to_string_lossy();
    if path.contains("/target/debug/") || path.contains("/target/release/") {
        return Some("This is a source build. Run: git pull && cargo build --release".to_string());
    }
    None
}

fn cargo_bin_dir() -> Option<PathBuf> {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".cargo")))?;
    let bin = home.join("bin");
    Some(bin.canonicalize().unwrap_or(bin))
}

/// Download a release archive and return the bytes of the `octav` binary inside it
fn download_binary(url: &str) -> Result<Vec<u8>, OctavError> {
    let response = http_client(Duration::from_secs(120))?
        .get(url)
        .send()
        .map_err(|e| OctavError::Update(format!("Failed to download {}: {}", url, e)))?;
    if !response.status().is_success() {
        return Err(OctavError::Update(format!(
            "Failed to download {}: HTTP {}",
            url,
            response.status()
        )));
    }
    let bytes = response
        .bytes()
        .map_err(|e| OctavError::Update(format!("Failed to download {}: {}", url, e)))?;

    let read_error =
        |e: std::io::Error| OctavError::Update(format!("Invalid release archive: {}", e));
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(&bytes[..]));
    for entry in archive.entries().map_err(read_error)? {
        let mut entry = entry.map_err(read_error)?;
        if entry.path().map_err(read_error)?.file_name() == Some(OsStr::new("octav")) {
            let mut binary = Vec::new();
            entry.read_to_end(&mut binary).map_err(read_error)?;
            return Ok(binary);
        }
    }
    Err(OctavError::Update(
        "Release archive does not contain an octav binary".to_string(),
    ))
}

fn write_executable(path: &Path, data: &[u8]) -> std::io::Result<()> {
    fs::write(path, data)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}

fn install_error(e: std::io::Error, dir: &Path) -> OctavError {
    if e.kind() == ErrorKind::PermissionDenied {
        OctavError::Update(format!(
            "No permission to write to {}. Re-run with sudo, or reinstall with: {}",
            dir.display(),
            INSTALL_SCRIPT
        ))
    } else {
        OctavError::Update(format!("Failed to install update: {}", e))
    }
}

/// Replace `exe` with `binary`. The new file is written next to it, checked to run,
/// then renamed over it, so a failure at any step leaves the current binary untouched.
fn install_binary(binary: &[u8], exe: &Path) -> Result<(), OctavError> {
    let dir = exe.parent().ok_or_else(|| {
        OctavError::Update("Could not locate the octav binary's directory".to_string())
    })?;
    let tmp = dir.join(format!(".octav-update-{}", std::process::id()));

    if let Err(e) = write_executable(&tmp, binary) {
        let _ = fs::remove_file(&tmp);
        return Err(install_error(e, dir));
    }
    let runs = Command::new(&tmp)
        .arg("--help")
        .output()
        .is_ok_and(|o| o.status.success());
    if !runs {
        let _ = fs::remove_file(&tmp);
        return Err(OctavError::Update(
            "The downloaded binary failed to run; nothing was changed".to_string(),
        ));
    }
    fs::rename(&tmp, exe).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        install_error(e, dir)
    })
}

pub fn check() -> Result<Value, OctavError> {
    let tag = latest_tag(Duration::from_secs(10))?;
    let latest = tag.trim_start_matches('v');
    save_cache(Some(latest));
    Ok(json!({
        "current": CURRENT_VERSION,
        "latest": latest,
        "update_available": is_newer(latest, CURRENT_VERSION)
    }))
}

pub fn update() -> Result<Value, OctavError> {
    let tag = latest_tag(Duration::from_secs(10))?;
    let latest = tag.trim_start_matches('v');
    save_cache(Some(latest));

    if !is_newer(latest, CURRENT_VERSION) {
        return Ok(json!({
            "status": "ok",
            "message": format!("octav {} is already the latest version", CURRENT_VERSION),
            "current": CURRENT_VERSION,
            "latest": latest
        }));
    }

    let exe = std::env::current_exe()
        .and_then(|p| p.canonicalize())
        .map_err(|e| OctavError::Update(format!("Could not locate the octav binary: {}", e)))?;
    if let Some(instead) = other_install_method(&exe, cargo_bin_dir().as_deref()) {
        return Err(OctavError::Update(format!(
            "octav {} is available. {}",
            latest, instead
        )));
    }

    let target = target_for(std::env::consts::OS, std::env::consts::ARCH).ok_or_else(|| {
        OctavError::Update(format!(
            "No prebuilt binary for {} {}. Build from source instead.",
            std::env::consts::OS,
            std::env::consts::ARCH
        ))
    })?;
    let url = format!(
        "https://github.com/{}/releases/download/{}/octav-{}.tar.gz",
        REPO, tag, target
    );

    let binary = download_binary(&url)?;
    install_binary(&binary, &exe)?;

    Ok(json!({
        "status": "ok",
        "message": format!("Updated octav {} → {}", CURRENT_VERSION, latest),
        "previous": CURRENT_VERSION,
        "current": latest,
        "path": exe.to_string_lossy()
    }))
}

// Background check: at most once a day, only for people at a terminal

#[derive(Serialize, Deserialize)]
struct CheckCache {
    checked_at: u64,
    latest: Option<String>,
}

fn cache_path() -> PathBuf {
    config::config_dir().join("update-check.json")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Record a check, including failed ones, so an offline machine doesn't retry every run
fn save_cache(latest: Option<&str>) {
    let cache = CheckCache {
        checked_at: now(),
        latest: latest.map(String::from),
    };
    if let Ok(json) = serde_json::to_string(&cache) {
        let _ = fs::create_dir_all(config::config_dir());
        let _ = fs::write(cache_path(), json);
    }
}

fn check_due() -> bool {
    fs::read_to_string(cache_path())
        .ok()
        .and_then(|s| serde_json::from_str::<CheckCache>(&s).ok())
        .is_none_or(|c| now().saturating_sub(c.checked_at) >= CHECK_INTERVAL_SECS)
}

/// Skipped for scripts, cron and agents (stderr is not a terminal), in CI, and when
/// OCTAV_NO_UPDATE_CHECK is set
fn background_check_enabled() -> bool {
    let opted_out = std::env::var_os("OCTAV_NO_UPDATE_CHECK").is_some_and(|v| !v.is_empty());
    !opted_out && std::env::var_os("CI").is_none() && std::io::stderr().is_terminal()
}

pub struct UpdateCheck(mpsc::Receiver<String>);

/// Start the daily update check in the background, if one is due
pub fn start_background_check() -> Option<UpdateCheck> {
    if !background_check_enabled() || !check_due() {
        return None;
    }
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || match latest_tag(Duration::from_secs(3)) {
        Ok(tag) => {
            let latest = tag.trim_start_matches('v');
            save_cache(Some(latest));
            if is_newer(latest, CURRENT_VERSION) {
                let _ = tx.send(format!(
                    "A new version of octav is available: {} → {}\n\
                     Run `octav update` to upgrade (set OCTAV_NO_UPDATE_CHECK=1 to turn off this check)",
                    CURRENT_VERSION, latest
                ));
            }
        }
        Err(_) => save_cache(None),
    });
    Some(UpdateCheck(rx))
}

impl UpdateCheck {
    /// Print the notice to stderr if the check found a newer version. Waits at most
    /// briefly for a check that is still running, so a slow network never holds up exit.
    pub fn finish(self) {
        if let Ok(notice) = self.0.recv_timeout(Duration::from_millis(500)) {
            eprintln!("\n{}", notice);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("v0.2.0"), Some((0, 2, 0)));
        assert_eq!(parse_version("1.10.3"), Some((1, 10, 3)));
        assert_eq!(parse_version("0.3.0-beta.1"), Some((0, 3, 0)));
        assert_eq!(parse_version("1.2"), None);
        assert_eq!(parse_version("1.2.3.4"), None);
        assert_eq!(parse_version("latest"), None);
    }

    #[test]
    fn test_is_newer() {
        assert!(is_newer("0.3.0", "0.2.0"));
        assert!(is_newer("v1.0.0", "0.9.9"));
        assert!(is_newer("0.10.0", "0.9.0")); // numeric, not string, comparison
        assert!(!is_newer("0.2.0", "0.2.0"));
        assert!(!is_newer("0.1.9", "0.2.0"));
        assert!(!is_newer("garbage", "0.2.0"));
    }

    #[test]
    fn test_target_for() {
        assert_eq!(
            target_for("macos", "aarch64").as_deref(),
            Some("aarch64-apple-darwin")
        );
        assert_eq!(
            target_for("linux", "x86_64").as_deref(),
            Some("x86_64-unknown-linux-gnu")
        );
        assert_eq!(target_for("windows", "x86_64"), None);
        assert_eq!(target_for("linux", "riscv64"), None);
    }

    #[test]
    fn test_other_install_method() {
        let cargo_bin = Path::new("/home/u/.cargo/bin");
        let cargo = other_install_method(Path::new("/home/u/.cargo/bin/octav"), Some(cargo_bin));
        assert!(cargo.unwrap().contains("cargo install octav"));

        let source = other_install_method(
            Path::new("/home/u/octav-cli/target/release/octav"),
            Some(cargo_bin),
        );
        assert!(source.unwrap().contains("cargo build --release"));

        assert_eq!(
            other_install_method(Path::new("/usr/local/bin/octav"), Some(cargo_bin)),
            None
        );
    }

    #[test]
    fn test_install_binary_replaces_file() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("octav");
        fs::write(&exe, b"old").unwrap();

        install_binary(b"#!/bin/sh\nexit 0\n", &exe).unwrap();
        assert_eq!(fs::read(&exe).unwrap(), b"#!/bin/sh\nexit 0\n");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1); // temp file cleaned up
    }

    #[test]
    fn test_install_binary_keeps_old_file_if_new_one_fails() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("octav");
        fs::write(&exe, b"old").unwrap();

        assert!(install_binary(b"#!/bin/sh\nexit 1\n", &exe).is_err());
        assert_eq!(fs::read(&exe).unwrap(), b"old");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
