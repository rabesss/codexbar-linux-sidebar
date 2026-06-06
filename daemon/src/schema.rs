use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Top-level state file written by the daemon for the Quickshell UI to consume.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarState {
    pub schema_version: u32,
    pub generated_at: DateTime<Utc>,
    pub codexbar: CodexbarMeta,
    pub providers: Vec<ProviderState>,
    pub errors: Vec<SidebarError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexbarMeta {
    pub available: bool,
    pub version: Option<String>,
    pub usage_command: Vec<String>,
    pub cost_command: Vec<String>,
    pub refresh_interval_seconds: u64,
    pub aggregate_succeeded: bool,
}

/// Normalized state for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderState {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    /// Whether this provider can work on Linux at all.
    /// Many CodexBar providers require macOS-only WebKit/browser access.
    pub platform_state: PlatformState,
    pub unsupported_reason: Option<String>,
    pub status: Option<ProviderStatus>,
    pub usage: Option<ProviderUsage>,
    pub credits: Option<ProviderCredits>,
    pub cost: Option<ProviderCost>,
    pub auth: Option<ProviderAuth>,
    pub stale: bool,
    pub last_updated: Option<DateTime<Utc>>,
    /// Raw verbatim output from CodexBar for this provider.
    /// Stored as untyped JSON to avoid over-typing provider-specific shapes.
    pub raw: ProviderRaw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlatformState {
    Supported,
    Unsupported,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub level: String,
    pub indicator: Option<String>,
    pub description: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderUsage {
    pub primary: Option<UsageWindow>,
    pub secondary: Option<UsageWindow>,
    pub tertiary: Option<UsageWindow>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageWindow {
    /// Percent of quota already consumed (0-100).
    pub used_percent: f64,
    /// Percent of quota remaining (100 - used_percent).
    pub remaining_percent: f64,
    /// Length of the quota window in minutes.
    pub window_minutes: Option<u64>,
    /// ISO 8601 timestamp of when the window resets.
    pub reset_at: Option<DateTime<Utc>>,
    /// Human-readable reset label (e.g. "1h 12m").
    pub reset_label: Option<String>,
    /// Display label preserving CodexBar wording (e.g. "72% left").
    pub display_label: String,
    /// Bar fill semantics: CodexBar `usedPercent` is consumed quota.
    #[serde(default = "default_meter_semantics")]
    pub meter_semantics: String,
}

fn default_meter_semantics() -> String {
    "used".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredits {
    pub remaining: Option<f64>,
    pub currency: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCost {
    pub session_cost_usd: Option<f64>,
    pub last_30_days_cost_usd: Option<f64>,
    pub total_tokens: Option<u64>,
    pub total_cost_usd: Option<f64>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAuth {
    pub source: Option<String>,
    pub state: String,
    pub message: Option<String>,
}

/// Raw verbatim CodexBar output preserved for debugging and future use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRaw {
    pub usage: Option<JsonValue>,
    pub cost: Option<JsonValue>,
}

/// Structured error entry for debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarError {
    pub scope: String,
    pub provider: Option<String>,
    pub level: String,
    pub kind: String,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub message: String,
    pub raw: Option<JsonValue>,
}

impl SidebarState {
    pub fn new(codexbar_available: bool) -> Self {
        Self {
            schema_version: 1,
            generated_at: Utc::now(),
            codexbar: CodexbarMeta {
                available: codexbar_available,
                version: None,
                usage_command: vec![
                    "codexbar".into(),
                    "--provider".into(),
                    "all".into(),
                    "--format".into(),
                    "json".into(),
                    "--json-only".into(),
                    "--status".into(),
                ],
                cost_command: vec![
                    "codexbar".into(),
                    "cost".into(),
                    "--format".into(),
                    "json".into(),
                    "--json-only".into(),
                ],
                refresh_interval_seconds: 60,
                aggregate_succeeded: false,
            },
            providers: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn error(&mut self, scope: &str, provider: Option<&str>, level: &str, kind: &str, command: Vec<String>, exit_code: Option<i32>, message: String, raw: Option<JsonValue>) {
        self.errors.push(SidebarError {
            scope: scope.to_string(),
            provider: provider.map(|s| s.to_string()),
            level: level.to_string(),
            kind: kind.to_string(),
            command,
            exit_code,
            message,
            raw,
        });
    }
}

impl UsageWindow {
    pub fn new(used_percent: f64, window_minutes: Option<u64>, reset_at: Option<DateTime<Utc>>, reset_label: Option<String>, display_label: String) -> Self {
        let remaining_percent = (100.0 - used_percent).max(0.0).min(100.0);
        Self {
            used_percent,
            remaining_percent,
            window_minutes,
            reset_at,
            reset_label,
            display_label,
            meter_semantics: default_meter_semantics(),
        }
    }
}
