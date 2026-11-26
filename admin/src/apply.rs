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

/// Ask for the full list of own channels to be applied. `refresh` makes the
/// TV re-read the folders (films added or deleted) even if the list is the same.
pub fn submit_channels(
    state: &Path,
    channels: &[Channel],
    refresh: bool,
) -> std::io::Result<String> {
    let id = random_token()[..32].to_string();
    let list: Vec<Value> = channels
        .iter()
        .map(|c| json!({"name": c.name, "number": c.number, "folder": c.folder}))
        .collect();
    let request =
        json!({"id": id, "kind": "virtual-channels", "channels": list, "refresh": refresh});
    let dir = state.join("requests");
    fs::create_dir_all(&dir)?;
    // Written under a temporary name first; the path unit only sees *.json.
    write_private(
        &dir.join(format!("{id}.json")),
        request.to_string().as_bytes(),
    )?;
    Ok(id)
}

pub fn submit_language(state: &Path, lang: crate::i18n::Lang) -> std::io::Result<String> {
    let id = random_token()[..32].to_string();
    let request = json!({"id": id, "kind": "language", "language": lang.code()});
    let dir = state.join("requests");
    fs::create_dir_all(&dir)?;
    write_private(
        &dir.join(format!("{id}.json")),
        request.to_string().as_bytes(),
    )?;
    Ok(id)
}

/// Addresses as typed; the root helper validates them (never logged here).
pub fn submit_iptv(state: &Path, m3u: &str, epg: &str) -> std::io::Result<String> {
    let id = random_token()[..32].to_string();
    let request = json!({"id": id, "kind": "iptv", "m3u_url": m3u, "epg_url": epg});
    let dir = state.join("requests");
    fs::create_dir_all(&dir)?;
    write_private(
        &dir.join(format!("{id}.json")),
        request.to_string().as_bytes(),
    )?;
    Ok(id)
}

/// Ask for a new WireGuard device; the answer carries its configuration.
pub fn submit_wireguard_add(state: &Path, name: &str, endpoint: &str) -> std::io::Result<String> {
    queue(
        state,
        json!({"kind": "wireguard-add", "name": name, "endpoint": endpoint}),
    )
}

pub fn submit_wireguard_remove(state: &Path, address: &str) -> std::io::Result<String> {
    queue(
        state,
        json!({"kind": "wireguard-remove", "address": address}),
    )
}

/// `tailscale-login`, `tailscale-status` or `tailscale-logout`.
pub fn submit_tailscale(state: &Path, kind: &'static str) -> std::io::Result<String> {
    queue(state, json!({ "kind": kind }))
}

fn queue(state: &Path, mut request: Value) -> std::io::Result<String> {
    let id = random_token()[..32].to_string();
    request["id"] = json!(id);
    let dir = state.join("requests");
    fs::create_dir_all(&dir)?;
    write_private(
        &dir.join(format!("{id}.json")),
        request.to_string().as_bytes(),
    )?;
    Ok(id)
}

fn answer_field(state: &Path, id: &str, name: &str) -> Option<String> {
    if !valid_id(id) {
        return None;
    }
    let text = fs::read_to_string(state.join("results").join(format!("{id}.json"))).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    value[name].as_str().map(String::from)
}

/// Configuration of a new WireGuard device, kept until it is dismissed.
pub fn peer_config(state: &Path, id: &str) -> Option<String> {
    answer_field(state, id, "peer").filter(|p| p.starts_with("[Interface]") && p.len() <= 2048)
}

/// Tailscale sign-in link for the owner's phone.
pub fn login_link(state: &Path, id: &str) -> Option<String> {
    answer_field(state, id, "login").filter(|l| {
        l.starts_with("https://")
            && l.len() <= 512
            && !l.chars().any(|c| c.is_whitespace() || c.is_control())
    })
}

/// Drop the device's private key from the answer once it was taken over.
pub fn forget_peer(state: &Path, id: &str) -> std::io::Result<()> {
    if !valid_id(id) {
        return Ok(());
    }
    let path = state.join("results").join(format!("{id}.json"));
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(());
    };
    let Ok(mut value) = serde_json::from_str::<Value>(&text) else {
        return Ok(());
    };
    if let Some(map) = value.as_object_mut() {
        if map.remove("peer").is_some() {
            write_private(&path, value.to_string().as_bytes())?;
        }
    }
    Ok(())
}

/// Same basic rule as the device: http(s), no blanks or control characters.
pub fn plausible_url(url: &str) -> bool {
    (url.starts_with("http://") || url.starts_with("https://"))
        && url.len() > 8
        && url.len() <= 2048
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
}

/// Host part of the configured M3U address, for display only.
pub fn iptv_host(config: &Path) -> Option<String> {
    let text = fs::read_to_string(config).ok()?;
    let url = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("M3U_URL="))?;
    let rest = url.split_once("://")?.1;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    (!host.is_empty()).then(|| host.chars().take(80).collect())
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
    fn iptv_helpers() {
        assert!(plausible_url("https://tv.example/get.php?u=1&t=2"));
        for bad in [
            "",
            "ftp://x",
            "https://a b",
            "http://",
            "javascript:alert(1)",
        ] {
            assert!(!plausible_url(bad), "{bad}");
        }
        let dir = std::env::temp_dir().join(format!("dede-iptv-{}", random_token()));
        fs::create_dir_all(&dir).unwrap();
        let conf = dir.join("iptv.conf");
        assert_eq!(iptv_host(&conf), None);
        fs::write(
            &conf,
            "M3U_URL=https://user:secret@tv.example:8443/list.m3u?token=abc\n",
        )
        .unwrap();
        assert_eq!(iptv_host(&conf).as_deref(), Some("tv.example:8443"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn request_and_outcome() {
        let dir = std::env::temp_dir().join(format!("dede-apply-{}", random_token()));
        let channels = vec![Channel {
            name: "Dede \"Film\"".into(),
            number: 5,
            folder: "film".into(),
        }];
        let id = submit_channels(&dir, &channels, true).unwrap();
        assert!(valid_id(&id));
        let request: Value = serde_json::from_str(
            &fs::read_to_string(dir.join("requests").join(format!("{id}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(request["kind"], "virtual-channels");
        assert_eq!(request["refresh"], true);
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
