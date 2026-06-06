use codexbar_sidebard::codexbar;
use codexbar_sidebard::config::{DaemonConfig, MockConfig};
use codexbar_sidebard::normalize;
use codexbar_sidebard::schema::*;
use codexbar_sidebard::state_writer;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde_json::Value as JsonValue;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "codexbar-sidebard", about = "Daemon that polls CodexBar CLI and writes normalized state for the Quickshell sidebar")]
struct Cli {
    /// Path to TOML config file
    #[arg(short, long, default_value = "~/.config/codexbar-linux-sidebar/config.toml")]
    config: String,

    /// Run once (single poll cycle), then exit
    #[arg(long)]
    once: bool,

    /// Enable mock mode (generates fake data)
    #[arg(long)]
    mock: bool,

    /// How often to poll (seconds), overrides config
    #[arg(long)]
    poll_interval: Option<u64>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("codexbar_sidebard={}", log_level))
        .init();

    // Load config
    let config_path = shellexpand::tilde(&cli.config).to_string();
    let config = match DaemonConfig::load(std::path::Path::new(&config_path)) {
        Ok(c) => c,
        Err(e) => {
            info!("No config found at {} ({}), using defaults", config_path, e);
            DaemonConfig::default()
        }
    };

    // Apply CLI overrides
    let config = if cli.mock {
        DaemonConfig {
            mock: MockConfig { enabled: true, num_providers: 6 },
            ..config
        }
    } else {
        config
    };

    let poll_interval = cli.poll_interval.unwrap_or(config.codexbar.poll_seconds);
    let state_path = DaemonConfig::state_path();

    // Ensure state directory exists
    state_writer::ensure_state_dir(&state_path)?;
    info!("State will be written to {}", state_path.display());

    if cli.once {
        run_poll_cycle(&config).await?;
        return Ok(());
    }

    // Main poll loop
    info!(
        "Starting poll loop (interval: {}s, mock: {})",
        poll_interval, config.mock.enabled
    );

    loop {
        if let Err(e) = run_poll_cycle(&config).await {
            error!("Poll cycle failed: {}", e);

            let mut state = SidebarState::new(false);
            state.error(
                "poll_loop",
                None,
                "error",
                "poll_failed",
                vec![],
                None,
                format!("Poll cycle failed: {}", e),
                None,
            );
            let _ = state_writer::write_state(&state_path, &state);
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received shutdown signal, exiting");
                break;
            }
            _ = sleep(Duration::from_secs(poll_interval)) => {}
        }
    }

    Ok(())
}

