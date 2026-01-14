use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const GITHUB_OWNER: &str = "lyonbot";
const GITHUB_REPO: &str = "ralph-cli";

#[derive(Debug)]
pub enum UpgradeOutcome {
    UpToDate { current: Version },
    Upgraded { from: Version, to: Version },
}

#[derive(Debug)]
pub enum UpgradeError {
    UnsupportedPlatform { os: String, arch: String },
    Network(String),
    GithubApi(String),
    VersionParse { tag: String },
    AssetNotFound { asset: String },
    ChecksumParse,
    ChecksumMismatch { expected: String, actual: String },
    PermissionDenied { path: PathBuf },
    Io(io::Error),
}

impl std::fmt::Display for UpgradeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeError::UnsupportedPlatform { os, arch } => {
                write!(f, "Unsupported platform: {os} {arch}")
            }
            UpgradeError::Network(msg) => write!(f, "Network error: {msg}"),
            UpgradeError::GithubApi(msg) => write!(f, "GitHub API error: {msg}"),
            UpgradeError::VersionParse { tag } => write!(f, "Failed to parse version tag: {tag}"),
            UpgradeError::AssetNotFound { asset } => write!(f, "Release asset not found: {asset}"),
            UpgradeError::ChecksumParse => write!(f, "Failed to parse checksum file"),
            UpgradeError::ChecksumMismatch { expected, actual } => write!(
                f,
                "Download verification failed (expected {expected}, got {actual})"
            ),
            UpgradeError::PermissionDenied { path } => write!(
                f,
                "Cannot write to installation path: {} (permission denied)",
                path.display()
            ),
            UpgradeError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for UpgradeError {}

impl From<io::Error> for UpgradeError {
    fn from(value: io::Error) -> Self {
        UpgradeError::Io(value)
    }
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub fn run_upgrade() -> Result<UpgradeOutcome, UpgradeError> {
    let current = Version::parse(env!("CARGO_PKG_VERSION")).expect("CARGO_PKG_VERSION is valid");
    let current_exe = std::env::current_exe().map_err(UpgradeError::Io)?;
    let install_dir = current_exe.parent().map(Path::to_path_buf).ok_or_else(|| {
        UpgradeError::Io(io::Error::new(io::ErrorKind::Other, "Invalid exe path"))
    })?;

    let client = github_client()?;

    eprintln!("Checking for updates…");
    let latest_release = get_latest_release(&client)?;
    let latest = parse_release_version(&latest_release.tag_name)?;

    eprintln!("Current version: v{current}");
    eprintln!("Latest version:  v{latest}");

    if latest <= current {
        return Ok(UpgradeOutcome::UpToDate { current });
    }

    ensure_install_dir_writable(&install_dir, &current_exe)?;

    let (target_triple, archive_ext) = current_target_triple_and_ext()?;
    let archive_name = format!("ralph-{target_triple}.{archive_ext}");
    let checksum_name = format!("{archive_name}.sha256");

    let archive_asset = latest_release
        .assets
        .iter()
        .find(|a| a.name == archive_name)
        .ok_or_else(|| UpgradeError::AssetNotFound {
            asset: archive_name.clone(),
        })?;
    let checksum_asset = latest_release
        .assets
        .iter()
        .find(|a| a.name == checksum_name)
        .ok_or_else(|| UpgradeError::AssetNotFound {
            asset: checksum_name.clone(),
        })?;

    eprintln!("Downloading: {archive_name} ({} bytes)", archive_asset.size);

    let tempdir = tempfile::tempdir().map_err(UpgradeError::Io)?;
    let archive_path = tempdir.path().join(&archive_name);
    let checksum_path = tempdir.path().join(&checksum_name);

    download_to_file(
        &client,
        &checksum_asset.browser_download_url,
        &checksum_path,
    )?;
    download_to_file(&client, &archive_asset.browser_download_url, &archive_path)?;

    let expected = read_sha256_from_file(&checksum_path)?;
    let actual = sha256_file_hex(&archive_path)?;
    if !eq_hex_digest(&expected, &actual) {
        return Err(UpgradeError::ChecksumMismatch { expected, actual });
    }

    eprintln!("Verified SHA256 checksum.");

    let extracted_binary_path =
        tempdir
            .path()
            .join(if cfg!(windows) { "ralph.exe" } else { "ralph" });
    extract_binary_from_archive(&archive_path, &archive_ext, &extracted_binary_path)?;
    ensure_executable(&extracted_binary_path)?;

    eprintln!("Replacing current binary: {}", current_exe.display());
    self_replace(&current_exe, &extracted_binary_path, &install_dir)?;

    // Confirm version by spawning the freshly replaced binary.
    let confirmed = Command::new(&current_exe)
        .arg("--version")
        .output()
        .map_err(UpgradeError::Io)
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    if !confirmed.trim().is_empty() {
        eprintln!("Now running: {}", confirmed.trim());
    }

    Ok(UpgradeOutcome::Upgraded {
        from: current,
        to: latest,
    })
}

pub fn permission_denied_suggestions(path: &Path) -> String {
    let mut lines = vec![
        format!(
            "Error: Cannot write to {} (permission denied)",
            path.display()
        ),
        "".to_string(),
        "Solutions:".to_string(),
        "1. Run with elevated permissions: sudo ralph upgrade".to_string(),
        "2. Reinstall to a user-writable location (e.g. ~/.local/bin)".to_string(),
        "3. Download manually from GitHub Releases and replace the binary".to_string(),
    ];
    lines.push("".to_string());
    lines.join("\n")
}

fn github_client() -> Result<Client, UpgradeError> {
    Client::builder()
        .user_agent(format!("ralph/{}", env!("CARGO_PKG_VERSION")))
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| UpgradeError::Network(e.to_string()))
}

