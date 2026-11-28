//! Update state written by dede-update (`/etc/dede-tv/update.conf`).
use std::fs;
use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct State {
    pub state: String,
    pub current: Option<String>,
    pub available: Option<String>,
    pub progress: Option<u8>,
    pub message: Option<String>,
    pub checked: Option<String>,
}

fn safe(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._:+-".contains(&b))
}

pub fn read(path: &Path, version_file: &Path) -> State {
    let mut state = State::default();
    for line in fs::read_to_string(path).unwrap_or_default().lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !safe(value) {
            continue;
        }
        if key == "UPDATE_STATE" {
            state.state = value.to_string();
            continue;
        }
        let value = Some(value.to_string());
        match key {
            "UPDATE_CURRENT" => state.current = value,
            "UPDATE_AVAILABLE" => state.available = value,
            "UPDATE_PROGRESS" => {
                state.progress = value.and_then(|v| v.parse().ok()).filter(|p| *p <= 100)
            }
            "UPDATE_MESSAGE" => state.message = value,
            "UPDATE_CHECKED" => state.checked = value,
            _ => {}
        }
    }
    if state.current.is_none() {
        state.current = fs::read_to_string(version_file)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| safe(v));
    }
    state
}

/// The page refreshes itself while dede-update works.
pub fn busy(state: &State) -> bool {
    matches!(
        state.state.as_str(),
        "checking" | "downloading" | "restarting"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_state_and_falls_back_to_the_image_version() {
        let dir = std::env::temp_dir().join(format!("dede-update-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let (conf, version) = (dir.join("update.conf"), dir.join("version"));
        fs::write(&version, "0.1.0-test.1\n").unwrap();
        let state = read(&conf, &version);
        assert_eq!(state.current.as_deref(), Some("0.1.0-test.1"));
        assert!(!busy(&state));
        fs::write(
            &conf,
            "# x\nUPDATE_STATE=downloading\nUPDATE_CURRENT=0.1.0\nUPDATE_AVAILABLE=0.2.0\n\
             UPDATE_PROGRESS=40\nUPDATE_MESSAGE=<b>\nUPDATE_CHECKED=2026-11-28T10:00\n",
        )
        .unwrap();
        let state = read(&conf, &version);
        assert_eq!(state.current.as_deref(), Some("0.1.0"));
        assert_eq!(state.available.as_deref(), Some("0.2.0"));
        assert_eq!(state.progress, Some(40));
        assert_eq!(state.message, None);
        assert!(busy(&state));
        fs::remove_dir_all(dir).unwrap();
    }
}
