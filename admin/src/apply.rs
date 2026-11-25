//! Change requests for the root apply helper (dede-admin-apply, dede-tv-os).
//!
//! The service has no root rights: it drops a JSON request into
//! `<state>/requests/`, a systemd path unit runs the helper, which validates
//! again, applies and answers in `<state>/results/<id>.json`.
use crate::auth::{random_token, write_private};
use crate::channels::Channel;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Done,
    Failed(String),
}

pub fn valid_id(id: &str) -> bool {
    id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Ask for the full list of own channels to be applied; `refresh` also
/// re-reads folders whose films changed.
pub fn submit_channels(state: &Path, channels: &[Channel]) -> std::io::Result<String> {
    let id = random_token()[..32].to_string();
    let list: Vec<Value> = channels
        .iter()
        .map(|c| json!({"name": c.name, "number": c.number, "folder": c.folder}))
        .collect();
    let request = json!({"id": id, "kind": "virtual-channels", "channels": list});
    let dir = state.join("requests");
    fs::create_dir_all(&dir)?;
    // Written under a temporary name first; the path unit only sees *.json.
    write_private(
        &dir.join(format!("{id}.json")),
        request.to_string().as_bytes(),
    )?;
    Ok(id)
}

pub fn outcome(state: &Path, id: &str) -> Outcome {
    if !valid_id(id) {
        return Outcome::Failed("unknown".into());
    }
    let Ok(text) = fs::read_to_string(state.join("results").join(format!("{id}.json"))) else {
        return Outcome::Pending;
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(v) if v["ok"] == json!(true) => Outcome::Done,
        Ok(v) => Outcome::Failed(
            v["error"]
                .as_str()
                .unwrap_or("unknown")
                .chars()
                .take(64)
                .collect(),
        ),
        Err(_) => Outcome::Pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_and_outcome() {
        let dir = std::env::temp_dir().join(format!("dede-apply-{}", random_token()));
        let channels = vec![Channel {
            name: "Dede \"Film\"".into(),
            number: 5,
            folder: "film".into(),
        }];
        let id = submit_channels(&dir, &channels).unwrap();
        assert!(valid_id(&id));
        let request: Value = serde_json::from_str(
            &fs::read_to_string(dir.join("requests").join(format!("{id}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(request["kind"], "virtual-channels");
        assert_eq!(request["channels"][0]["name"], "Dede \"Film\"");
        assert_eq!(outcome(&dir, &id), Outcome::Pending);
        fs::create_dir_all(dir.join("results")).unwrap();
        fs::write(
            dir.join("results").join(format!("{id}.json")),
            r#"{"ok": false, "error": "number-taken"}"#,
        )
        .unwrap();
        assert_eq!(outcome(&dir, &id), Outcome::Failed("number-taken".into()));
        fs::write(
            dir.join("results").join(format!("{id}.json")),
            r#"{"ok": true}"#,
        )
        .unwrap();
        assert_eq!(outcome(&dir, &id), Outcome::Done);
        assert_eq!(outcome(&dir, "../x"), Outcome::Failed("unknown".into()));
        fs::remove_dir_all(dir).unwrap();
    }
}