fn get_latest_release(client: &Client) -> Result<GithubRelease, UpgradeError> {
    let url = format!("https://api.github.com/repos/{GITHUB_OWNER}/{GITHUB_REPO}/releases/latest");

    let resp = client
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| UpgradeError::Network(e.to_string()))?;

    if resp.status().is_success() {
        return resp
            .json::<GithubRelease>()
            .map_err(|e| UpgradeError::GithubApi(e.to_string()));
    }

    let status = resp.status();
    let remaining = resp
        .headers()
        .get("x-ratelimit-remaining")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_string();
    let body = resp.text().unwrap_or_default();

    if status.as_u16() == 403 && remaining == "0" {
        return Err(UpgradeError::GithubApi(
            "GitHub rate limit exceeded. Please try again in an hour.".to_string(),
        ));
    }

    Err(UpgradeError::GithubApi(format!(
        "Request failed (HTTP {}): {}",
        status.as_u16(),
        body.trim()
    )))
}

fn parse_release_version(tag_name: &str) -> Result<Version, UpgradeError> {
    let trimmed = tag_name
        .trim()
        .strip_prefix("ralph-v")
        .or_else(|| tag_name.trim().strip_prefix('v'))
        .unwrap_or(tag_name.trim());

    Version::parse(trimmed).map_err(|_| UpgradeError::VersionParse {
        tag: tag_name.to_string(),
    })
}

fn current_target_triple_and_ext() -> Result<(String, &'static str), UpgradeError> {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    match (os.as_str(), arch.as_str()) {
        ("macos", "x86_64") => Ok(("x86_64-apple-darwin".to_string(), "tar.gz")),
        ("macos", "aarch64") => Ok(("aarch64-apple-darwin".to_string(), "tar.gz")),
        ("linux", "x86_64") => Ok(("x86_64-unknown-linux-gnu".to_string(), "tar.gz")),
        ("linux", "aarch64") => Ok(("aarch64-unknown-linux-gnu".to_string(), "tar.gz")),
        ("windows", "x86_64") => Ok(("x86_64-pc-windows-msvc".to_string(), "zip")),
        _ => Err(UpgradeError::UnsupportedPlatform { os, arch }),
    }
}

fn ensure_install_dir_writable(install_dir: &Path, target_path: &Path) -> Result<(), UpgradeError> {
    match tempfile::NamedTempFile::new_in(install_dir) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            Err(UpgradeError::PermissionDenied {
                path: target_path.to_path_buf(),
            })
        }
        Err(e) => Err(UpgradeError::Io(e)),
    }
}

fn download_to_file(client: &Client, url: &str, path: &Path) -> Result<(), UpgradeError> {
    let mut resp = client
        .get(url)
        .send()
        .map_err(|e| UpgradeError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(UpgradeError::Network(format!(
            "Download failed (HTTP {}): {url}",
            resp.status().as_u16()
        )));
    }

    let mut out = fs::File::create(path).map_err(UpgradeError::Io)?;
    let total = resp.content_length();
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 64 * 1024];

    loop {
        let n = resp.read(&mut buf).map_err(UpgradeError::Io)?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(UpgradeError::Io)?;
        downloaded += n as u64;
        if let Some(total) = total {
            eprint!("\rDownloaded {downloaded}/{total} bytes…");
        }
    }
    if total.is_some() {
        eprintln!();
    }
    Ok(())
}

