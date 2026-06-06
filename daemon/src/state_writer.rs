use crate::schema::SidebarState;
use fs2::FileExt;
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::debug;

/// Atomically write the sidebar state to the state file.
/// Uses the atomic-write-via-tempfile pattern to prevent QML from reading partial state.
pub fn write_state(state_path: &Path, state: &SidebarState) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(state)?;

    // Write to a temp file first, then rename atomically.
    let dir = state_path.parent().unwrap_or(Path::new("/tmp"));
    let tmp_path = dir.join("state.json.tmp");

    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(json.as_bytes())?;
        file.sync_all()?;
    }

    fs::rename(&tmp_path, state_path)?;

    // Sync the directory to ensure the rename is durable on ext4/XFS.
    if let Ok(dir_file) = fs::File::open(dir) {
        dir_file.sync_all().ok();
    }

    debug!("State written to {}", state_path.display());
    Ok(())
}

/// Ensure the state directory exists, creating it if necessary.
pub fn ensure_state_dir(state_path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Acquire an exclusive lock on the state lock file.
/// Returns the lock file handle. The lock is released when the handle is dropped.
pub fn acquire_lock(lock_path: &Path) -> Result<fs::File, String> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(lock_path)
        .map_err(|e| format!("Failed to open lock file: {}", e))?;
    file.lock_exclusive()
        .map_err(|e| format!("Failed to acquire lock: {}", e))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SidebarState;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn atomic_write_is_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let state = SidebarState::new(true);

        write_state(&state_path, &state).unwrap();

        let content = std::fs::read_to_string(&state_path).unwrap();
        let parsed: SidebarState = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.schema_version, 1);
        assert!(!state_path.with_extension("json.tmp").exists());
    }

    #[test]
    fn lock_serializes_writers() {
        let dir = tempfile::tempdir().unwrap();
        let lock_path = dir.path().join("state.lock");
        let barrier = Arc::new(Barrier::new(2));
        let b1 = barrier.clone();
        let b2 = barrier.clone();
        let lp1 = lock_path.clone();
        let lp2 = lock_path.clone();

        let t1 = thread::spawn(move || {
            let _lock = acquire_lock(&lp1).unwrap();
            b1.wait();
            std::thread::sleep(std::time::Duration::from_millis(50));
        });
        let t2 = thread::spawn(move || {
            b2.wait();
            assert!(try_acquire_lock(&lp2).unwrap().is_none());
        });

        t1.join().unwrap();
        t2.join().unwrap();
    }
}

#[allow(dead_code)]
/// Try to acquire a non-blocking lock. Returns None if another process holds the lock.
pub fn try_acquire_lock(lock_path: &Path) -> Result<Option<fs::File>, String> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(lock_path)
        .map_err(|e| format!("Failed to open lock file: {}", e))?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(file)),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::WouldBlock {
                Ok(None)
            } else {
                Err(format!("Failed to acquire lock: {}", e))
            }
        }
    }
}

#[allow(dead_code)]
/// Write a singleton-initialized state (for --once mode with lock).
pub fn write_state_once(state_path: &Path, state: &SidebarState) -> anyhow::Result<()> {
    let lock_path = state_path.with_file_name("state.lock");
    let _lock = acquire_lock(&lock_path).map_err(|e| anyhow::anyhow!("{}", e))?;
    write_state(state_path, state)
}

#[allow(dead_code)]
/// Read the state file without blocking.
pub fn read_state(state_path: &Path) -> anyhow::Result<SidebarState> {
    let content = fs::read_to_string(state_path)?;
    let state: SidebarState = serde_json::from_str(&content)?;
    Ok(state)
}
