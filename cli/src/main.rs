use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "codexbar-sidebarctl", about = "Control the CodexBar Linux sidebar daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon (via systemd --user or direct)
    Start {
        /// Path to config file
        #[arg(short, long, default_value = "~/.config/codexbar-linux-sidebar/config.toml")]
        config: String,

        /// Enable mock mode
        #[arg(long)]
        mock: bool,
    },
    /// Stop the daemon
    Stop,
    /// Show daemon status and state summary
    Status,
    /// Trigger a single poll cycle (for manual refresh)
    Refresh {
        /// Path to config file
        #[arg(short, long, default_value = "~/.config/codexbar-linux-sidebar/config.toml")]
        config: String,

        /// Enable mock mode
        #[arg(long)]
        mock: bool,
    },
    /// Restart the daemon
    Restart {
        /// Path to config file
        #[arg(short, long, default_value = "~/.config/codexbar-linux-sidebar/config.toml")]
        config: String,

        /// Enable mock mode
        #[arg(long)]
        mock: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config, mock } => cmd_start(&config, mock),
        Commands::Stop => cmd_stop(),
        Commands::Status => cmd_status(),
        Commands::Refresh { config, mock } => cmd_refresh(&config, mock),
        Commands::Restart { config, mock } => {
            cmd_stop()?;
            cmd_start(&config, mock)
        }
    }
}

fn daemon_binary() -> PathBuf {
    // Try to find the daemon binary next to the ctl binary, or in PATH
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let side_by_side = parent.join("codexbar-sidebard");
            if side_by_side.exists() {
                return side_by_side;
            }
        }
    }
    // Fall back to PATH
    "codexbar-sidebard".into()
}

fn cmd_start(config: &str, mock: bool) -> anyhow::Result<()> {
    let expanded = shellexpand::tilde(config).to_string();

    // Check if systemd --user is available
    let has_systemd = Command::new("systemctl")
        .args(["--user", "is-system-running"])
        .output()
        .is_ok();

    if has_systemd {
        let service = "codexbar-sidebard.service";
        let status = Command::new("systemctl")
            .args(["--user", "is-active", service])
            .output();

        match status {
            Ok(out) if out.status.success() => {
                println!("Daemon already running (systemd --user). Use 'restart' to reload.");
                return Ok(());
            }
            _ => {
                println!("Starting daemon via systemd --user...");
                let mut cmd = Command::new("systemctl");
                cmd.args(["--user", "start", service]);
                let result = cmd.output()?;
                if result.status.success() {
                    println!("Daemon started.");
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    eprintln!("Failed to start via systemd: {}", stderr);
                    eprintln!("Falling back to direct launch...");
                    return launch_direct(&expanded, mock);
                }
            }
        }
    } else {
        return launch_direct(&expanded, mock);
    }

    Ok(())
}

fn launch_direct(config: &str, mock: bool) -> anyhow::Result<()> {
    let daemon = daemon_binary();
    println!("Launching daemon directly: {}", daemon.display());

    let mut cmd = Command::new(&daemon);
    cmd.arg("--config").arg(config);
    if mock {
        cmd.arg("--mock");
    }

    // Detach from the parent process
    unsafe {
        cmd.pre_exec(|| {
            // Create a new session to detach
            libc::setsid();
            Ok(())
        });
    }

    let child = cmd.spawn()?;
    println!("Daemon started (PID: {})", child.id());
    println!("State file: $XDG_RUNTIME_DIR/codexbar-sidebar/state.json");
    Ok(())
}

fn cmd_stop() -> anyhow::Result<()> {
    // Try systemd first
    let has_systemd = Command::new("systemctl")
        .args(["--user", "is-system-running"])
        .output()
        .is_ok();

    if has_systemd {
        let result = Command::new("systemctl")
            .args(["--user", "stop", "codexbar-sidebard.service"])
            .output()?;

        if result.status.success() {
            println!("Daemon stopped.");
            return Ok(());
        }
    }

    // Fall back to killing the process
    let result = Command::new("pkill")
        .args(["codexbar-sidebard"])
        .output()?;

    if result.status.success() {
        println!("Daemon stopped.");
    } else {
        println!("Daemon not running or already stopped.");
    }

    Ok(())
}

fn cmd_status() -> anyhow::Result<()> {
    let state_path = get_state_path();
    if !state_path.exists() {
        println!("State file not found at {}", state_path.display());
        println!("Is the daemon running? Try 'codexbar-sidebarctl start'");
        return Ok(());
    }

    let content = std::fs::read_to_string(&state_path)?;
    let state: serde_json::Value = serde_json::from_str(&content)?;

    let generated_at = state.get("generated_at")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let available = state.get("codexbar")
        .and_then(|c| c.get("available"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let version = state.get("codexbar")
        .and_then(|c| c.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("N/A");

    println!("CodexBar Linux Sidebar Status");
    println!("=============================");
    println!("Generated at:  {}", generated_at);
    println!("CodexBar CLI:  {} (v{})", if available { "available" } else { "not found" }, version);

    let providers = state.get("providers")
        .and_then(|p| p.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    let errors = state.get("errors")
        .and_then(|e| e.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    println!("Providers:     {}", providers);
    println!("Errors:        {}", errors);

    if let Some(providers_arr) = state.get("providers").and_then(|p| p.as_array()) {
        println!();
        println!("Provider Summary:");
        for p in providers_arr {
            let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("?");
            let status = p.get("status").and_then(|s| s.get("level")).and_then(|v| v.as_str()).unwrap_or("?");
            let platform = p.get("platform_state").and_then(|v| v.as_str()).unwrap_or("?");
            let stale = p.get("stale").and_then(|v| v.as_bool()).unwrap_or(false);

            let usage = p.get("usage").and_then(|u| u.get("primary")).and_then(|w| w.get("display_label")).and_then(|v| v.as_str()).unwrap_or("N/A");
            let stale_mark = if stale { " [STALE]" } else { "" };
            println!("  {:<12} status={:<12} platform={:<12} usage={}{}", id, status, platform, usage, stale_mark);
        }
    }

    if errors > 0 {
        println!();
        println!("Errors:");
        if let Some(errors_arr) = state.get("errors").and_then(|e| e.as_array()) {
            for e in errors_arr {
                let scope = e.get("scope").and_then(|v| v.as_str()).unwrap_or("?");
                let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
                let msg = e.get("message").and_then(|v| v.as_str()).unwrap_or("?");
                println!("  [{}] {}: {}", scope, kind, msg);
            }
        }
    }

    Ok(())
}

fn cmd_refresh(config: &str, mock: bool) -> anyhow::Result<()> {
    let expanded = shellexpand::tilde(config).to_string();
    let daemon = daemon_binary();

    println!("Triggering refresh cycle...");
    let mut cmd = Command::new(&daemon);
    cmd.args(["--config", &expanded, "--once"]);
    if mock {
        cmd.arg("--mock");
    }

    let result = cmd.output()?;
    if result.status.success() {
        println!("Refresh complete.");
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);
        eprintln!("Refresh failed: {} {}", stderr, stdout);
    }

    Ok(())
}

fn get_state_path() -> PathBuf {
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        PathBuf::from(runtime_dir).join("codexbar-sidebar").join("state.json")
    } else {
        let uid = std::env::var("UID").unwrap_or_else(|_| "1000".into());
        PathBuf::from("/run/user").join(uid).join("codexbar-sidebar").join("state.json")
    }
}