fn read_sha256_from_file(path: &Path) -> Result<String, UpgradeError> {
    let content = fs::read_to_string(path).map_err(UpgradeError::Io)?;
    content
        .split_whitespace()
        .next()
        .map(|s| s.to_string())
        .ok_or(UpgradeError::ChecksumParse)
}

fn sha256_file_hex(path: &Path) -> Result<String, UpgradeError> {
    let mut file = fs::File::open(path).map_err(UpgradeError::Io)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).map_err(UpgradeError::Io)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    Ok(format!("{digest:x}"))
}

fn eq_hex_digest(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

fn extract_binary_from_archive(
    archive_path: &Path,
    archive_ext: &str,
    out_path: &Path,
) -> Result<(), UpgradeError> {
    if archive_ext == "tar.gz" {
        let tar_gz = fs::File::open(archive_path).map_err(UpgradeError::Io)?;
        let gz = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries().map_err(UpgradeError::Io)? {
            let mut entry = entry.map_err(UpgradeError::Io)?;
            let path = entry.path().map_err(UpgradeError::Io)?;
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if file_name == "ralph" {
                entry.unpack(out_path).map_err(UpgradeError::Io)?;
                return Ok(());
            }
        }
        return Err(UpgradeError::GithubApi(
            "Downloaded archive did not contain 'ralph' binary".to_string(),
        ));
    }

    if archive_ext == "zip" {
        let file = fs::File::open(archive_path).map_err(UpgradeError::Io)?;
        let mut zip = zip::ZipArchive::new(file)
            .map_err(|e| UpgradeError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
        for i in 0..zip.len() {
            let mut file = zip
                .by_index(i)
                .map_err(|e| UpgradeError::Io(io::Error::new(io::ErrorKind::Other, e)))?;
            let name = file.name().rsplit('/').next().unwrap_or("");
            if name.eq_ignore_ascii_case("ralph.exe") {
                let mut out = fs::File::create(out_path).map_err(UpgradeError::Io)?;
                io::copy(&mut file, &mut out).map_err(UpgradeError::Io)?;
                return Ok(());
            }
        }
        return Err(UpgradeError::GithubApi(
            "Downloaded archive did not contain 'ralph.exe'".to_string(),
        ));
    }

    Err(UpgradeError::GithubApi(format!(
        "Unknown archive extension: {archive_ext}"
    )))
}

fn ensure_executable(path: &Path) -> Result<(), UpgradeError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perm = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perm).map_err(UpgradeError::Io)?;
    }
    Ok(())
}

fn self_replace(
    current_exe: &Path,
    new_exe: &Path,
    install_dir: &Path,
) -> Result<(), UpgradeError> {
    let file_name = current_exe
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("ralph");
    let backup = install_dir.join(format!("{file_name}.old"));

    let _ = fs::remove_file(&backup);

    match fs::rename(current_exe, &backup) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            return Err(UpgradeError::PermissionDenied {
                path: current_exe.to_path_buf(),
            });
        }
        Err(e) => return Err(UpgradeError::Io(e)),
    }

    match fs::rename(new_exe, current_exe) {
        Ok(_) => {}
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            fs::copy(new_exe, current_exe).map_err(UpgradeError::Io)?;
            let _ = fs::remove_file(new_exe);
            ensure_executable(current_exe)?;
        }
        Err(e) => {
            let _ = fs::rename(&backup, current_exe);
            return Err(UpgradeError::Io(e));
        }
    }

    let _ = fs::remove_file(&backup);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_release_version_accepts_v_prefix() {
        let v = parse_release_version("v1.2.3").unwrap();
        assert_eq!(v, Version::parse("1.2.3").unwrap());
    }

    #[test]
    fn parse_release_version_accepts_plain() {
        let v = parse_release_version("0.9.0").unwrap();
        assert_eq!(v, Version::parse("0.9.0").unwrap());
    }

    #[test]
    fn parse_release_version_accepts_ralph_v_prefix() {
        let v = parse_release_version("ralph-v0.2.0").unwrap();
        assert_eq!(v, Version::parse("0.2.0").unwrap());
    }

    #[test]
    fn eq_hex_digest_is_case_insensitive() {
        assert!(eq_hex_digest("ABC", "abc"));
        assert!(eq_hex_digest(" abc ", "ABC"));
    }
}
