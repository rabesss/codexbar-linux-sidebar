use crate::schema::PlatformState;

pub fn detect_platform_state(error_message: Option<&str>, has_usage: bool) -> (PlatformState, Option<String>) {
    let Some(msg) = error_message else {
        return (PlatformState::Supported, None);
    };

    let lower = msg.to_lowercase();

    if lower.contains("only supported on macos")
        || lower.contains("not supported on linux")
        || lower.contains("requires web support")
        || lower.contains("notsupportedonthisplatform")
        || lower.contains("web-backed")
    {
        return (
            PlatformState::Unsupported,
            Some(if msg.contains("macOS") || msg.contains("web") {
                "Provider requires web-backed source not available in Linux CLI".into()
            } else {
                msg.to_string()
            }),
        );
    }

    if has_usage {
        return (PlatformState::Partial, Some(msg.to_string()));
    }

    if lower.contains("not configured") || lower.contains("api key") {
        return (PlatformState::Partial, Some(msg.to_string()));
    }

    (PlatformState::Unknown, Some(msg.to_string()))
}

pub fn is_linux_unsupported_message(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("not supported")
        || lower.contains("notsupportedonthisplatform")
        || lower.contains("only supported on macos")
        || lower.contains("web-backed")
        || lower.contains("requires web support")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_web_error_is_unsupported() {
        let (state, reason) = detect_platform_state(
            Some("Error: selected source requires web support and is only supported on macOS."),
            false,
        );
        assert!(matches!(state, PlatformState::Unsupported));
        assert!(reason.unwrap().contains("web-backed"));
    }

    #[test]
    fn usage_with_error_is_partial() {
        let (state, _) = detect_platform_state(Some("Rate limited"), true);
        assert!(matches!(state, PlatformState::Partial));
    }

    #[test]
    fn no_error_is_supported() {
        let (state, reason) = detect_platform_state(None, false);
        assert!(matches!(state, PlatformState::Supported));
        assert!(reason.is_none());
    }
}
