use crate::normalize::platform::{detect_platform_state, is_linux_unsupported_message};
use crate::schema::*;
use chrono::{DateTime, Duration, Utc};
use serde_json::Value as JsonValue;

const DEFAULT_STALE_AFTER_SECS: i64 = 7200;

/// Normalize a single CodexBar provider JSON object into our rich schema.
pub fn normalize_provider(provider_json: &JsonValue) -> Option<ProviderState> {
    let obj = provider_json.as_object()?;

    let id = obj.get("provider")
        .or_else(|| obj.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let name = obj.get("provider")
        .and_then(|v| v.as_str())
        .map(capitalize_provider_name)
        .unwrap_or_else(|| id.clone());

    let usage = normalize_usage(obj.get("usage"));
    let credits = normalize_credits(obj.get("credits"));
    let status = normalize_status(obj.get("status"));
    let auth = normalize_auth(obj.get("usage"), obj.get("error"));

    let error_message = obj.get("error")
        .and_then(|e| e.as_object())
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str());

    let has_usage = usage.as_ref().map(|u| u.primary.is_some()).unwrap_or(false);
    let (platform_state, unsupported_reason) = detect_platform_state(error_message, has_usage);

    let last_updated = usage.as_ref()
        .and_then(|u| u.updated_at)
        .or_else(|| credits.as_ref().and_then(|c| c.updated_at));

    let stale = is_stale(last_updated, error_message, auth.as_ref());

    Some(ProviderState {
        id,
        name,
        enabled: true,
        platform_state,
        unsupported_reason,
        status,
        usage,
        credits,
        cost: None,
        auth,
        stale,
        last_updated,
        raw: ProviderRaw {
            usage: Some(provider_json.clone()),
            cost: None,
        },
    })
}

/// Normalize a provider that failed on Linux (platform_state = Unsupported).
pub fn unsupported_provider(id: &str, reason: &str) -> ProviderState {
    ProviderState {
        id: id.to_string(),
        name: id.to_string(),
        enabled: true,
        platform_state: PlatformState::Unsupported,
        unsupported_reason: Some(reason.to_string()),
        status: Some(ProviderStatus {
            level: "unsupported".into(),
            indicator: None,
            description: Some(reason.to_string()),
            updated_at: None,
        }),
        usage: None,
        credits: None,
        cost: None,
        auth: Some(ProviderAuth {
            source: None,
            state: "unsupported_on_linux".into(),
            message: Some(reason.to_string()),
        }),
        stale: false,
        last_updated: None,
        raw: ProviderRaw {
            usage: None,
            cost: None,
        },
    }
}

/// Normalize a provider given an individual fetch result (with possible error).
pub fn normalized_from_individual(
    id: &str,
    data: Option<&JsonValue>,
    error: Option<&str>,
    _exit_code: Option<i32>,
) -> ProviderState {
    if let Some(json) = data {
        if let Some(provider) = normalize_provider(json) {
            return provider;
        }
    }

    // If we have an error but also some data context, try to extract what we can
    if let Some(json) = data {
        let empty_map = serde_json::Map::new();
        let obj = json.as_object().unwrap_or(&empty_map);
        let err_msg = obj.get("error")
            .and_then(|e| e.as_object())
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or(error.unwrap_or("Unknown error"));

        let is_linux_unsupported = is_linux_unsupported_message(err_msg);

        if is_linux_unsupported {
            return unsupported_provider(id, err_msg);
        }

        return ProviderState {
            id: id.to_string(),
            name: id.to_string(),
            enabled: true,
            platform_state: PlatformState::Unknown,
            unsupported_reason: None,
            status: Some(ProviderStatus {
                level: "error".into(),
                indicator: None,
                description: Some(err_msg.to_string()),
                updated_at: None,
            }),
            usage: None,
            credits: None,
            cost: None,
            auth: Some(ProviderAuth {
                source: None,
                state: "error".into(),
                message: Some(err_msg.to_string()),
            }),
            stale: true,
            last_updated: None,
            raw: ProviderRaw {
                usage: Some(json.clone()),
                cost: None,
            },
        };
    }

    ProviderState {
        id: id.to_string(),
        name: id.to_string(),
        enabled: true,
        platform_state: PlatformState::Unknown,
        unsupported_reason: None,
        status: Some(ProviderStatus {
            level: "error".into(),
            indicator: None,
            description: Some(error.unwrap_or("Unknown error").to_string()),
            updated_at: None,
        }),
        usage: None,
        credits: None,
        cost: None,
        auth: None,
        stale: true,
        last_updated: None,
        raw: ProviderRaw {
            usage: None,
            cost: None,
        },
    }
}

fn capitalize_provider_name(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => s.to_string(),
    }
}

fn is_stale(
    last_updated: Option<DateTime<Utc>>,
    error_message: Option<&str>,
    auth: Option<&ProviderAuth>,
) -> bool {
    if let Some(msg) = error_message {
        let lower = msg.to_lowercase();
        if lower.contains("auth") || lower.contains("cookie") || lower.contains("api key") {
            return true;
        }
    }

    if let Some(auth) = auth {
        if auth.state == "unsupported_on_linux" {
            return false;
        }
        if auth.state == "auth_required" || auth.state == "error" {
            return true;
        }
    }

    if let Some(updated) = last_updated {
        return Utc::now().signed_duration_since(updated) > Duration::seconds(DEFAULT_STALE_AFTER_SECS);
    }

    error_message.is_some()
}

