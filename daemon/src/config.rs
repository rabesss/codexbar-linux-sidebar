use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TOML configuration for the CodexBar sidebar daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub codexbar: CodexbarConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub mock: MockConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Sidebar position on screen.
    #[serde(default = "default_position")]
    pub position: String,
    /// Sidebar width in pixels.
    #[serde(default = "default_width")]
    pub width: u32,
    /// How often the QML UI polls the state file (ms).
    #[serde(default = "default_poll_ms")]
    pub poll_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexbarConfig {
    /// Path to the codexbar CLI binary.
    #[serde(default = "default_codexbar_path")]
    pub command: String,
    /// Path to ~/.codexbar/config.json (for reading provider list).
    #[serde(default = "default_config_path")]
    pub config_path: String,
    /// How often to poll CodexBar (seconds).
    #[serde(default = "default_poll_seconds")]
    pub poll_seconds: u64,
    /// Whether to show disabled providers.
    #[serde(default = "default_true")]
    pub show_disabled_providers: bool,
    /// Whether to show providers with stale data.
    #[serde(default = "default_true")]
    pub show_stale_data: bool,
    /// Whether to show error-state providers.
    #[serde(default = "default_true")]
    pub show_errors: bool,
    /// Cost collector settings.
    #[serde(default)]
    pub cost: CostConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostConfig {
    /// Whether to run the cost collector.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Number of days of cost history.
    #[serde(default = "default_cost_days")]
    pub days: u32,
    /// What to do if cost collection fails.
    #[serde(default = "default_cost_failure_mode")]
    pub failure_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default = "default_true")]
    pub show_usage_bars: bool,
    #[serde(default = "default_true")]
    pub show_reset_countdowns: bool,
    #[serde(default = "default_true")]
    pub show_spend: bool,
    #[serde(default = "default_true")]
    pub show_credit_balance: bool,
    #[serde(default = "default_true")]
    pub show_provider_status: bool,
    #[serde(default = "default_true")]
    pub show_last_updated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockConfig {
    /// Enable mock mode (generates fake data instead of calling CodexBar).
    #[serde(default)]
    pub enabled: bool,
    /// Number of mock providers to generate.
    #[serde(default = "default_mock_providers")]
    pub num_providers: usize,
}

fn default_position() -> String { "left".into() }
fn default_width() -> u32 { 430 }
fn default_poll_ms() -> u64 { 2000 }
fn default_codexbar_path() -> String { "codexbar".into() }
fn default_config_path() -> String { "~/.codexbar/config.json".into() }
fn default_poll_seconds() -> u64 { 60 }
fn default_true() -> bool { true }
fn default_cost_days() -> u32 { 30 }
fn default_cost_failure_mode() -> String { "warn".into() }
fn default_mock_providers() -> usize { 6 }

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            codexbar: CodexbarConfig::default(),
            display: DisplayConfig::default(),
            mock: MockConfig::default(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            position: default_position(),
            width: default_width(),
            poll_ms: default_poll_ms(),
        }
    }
}

impl Default for CodexbarConfig {
    fn default() -> Self {
        Self {
            command: default_codexbar_path(),
            config_path: default_config_path(),
            poll_seconds: default_poll_seconds(),
            show_disabled_providers: default_true(),
            show_stale_data: default_true(),
            show_errors: default_true(),
            cost: CostConfig::default(),
        }
    }
}

impl Default for CostConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            days: default_cost_days(),
            failure_mode: default_cost_failure_mode(),
        }
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_usage_bars: default_true(),
            show_reset_countdowns: default_true(),
            show_spend: default_true(),
            show_credit_balance: default_true(),
            show_provider_status: default_true(),
            show_last_updated: default_true(),
        }
    }
}

impl Default for MockConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            num_providers: default_mock_providers(),
        }
    }
}

impl DaemonConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: DaemonConfig = toml::from_str(&content)?;
        Ok(config)
    }
    pub fn state_dir() -> PathBuf {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("codexbar-sidebar")
        } else {
            PathBuf::from("/run/user").join(std::env::var("UID").unwrap_or_else(|_| "1000".into()))
                .join("codexbar-sidebar")
        }
    }

    pub fn state_path() -> PathBuf {
        Self::state_dir().join("state.json")
    }

    pub fn lock_path() -> PathBuf {
        Self::state_dir().join("state.lock")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_example_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, include_str!("../../examples/config.example.toml")).unwrap();
        let config = DaemonConfig::load(&path).unwrap();
        assert_eq!(config.ui.width, 430);
        assert_eq!(config.codexbar.poll_seconds, 60);
        assert!(config.codexbar.cost.enabled);
    }

    #[test]
    fn defaults_when_fields_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[mock]\nenabled = true\n").unwrap();
        let config = DaemonConfig::load(&path).unwrap();
        assert!(config.mock.enabled);
        assert_eq!(config.codexbar.command, "codexbar");
    }
}
