use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Result of running a CodexBar CLI command.
#[derive(Debug)]
#[allow(dead_code)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
}

/// Run a codexbar CLI subprocess with a configurable timeout.
/// Returns stdout, stderr, exit code, and whether a timeout occurred.
pub async fn run_codexbar(
    program: &str,
    args: &[String],
    timeout_secs: u64,
) -> CommandResult {
    let timeout_duration = Duration::from_secs(timeout_secs);

    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.kill_on_drop(true);

    let result = timeout(timeout_duration, cmd.output()).await;

    match result {
        Ok(Ok(output)) => CommandResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            timed_out: false,
        },
        Ok(Err(e)) => CommandResult {
            stdout: String::new(),
            stderr: format!("Failed to execute codexbar: {}", e),
            exit_code: None,
            timed_out: false,
        },
        Err(_) => CommandResult {
            stdout: String::new(),
            stderr: "Command timed out".to_string(),
            exit_code: None,
            timed_out: true,
        },
    }
}

/// Check if codexbar CLI is available on PATH.
pub async fn check_codexbar_available(program: &str) -> bool {
    fetch_codexbar_version(program).await.is_some()
}

/// Best-effort CodexBar CLI version string (e.g. "0.32.4").
pub async fn fetch_codexbar_version(program: &str) -> Option<String> {
    for flag in ["-V", "--version"] {
        let result = run_codexbar(program, &[flag.into()], 10).await;
        if result.exit_code != Some(0) {
            continue;
        }
        if let Some(version) = parse_version_output(&result.stdout) {
            return Some(version);
        }
        if let Some(version) = parse_version_output(&result.stderr) {
            return Some(version);
        }
    }

    version_from_install_metadata(program)
}

fn parse_version_output(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("CodexBar ") {
            if !rest.is_empty() && rest != "unknown" {
                return Some(rest.to_string());
            }
        }
        if trimmed.chars().next()?.is_ascii_digit() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Read VERSION file colocated with the installed CLI (release tarball layout).
fn version_from_install_metadata(program: &str) -> Option<String> {
    let path = std::path::Path::new(program);
    if path.is_absolute() {
        if let Some(parent) = path.parent() {
            let version_file = parent.join("VERSION");
            if let Ok(content) = std::fs::read_to_string(version_file) {
                let version = content.trim();
                if !version.is_empty() {
                    return Some(version.to_string());
                }
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let share_version = home.join(".local/share/codexbar-cli/VERSION");
        if let Ok(content) = std::fs::read_to_string(share_version) {
            let version = content.trim();
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }

    None
}