fn normalize_usage(usage_val: Option<&JsonValue>) -> Option<ProviderUsage> {
    let usage_obj = usage_val?.as_object()?;

    let updated_at = usage_obj.get("updatedAt")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let primary = usage_obj.get("primary").and_then(normalize_window);
    let secondary = usage_obj.get("secondary").and_then(normalize_window);
    let tertiary = usage_obj.get("tertiary").and_then(normalize_window);

    Some(ProviderUsage { primary, secondary, tertiary, updated_at })
}

fn normalize_window(window_val: &JsonValue) -> Option<UsageWindow> {
    let obj = window_val.as_object()?;

    let used_percent = obj.get("usedPercent")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let window_minutes = obj.get("windowMinutes").and_then(|v| v.as_u64());

    let reset_at = obj.get("resetsAt")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let reset_label = obj.get("resetDescription")
        .or_else(|| obj.get("resetLabel"))
        .and_then(|v| v.as_str())
        .map(String::from);

    let display_label = format!("{}% left", (100.0 - used_percent).max(0.0).round());

    Some(UsageWindow::new(used_percent, window_minutes, reset_at, reset_label, display_label))
}

fn normalize_credits(credits_val: Option<&JsonValue>) -> Option<ProviderCredits> {
    let obj = credits_val?.as_object()?;

    let remaining = obj.get("remaining").and_then(|v| v.as_f64());
    let updated_at = obj.get("updatedAt")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Some(ProviderCredits {
        remaining,
        currency: None,
        updated_at,
    })
}

fn normalize_status(status_val: Option<&JsonValue>) -> Option<ProviderStatus> {
    let obj = status_val?.as_object()?;

    let indicator = obj.get("indicator").and_then(|v| v.as_str()).map(String::from);

    // Map statuspage indicator to our level
    let level = match indicator.as_deref() {
        Some("none") => "ok".into(),
        Some("minor") => "warn".into(),
        Some("major") => "error".into(),
        Some("critical") => "critical".into(),
        Some("maintenance") => "maintenance".into(),
        Some(other) => other.to_string(),
        None => "unknown".into(),
    };

    let description = obj.get("description").and_then(|v| v.as_str()).map(String::from);
    let updated_at = obj.get("updatedAt")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    Some(ProviderStatus { level, indicator, description, updated_at })
}

fn normalize_auth(usage_val: Option<&JsonValue>, error_val: Option<&JsonValue>) -> Option<ProviderAuth> {
    let error_message = error_val
        .and_then(|e| e.as_object())
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str());

    if let Some(msg) = error_message {
        if is_linux_unsupported_message(msg) {
            return Some(ProviderAuth {
                source: None,
                state: "unsupported_on_linux".into(),
                message: Some(msg.to_string()),
            });
        }

        let lower = msg.to_lowercase();
        let state = if lower.contains("auth") || lower.contains("cookie") || lower.contains("api key") {
            "auth_required"
        } else {
            "error"
        };

        return Some(ProviderAuth {
            source: None,
            state: state.into(),
            message: Some(msg.to_string()),
        });
    }

    let obj = usage_val?.as_object()?;

    let source = obj.get("identity")
        .and_then(|v| v.as_object())
        .and_then(|i| i.get("providerID"))
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("source").and_then(|v| v.as_str()))
        .map(String::from);

    let state = if obj.contains_key("identity") { "authenticated" } else { "unknown" };

    Some(ProviderAuth {
        source,
        state: state.into(),
        message: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn used_percent_maps_to_remaining_display_label() {
        let input = json!({
            "provider": "codex",
            "usage": {
                "primary": { "usedPercent": 28.0, "windowMinutes": 300 },
                "updatedAt": "2026-06-06T10:29:41Z"
            }
        });
        let provider = normalize_provider(&input).unwrap();
        let primary = provider.usage.unwrap().primary.unwrap();
        assert_eq!(primary.used_percent, 28.0);
        assert_eq!(primary.remaining_percent, 72.0);
        assert_eq!(primary.display_label, "72% left");
        assert_eq!(primary.meter_semantics, "used");
    }

    #[test]
    fn macos_web_error_becomes_unsupported_platform() {
        let input = json!({
            "provider": "gemini",
            "error": {
                "message": "Error: selected source requires web support and is only supported on macOS."
            }
        });
        let provider = normalize_provider(&input).unwrap();
        assert!(matches!(provider.platform_state, PlatformState::Unsupported));
        assert!(provider.unsupported_reason.unwrap().contains("web-backed"));
        assert!(provider.raw.usage.is_some());
    }

    #[test]
    fn preserves_raw_usage_payload() {
        let input = json!({ "provider": "codex", "usage": { "primary": { "usedPercent": 10.0 } } });
        let provider = normalize_provider(&input).unwrap();
        assert_eq!(provider.raw.usage.unwrap()["provider"], "codex");
    }
}
