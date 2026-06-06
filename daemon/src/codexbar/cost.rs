use crate::codexbar::command::run_codexbar;
use crate::config::CostConfig;
use serde_json::Value as JsonValue;
use tracing::{debug, warn};

/// Result of collecting cost data from CodexBar.
#[derive(Debug)]
#[allow(dead_code)]
pub struct CostCollectionResult {
    /// Parsed JSON. CodexBar cost returns an array of payloads.
    pub data: Option<JsonValue>,
    pub succeeded: bool,
    pub error: Option<String>,
}

/// Collect cost/usage data from `codexbar cost`.
/// This is separate from usage because:
/// - Only Claude + Codex support cost data
/// - Cost collection is slower
/// - Cost failure should not poison usage display
pub async fn collect_cost(
    program: &str,
    cost_config: &CostConfig,
    timeout_secs: u64,
) -> CostCollectionResult {
    if !cost_config.enabled {
        debug!("Cost collector disabled in config");
        return CostCollectionResult {
            data: None,
            succeeded: true,
            error: None,
        };
    }

    let mut args = vec![
        "cost".into(),
        "--format".into(),
        "json".into(),
        "--json-only".into(),
    ];

    if cost_config.days != 30 {
        args.push("--days".into());
        args.push(cost_config.days.to_string());
    }

    debug!("Running cost collector");
    let result = run_codexbar(program, &args, timeout_secs).await;

    if result.exit_code != Some(0) {
        let msg = format!(
            "Cost command failed (exit: {:?}): {}",
            result.exit_code,
            result.stderr.trim()
        );
        warn!("{}", msg);

        if cost_config.failure_mode == "warn" {
            return CostCollectionResult {
                data: None,
                succeeded: true,
                error: Some(msg),
            };
        }

        return CostCollectionResult {
            data: None,
            succeeded: false,
            error: Some(msg),
        };
    }

    match serde_json::from_str::<JsonValue>(&result.stdout) {
        Ok(data) => {
            debug!("Cost collection succeeded");
            CostCollectionResult {
                data: Some(data),
                succeeded: true,
                error: None,
            }
        }
        Err(e) => {
            let msg = format!("Failed to parse cost JSON: {}", e);
            warn!("{}", msg);
            CostCollectionResult {
                data: None,
                succeeded: false,
                error: Some(msg),
            }
        }
    }
}
