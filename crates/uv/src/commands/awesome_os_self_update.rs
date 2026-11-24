use std::fmt::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use uv_client::BaseClientBuilder;
use uv_command_support::{ExitStatus, Printer};

pub(crate) const DEFAULT_AWESOME_OS_UV_REPO: &str = "awesome-os/uv";

fn platform_asset_name() -> String {
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "macos" => "macos",
        "linux" => "linux",
        other => other,
    };
    let arch = std::env::consts::ARCH;
    let ext = std::env::consts::EXE_SUFFIX;
    format!("uv-{os}-{arch}{ext}")
}

fn build_release_download_url(repo: &str, target_version: Option<&str>) -> String {
    let repo = repo.trim().trim_matches('/');
    let asset = platform_asset_name();
    match target_version {
        Some(ver) if !ver.trim().is_empty() => {
            format!("https://github.com/{repo}/releases/download/{}/{asset}", ver.trim())
        }
        _ => format!("https://github.com/{repo}/releases/latest/download/{asset}"),
    }
}

/// Update `uv` directly from `awesome-os/uv` GitHub Releases and atomically replace
/// the running executable.
pub(crate) async fn awesome_os_self_update(
    target_version: Option<String>,
    token: Option<String>,
    dry_run: bool,
    github_repo: Option<String>,
    printer: Printer,
    client_builder: BaseClientBuilder<'_>,
) -> Result<ExitStatus> {
    let repo = github_repo
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            std::env::var("AWESOME_OS_UV_REPO")
                .ok()
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or_else(|| DEFAULT_AWESOME_OS_UV_REPO.to_owned());

    let url = build_release_download_url(&repo, target_version.as_deref());
    let current_exe = std::env::current_exe().context("Failed to locate current uv executable")?;

    if dry_run {
        writeln!(
            printer.stderr_important(),
            "{}{} [dry-run] Would update uv (v{}) from {} -> {}",
            "info".cyan().bold(),
            ":".bold(),
            env!("CARGO_PKG_VERSION"),
            url.cyan(),
            current_exe.display()
        )?;
        return Ok(ExitStatus::Success);
    }

    if client_builder.is_offline() {
        writeln!(
            printer.stderr_important(),
            "{}{} Self-update is not possible because network connectivity is disabled (`--offline`)",
            "error".red().bold(),
            ":".bold()
        )?;
        return Ok(ExitStatus::Failure);
    }

    writeln!(
        printer.stderr(),
        "{}{} Downloading uv from {}...",
        "info".cyan().bold(),
        ":".bold(),
        url.cyan()
    )?;

    let client = client_builder
        .build()
        .context("Failed to build HTTP client for self-update")?;
    let mut req = client.raw_client().get(&url);
    if let Some(ref tok) = token {
        if !tok.is_empty() {
            req = req.header("Authorization", format!("Bearer {tok}"));
        }
    }

    let response = req
        .send()
        .await
        .with_context(|| format!("Failed to download uv binary from `{url}`"))?
        .error_for_status()
        .with_context(|| format!("GitHub release download failed for `{url}`"))?;

    let bytes = response
        .bytes()
        .await
        .with_context(|| format!("Failed to read response body from `{url}`"))?;

    let parent_dir: PathBuf = current_exe
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);

    let temp_file = tempfile::Builder::new()
        .prefix(".uv-self-update-")
        .suffix(std::env::consts::EXE_SUFFIX)
        .tempfile_in(&parent_dir)
        .or_else(|_| {
            tempfile::Builder::new()
                .prefix(".uv-self-update-")
                .suffix(std::env::consts::EXE_SUFFIX)
                .tempfile()
        })
        .context("Failed to create temporary file for self-update")?;

    let temp_path = temp_file.path().to_path_buf();
    fs_err::tokio::write(&temp_path, &bytes)
        .await
        .with_context(|| format!("Failed to write downloaded uv binary to `{}`", temp_path.display()))?;

    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs_err::tokio::set_permissions(&temp_path, Permissions::from_mode(0o755)).await?;
    }

    self_replace::self_replace(&temp_path)
        .with_context(|| format!("Failed to atomically replace `{}`", current_exe.display()))?;

    writeln!(
        printer.stderr_important(),
        "{}{} Updated uv from {} ({})",
        "success".green().bold(),
        ":".bold(),
        url.cyan(),
        current_exe.display()
    )?;

    Ok(ExitStatus::Success)
}
