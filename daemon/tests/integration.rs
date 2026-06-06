//! Shell-level integration checks (optional `codexbar` on PATH).

use std::path::PathBuf;
use std::process::Command;

fn sidebard_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("debug")
        .join("codexbar-sidebard")
}

#[test]
#[ignore = "requires codexbar CLI installed"]
fn integration_once_poll_writes_state() {
    let runtime = tempfile::tempdir().unwrap();
    let state_path = runtime.path().join("codexbar-sidebar/state.json");

    let bin = sidebard_bin();
    assert!(bin.exists(), "build codexbar-sidebard first");

    let status = Command::new(&bin)
        .env("XDG_RUNTIME_DIR", runtime.path())
        .args(["--once"])
        .status()
        .expect("run sidebard");

    assert!(status.success());
    assert!(state_path.exists());

    let content = std::fs::read_to_string(state_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed.get("schema_version").and_then(|v| v.as_u64()), Some(1));
    assert!(parsed
        .get("providers")
        .and_then(|v| v.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false));
}
