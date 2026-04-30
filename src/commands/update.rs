use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::fs;
use std::os::unix::fs::PermissionsExt;

const REPO: &str = "starc007/relay-cli";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: Option<String>,
    message: Option<String>,
}

pub async fn run(client: &reqwest::Client) -> Result<()> {
    println!("current version: v{}", CURRENT_VERSION);
    print!("checking for updates... ");
    use std::io::Write;
    std::io::stdout().flush().ok();

    let release: Release = client
        .get(format!("https://api.github.com/repos/{}/releases/latest", REPO))
        .header("User-Agent", concat!("relay-cli/", env!("CARGO_PKG_VERSION")))
        .send()
        .await?
        .json()
        .await?;

    if let Some(msg) = &release.message {
        bail!("GitHub API: {}", msg);
    }

    let tag = release.tag_name.context("no release found")?;
    let latest = tag.trim_start_matches('v');

    if latest == CURRENT_VERSION {
        println!("already up to date.");
        return Ok(());
    }

    println!("new version available: v{}", latest);

    let target = detect_target()?;
    let archive_name = format!("relay-v{}-{}.tar.gz", latest, target);
    let url = format!(
        "https://github.com/{}/releases/download/v{}/{}",
        REPO, latest, archive_name
    );

    println!("downloading {}...", archive_name);

    let bytes = client
        .get(&url)
        .send()
        .await
        .context("failed to download release")?
        .error_for_status()
        .context("release not found — may not be published yet")?
        .bytes()
        .await?;

    let tmp_dir = tempfile::tempdir().context("failed to create temp dir")?;
    let archive_path = tmp_dir.path().join(&archive_name);
    fs::write(&archive_path, &bytes)?;

    // Extract binary from tarball
    let archive_file = fs::File::open(&archive_path)?;
    let decoder = flate2::read::GzDecoder::new(archive_file);
    let mut tar = tar::Archive::new(decoder);

    let bin_path = tmp_dir.path().join("relay");
    for entry in tar.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().map(|n| n == "relay").unwrap_or(false) {
            entry.unpack(&bin_path)?;
            break;
        }
    }

    if !bin_path.exists() {
        bail!("relay binary not found in archive");
    }

    fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))?;

    // Replace running binary
    let current_exe = std::env::current_exe().context("failed to get current binary path")?;
    fs::rename(&bin_path, &current_exe)
        .or_else(|_| {
            // rename fails across filesystems — copy instead
            fs::copy(&bin_path, &current_exe).map(|_| ())
        })
        .context("failed to replace binary — try running with sudo")?;

    println!("updated to v{}", latest);
    Ok(())
}

fn detect_target() -> Result<String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let target = match (os, arch) {
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => bail!("unsupported platform: {}-{}", os, arch),
    };

    Ok(target.to_string())
}
