use crate::codexbar::command::{run_codexbar, CommandResult};
use serde_json::Value as JsonValue;
use tracing::{warn, debug};

/// Result of collecting usage data from CodexBar.
#[derive(Debug)]
pub struct UsageCollectionResult {
    /// Parsed JSON value from stdout. May be object or array.
    pub data: Option<JsonValue>,
    /// If aggregate call failed, individual provider results.
    pub per_provider: Vec<IndividualUsageResult>,
    pub command_result: CommandResult,
    pub aggregate_succeeded: bool,
}

#[derive(Debug)]
pub struct IndividualUsageResult {
    pub provider_id: String,
    pub data: Option<JsonValue>,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
}

/// Collect usage data from CodexBar.
///
/// Strategy:
/// 1. Try aggregate call: `codexbar --provider all --format json --json-only --status`
/// 2. If aggregate fails (common on Linux since some providers exit non-zero),
///    fall back to polling each enabled provider individually.
pub async fn collect_usage(
    program: &str,
    show_status: bool,
    timeout_secs: u64,
) -> UsageCollectionResult {
    let mut args = vec![
        "--provider".into(),
        "all".into(),
        "--format".into(),
        "json".into(),
        "--json-only".into(),
    ];

    if show_status {
        args.push("--status".into());
    }

    debug!("Running aggregate usage command");
    let result = run_codexbar(program, &args, timeout_secs).await;

    // If aggregate succeeded, parse it
    if result.exit_code == Some(0) && !result.stdout.trim().is_empty() {
        match serde_json::from_str::<JsonValue>(&result.stdout) {
            Ok(data) => {
                debug!("Aggregate usage collection succeeded");
                return UsageCollectionResult {
                    data: Some(data),
                    per_provider: Vec::new(),
                    command_result: result,
                    aggregate_succeeded: true,
                };
            }
            Err(e) => {
                warn!("Failed to parse aggregate usage JSON: {}", e);
            }
        }
    }

    // Aggregate failed or returned no data. Try individual providers.
    warn!(
        "Aggregate usage command failed (exit: {:?}, stderr: {}). Falling back to per-provider polling.",
        result.exit_code,
        result.stderr.trim()
    );

    // We need to know which providers are enabled. Try codexbar config dump.
    let providers = discover_providers(program, timeout_secs).await;

    let mut per_provider = Vec::new();
    for provider_id in &providers {
        let mut prov_args = vec![
            "--provider".into(),
            provider_id.clone(),
            "--format".into(),
            "json".into(),
            "--json-only".into(),
        ];
        if show_status {
            prov_args.push("--status".into());
        }

        debug!("Polling individual provider: {}", provider_id);
        let prov_result = run_codexbar(program, &prov_args, timeout_secs).await;

        let data = if prov_result.exit_code == Some(0) {
            match serde_json::from_str::<JsonValue>(&prov_result.stdout) {
                Ok(d) => Some(d),
                Err(e) => {
                    warn!("Failed to parse JSON for provider {}: {}", provider_id, e);
                    None
                }
            }
        } else {
            None
        };

        let error = if prov_result.exit_code != Some(0) {
            let stderr = prov_result.stderr.trim().to_string();
            Some(if stderr.is_empty() {
                format!("Exit code: {}", prov_result.exit_code.unwrap_or(-1))
            } else {
                stderr
            })
        } else {
            None
        };

        per_provider.push(IndividualUsageResult {
            provider_id: provider_id.clone(),
            data,
            error,
            exit_code: prov_result.exit_code,
        });
    }

    UsageCollectionResult {
        data: None,
        per_provider,
        command_result: result,
        aggregate_succeeded: false,
    }
}

/// Discover enabled providers from `codexbar config dump`.
async fn discover_providers(program: &str, timeout_secs: u64) -> Vec<String> {
    let args = vec!["config".into(), "dump".into(), "--format".into(), "json".into(), "--json-only".into()];
    let result = run_codexbar(program, &args, timeout_secs).await;

    if result.exit_code != Some(0) {
        warn!("Failed to discover providers from config dump. Using fallback list.");
        return fallback_providers();
    }

    // Try to parse the config dump JSON
    if let Ok(value) = serde_json::from_str::<JsonValue>(&result.stdout) {
        let providers = value.get("providers")
            .and_then(|p| p.as_array())
            .map(|arr| {
                arr.iter()
                    .filter(|p| p.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false))
                    .filter_map(|p| p.get("id").and_then(|id| id.as_str().map(String::from)))
                    .collect::<Vec<String>>()
            });

        if let Some(p) = providers {
            if !p.is_empty() {
                return p;
            }
        }
    }

    warn!("Could not parse config dump, using fallback provider list");
    fallback_providers()
}

/// Fallback list of commonly enabled providers.
fn fallback_providers() -> Vec<String> {
    vec![
        "codex".into(),
        "openai".into(),
        "claude".into(),
        "cursor".into(),
        "gemini".into(),
        "copilot".into(),
        "minimax".into(),
        "openrouter".into(),
    ]
}