async fn run_poll_cycle(config: &DaemonConfig) -> anyhow::Result<()> {
    let state_path = DaemonConfig::state_path();
    let _state_dir = DaemonConfig::state_dir();

    // Create lock
    let lock_path = DaemonConfig::lock_path();
    let _lock = state_writer::acquire_lock(&lock_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    if config.mock.enabled {
        write_mock_state(&state_path, config.mock.num_providers)?;
        info!("Wrote mock state with {} providers", config.mock.num_providers);
        return Ok(());
    }

    // Real mode: poll CodexBar
    let program = &config.codexbar.command;
    let timeout = config.codexbar.poll_seconds.max(30);

    // Check if codexbar is available
    let version = codexbar::command::fetch_codexbar_version(program).await;
    let available = version.is_some();
    if !available {
        let mut state = SidebarState::new(false);
        state.error(
            "startup",
            None,
            "error",
            "codexbar_not_found",
            vec![program.clone()],
            None,
            format!("CodexBar CLI '{}' not found on PATH or not executable", program),
            None,
        );
        state_writer::write_state(&state_path, &state)?;
        warn!("CodexBar CLI '{}' not available. UI will show 'not installed' state.", program);
        return Ok(());
    }

    // Collect usage data
    let usage_result = codexbar::usage::collect_usage(
        program,
        config.display.show_provider_status,
        timeout,
    )
    .await;

    // Collect cost data (optional enrichment)
    let cost_result = codexbar::cost::collect_cost(
        program,
        &config.codexbar.cost,
        timeout,
    )
    .await;

    // Build state
    let mut state = SidebarState::new(true);
    state.codexbar.version = version;
    state.codexbar.aggregate_succeeded = usage_result.aggregate_succeeded;
    state.codexbar.refresh_interval_seconds = config.codexbar.poll_seconds;

    // Normalize providers
    if let Some(data) = &usage_result.data {
        // Aggregate succeeded: extract providers from the JSON
        let providers = extract_providers_from_aggregate(data);
        state.providers = providers.into_iter()
            .filter_map(|p| normalize::provider::normalize_provider(&p))
            .filter(|p| should_include_provider(p, config))
            .collect();

        // Merge cost data if available
        if let Some(cost_data) = &cost_result.data {
            merge_cost_into_providers(&mut state.providers, cost_data);
        }
    } else {
        // Aggregate failed: use per-provider results
        for individual in &usage_result.per_provider {
            let provider = normalize::provider::normalized_from_individual(
                &individual.provider_id,
                individual.data.as_ref(),
                individual.error.as_deref(),
                individual.exit_code,
            );
            state.providers.push(provider);
        }

        state.providers.retain(|p| should_include_provider(p, config));

        // Merge cost data (by matching provider IDs)
        if let Some(cost_data) = &cost_result.data {
            merge_cost_into_providers(&mut state.providers, cost_data);
        }
    }

    // Add cost-specific errors
    if let Some(err) = &cost_result.error {
        state.error(
            "cost",
            None,
            "warn",
            "cost_collection_warning",
            state.codexbar.cost_command.clone(),
            None,
            err.clone(),
            None,
        );
    }

    // Add aggregate failure as a warning
    if !usage_result.aggregate_succeeded {
        state.error(
            "usage",
            None,
            "warn",
            "aggregate_failed_fell_back_to_per_provider",
            state.codexbar.usage_command.clone(),
            usage_result.command_result.exit_code,
            format!(
                "Aggregate poll failed; fell back to per-provider polling. stderr: {}",
                usage_result.command_result.stderr.trim()
            ),
            usage_result.command_result.stdout.is_empty().then(|| JsonValue::String("empty stdout".into())),
        );
    }

    state.generated_at = Utc::now();
    state_writer::write_state(&state_path, &state)?;
    info!("State written with {} providers", state.providers.len());

    Ok(())
}

fn should_include_provider(provider: &ProviderState, config: &DaemonConfig) -> bool {
    if !provider.enabled && !config.codexbar.show_disabled_providers {
        return false;
    }
    if provider.stale && !config.codexbar.show_stale_data {
        return false;
    }
    if provider.platform_state == PlatformState::Unknown
        && provider.status.as_ref().map(|s| s.level.as_str()) == Some("error")
        && !config.codexbar.show_errors
    {
        return false;
    }
    true
}

/// Extract individual provider objects from an aggregate CodexBar response.
/// The response can be:
/// - An array of provider payloads
/// - A single provider object
/// - An object with a "providers" key
fn extract_providers_from_aggregate(data: &JsonValue) -> Vec<JsonValue> {
    match data {
        JsonValue::Array(arr) => arr.clone(),
        JsonValue::Object(obj) => {
            // Check for providers array
            if let Some(providers) = obj.get("providers").and_then(|p| p.as_array()) {
                return providers.clone();
            }
            // Single provider object
            if obj.contains_key("provider") || obj.contains_key("usage") {
                return vec![data.clone()];
            }
            // Unknown shape, return as-is
            vec![data.clone()]
        }
        _ => vec![data.clone()],
    }
}

/// Merge cost data into provider states by matching provider IDs.
fn merge_cost_into_providers(providers: &mut Vec<ProviderState>, cost_data: &JsonValue) {
    let cost_array = match cost_data {
        JsonValue::Array(arr) => arr.clone(),
        JsonValue::Object(obj) if obj.contains_key("provider") => vec![cost_data.clone()],
        _ => return,
    };

    for cost_entry in &cost_array {
        let obj = match cost_entry.as_object() {
            Some(o) => o,
            None => continue,
        };

        let cost_provider_id = obj.get("provider").and_then(|v| v.as_str());
        let Some(cost_id) = cost_provider_id else { continue };

        // Find matching provider
        if let Some(provider) = providers.iter_mut().find(|p| p.id == cost_id) {
            let session_cost = obj.get("sessionCostUSD").and_then(|v| v.as_f64());
            let last_30_days_cost = obj.get("last30DaysCostUSD").and_then(|v| v.as_f64());

            let total_cost = obj.get("totals")
                .and_then(|t| t.as_object())
                .and_then(|t| t.get("totalCost"))
                .and_then(|v| v.as_f64());

            let total_tokens = obj.get("totals")
                .and_then(|t| t.as_object())
                .and_then(|t| t.get("totalTokens"))
                .and_then(|v| v.as_u64());

            let updated_at = obj.get("updatedAt")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            provider.cost = Some(ProviderCost {
                session_cost_usd: session_cost,
                last_30_days_cost_usd: last_30_days_cost,
                total_tokens,
                total_cost_usd: total_cost,
                updated_at,
            });

            // Store raw cost
            provider.raw.cost = Some(cost_entry.clone());
        }
    }
}

/// Generate a mock state with fake providers for UI development.
fn write_mock_state(state_path: &std::path::Path, num_providers: usize) -> anyhow::Result<()> {
    let now = Utc::now();
    let mut state = SidebarState::new(true);
    state.codexbar.version = Some("0.32.4".into());
    state.codexbar.aggregate_succeeded = true;

    let mock_providers = vec![
        ("codex", "Codex", 28.0, 72.0, 300u64, "1h 12m", "ok", "authenticated", "openai-web"),
        ("claude", "Claude", 54.0, 46.0, 10080, "Fri at 9:00 AM", "ok", "authenticated", "oauth"),
        ("cursor", "Cursor", 0.0, 0.0, 0, "N/A", "warn", "auth_required", "cli"),
        ("openrouter", "OpenRouter", 0.0, 0.0, 0, "N/A", "ok", "authenticated", "api"),
        ("gemini", "Gemini", 0.0, 0.0, 0, "N/A", "unsupported", "unsupported_on_linux", "unknown"),
        ("minimax", "MiniMax", 22.0, 78.0, 43200, "monthly window", "ok", "authenticated", "api"),
        ("copilot", "Copilot", 65.0, 35.0, 10080, "Tue at 12:00 AM", "ok", "authenticated", "oauth"),
        ("grok", "Grok", 10.0, 90.0, 300, "in 4h 30m", "ok", "authenticated", "cli"),
    ];

    for i in 0..num_providers.min(mock_providers.len()) {
        let (id, name, used, _remaining, window, reset, status_level, auth_state, source) = mock_providers[i];

        let reset_time = if window > 0 {
            Some(now + chrono::Duration::seconds((window * 60) as i64 / 2))
        } else {
            None
        };

        let reset_label = if reset == "N/A" { None } else { Some(reset.to_string()) };

        let display_label = if used > 0.0 {
            format!("{}% left", (100.0_f64 - used).round() as u64)
        } else if status_level == "unsupported" {
            "N/A".into()
        } else {
            "No fresh data".into()
        };

        let is_stale = id == "cursor";
        let is_unsupported = status_level == "unsupported";

        let provider = ProviderState {
            id: id.to_string(),
            name: name.to_string(),
            enabled: true,
            platform_state: if is_unsupported { PlatformState::Unsupported } else { PlatformState::Supported },
            unsupported_reason: if is_unsupported { Some("Provider requires web-backed source not available in Linux CLI".into()) } else { None },
            status: Some(ProviderStatus {
                level: status_level.to_string(),
                indicator: Some(if status_level == "ok" { "none".into() } else { status_level.into() }),
                description: Some(match status_level {
                    "ok" => "Operational".into(),
                    "warn" => "Auth required".into(),
                    "unsupported" => "Not available on Linux".into(),
                    _ => "Unknown".into(),
                }),
                updated_at: Some(now),
            }),
            usage: if !is_unsupported && !is_stale {
                Some(ProviderUsage {
                    primary: Some(UsageWindow::new(
                        used,
                        Some(window),
                        reset_time,
                        reset_label.clone(),
                        display_label.clone(),
                    )),
                    secondary: if window == 10080 {
                        Some(UsageWindow::new(
                            30.0,
                            Some(300),
                            Some(now + chrono::Duration::hours(3)),
                            Some("3h".into()),
                            "70% left".into(),
                        ))
                    } else { None },
                    tertiary: None,
                    updated_at: Some(now),
                })
            } else if is_stale {
                Some(ProviderUsage {
                    primary: None,
                    secondary: None,
                    tertiary: None,
                    updated_at: Some(now - chrono::Duration::hours(2)),
                })
            } else {
                None
            },
            credits: if !is_unsupported && !is_stale {
                Some(ProviderCredits {
                    remaining: if id == "openrouter" { Some(3.41) } else { Some(112.4) },
                    currency: if id == "openrouter" { Some("USD".into()) } else { None },
                    updated_at: Some(now),
                })
            } else { None },
            cost: if id == "codex" || id == "claude" {
                Some(ProviderCost {
                    session_cost_usd: Some(if id == "codex" { 1.23 } else { 4.56 }),
                    last_30_days_cost_usd: Some(if id == "codex" { 45.67 } else { 89.01 }),
                    total_tokens: Some(if id == "codex" { 123456 } else { 789012 }),
                    total_cost_usd: Some(if id == "codex" { 45.67 } else { 89.01 }),
                    updated_at: Some(now),
                })
            } else { None },
            auth: Some(ProviderAuth {
                source: Some(source.to_string()),
                state: auth_state.to_string(),
                message: if is_stale { Some("Cookie/auth needed. Run codexbar config set-api-key or enable browser cookie import.".into()) } else { None },
            }),
            stale: is_stale,
            last_updated: Some(now),
            raw: ProviderRaw {
                usage: None,
                cost: None,
            },
        };

        state.providers.push(provider);
    }

    state.generated_at = now;

    // Add a sample error for mock testing
    state.error(
        "usage",
        Some("cursor"),
        "warn",
        "auth_required",
        vec!["codexbar".into(), "--provider".into(), "cursor".into(), "--format".into(), "json".into()],
        Some(2),
        "Cookie/auth needed for Cursor. See codexbar docs.".into(),
        None,
    );

    state_writer::write_state(state_path, &state)?;
    Ok(())
}
