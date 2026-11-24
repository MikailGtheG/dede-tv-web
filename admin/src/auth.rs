//! Password, setup code, sessions and login throttling.
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::net::IpAddr;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const MIN_PASSWORD_CHARS: usize = 10;
pub const MAX_PASSWORD_BYTES: usize = 256;
pub const SESSION_TTL: Duration = Duration::from_secs(12 * 3600);
pub const SETUP_CODE_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_FAILURES: u32 = 5;
const LOCKOUT: Duration = Duration::from_secs(15 * 60);
const GLOBAL_FAILURES_PER_MINUTE: usize = 20;
const MAX_SESSIONS: usize = 64;

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes).expect("kernel random source");
    bytes
}

pub fn random_token() -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut token = String::with_capacity(64);
    for byte in random_bytes::<32>() {
        token.push(HEX[usize::from(byte >> 4)] as char);
        token.push(HEX[usize::from(byte & 15)] as char);
    }
    token
}

/// Comparison time does not depend on where the inputs differ.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[derive(Debug, PartialEq, Eq)]
pub enum PasswordProblem {
    TooShort,
    TooLong,
    Mismatch,
}

pub fn check_new_password(password: &str, repeat: &str) -> Result<(), PasswordProblem> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(PasswordProblem::TooShort);
    }
    if password.len() > MAX_PASSWORD_BYTES {
        return Err(PasswordProblem::TooLong);
    }
    if password != repeat {
        return Err(PasswordProblem::Mismatch);
    }
    Ok(())
}

/// The Argon2id hash of the administrator password, stored 0600.
pub struct PasswordStore {
    path: PathBuf,
}

impl PasswordStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn is_set(&self) -> bool {
        self.load().is_some()
    }

    fn load(&self) -> Option<String> {
        let meta = fs::symlink_metadata(&self.path).ok()?;
        if !meta.is_file() {
            return None;
        }
        let text = fs::read_to_string(&self.path).ok()?;
        let text = text.trim().to_string();
        PasswordHash::new(&text).ok()?;
        Some(text)
    }

    pub fn set(&self, password: &str) -> std::io::Result<()> {
        let salt = SaltString::encode_b64(&random_bytes::<16>()).expect("16-byte salt");
        let hash = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .expect("argon2 with default parameters")
            .to_string();
        write_private(&self.path, hash.as_bytes())
    }

    pub fn verify(&self, password: &str) -> bool {
        if password.len() > MAX_PASSWORD_BYTES {
            return false;
        }
        let Some(stored) = self.load() else {
            return false;
        };
        let Ok(parsed) = PasswordHash::new(&stored) else {
            return false;
        };
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    }
}

/// Write a file readable only by its owner, replacing it atomically.
pub fn write_private(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let temp = dir.join(format!(
        ".{}.{}",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("tmp"),
        random_token()
    ));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        file.write_all(content)?;
        file.sync_all()?;
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// One-time code shown on the TV while no password is set.
pub struct SetupCode {
    code: String,
    created: Instant,
}

impl SetupCode {
    pub fn generate() -> Self {
        let n = u64::from_le_bytes(random_bytes::<8>()) % 100_000_000;
        Self {
            code: format!("{n:08}"),
            created: Instant::now(),
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn expired(&self, now: Instant) -> bool {
        now.duration_since(self.created) >= SETUP_CODE_TTL
    }

    pub fn matches(&self, entered: &str, now: Instant) -> bool {
        let digits: String = entered.chars().filter(|c| !c.is_whitespace()).collect();
        !self.expired(now) && constant_time_eq(digits.as_bytes(), self.code.as_bytes())
    }

    /// Publish the code for the TV (group-readable, never logged).
    pub fn publish(&self, path: &Path) -> std::io::Result<()> {
        write_private(path, self.code.as_bytes())?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o640))
    }
}

pub struct Session {
    pub csrf: String,
    expires: Instant,
}

#[derive(Default)]
pub struct Sessions {
    map: HashMap<String, Session>,
}

impl Sessions {
    pub fn create(&mut self, now: Instant) -> String {
        self.map.retain(|_, s| s.expires > now);
        if self.map.len() >= MAX_SESSIONS {
            // Drop the session closest to expiry.
            if let Some(oldest) = self
                .map
                .iter()
                .min_by_key(|(_, s)| s.expires)
                .map(|(k, _)| k.clone())
            {
                self.map.remove(&oldest);
            }
        }
        let token = random_token();
        self.map.insert(
            token.clone(),
            Session {
                csrf: random_token(),
                expires: now + SESSION_TTL,
            },
        );
        token
    }

    pub fn get(&self, token: &str, now: Instant) -> Option<&Session> {
        self.map.get(token).filter(|s| s.expires > now)
    }

    pub fn remove(&mut self, token: &str) {
        self.map.remove(token);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}

/// Per-address lockout after repeated failures, plus a global cap.
#[derive(Default)]
pub struct Throttle {
    per_address: HashMap<IpAddr, (u32, Instant)>,
    recent: Vec<Instant>,
}

impl Throttle {
    pub fn allowed(&mut self, address: IpAddr, now: Instant) -> bool {
        self.recent
            .retain(|t| now.duration_since(*t) < Duration::from_secs(60));
        if self.recent.len() >= GLOBAL_FAILURES_PER_MINUTE {
            return false;
        }
        self.per_address
            .retain(|_, (_, since)| now.duration_since(*since) < LOCKOUT);
        self.per_address
            .get(&address)
            .is_none_or(|(count, _)| *count < MAX_FAILURES)
    }

    pub fn failed(&mut self, address: IpAddr, now: Instant) {
        self.recent.push(now);
        let entry = self.per_address.entry(address).or_insert((0, now));
        entry.0 += 1;
        entry.1 = now;
        if self.per_address.len() > 4096 {
            self.per_address
                .retain(|_, (_, since)| now.duration_since(*since) < LOCKOUT);
        }
    }

    pub fn succeeded(&mut self, address: IpAddr) {
        self.per_address.remove(&address);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_rules() {
        assert_eq!(
            check_new_password("kurz", "kurz"),
            Err(PasswordProblem::TooShort)
        );
        assert_eq!(
            check_new_password("langes-passwort", "anderes-passwort"),
            Err(PasswordProblem::Mismatch)
        );
        let long = "x".repeat(MAX_PASSWORD_BYTES + 1);
        assert_eq!(
            check_new_password(&long, &long),
            Err(PasswordProblem::TooLong)
        );
        // Ten characters, not ten bytes: Turkish letters count once.
        assert_eq!(check_new_password("şşşşşşşşşş", "şşşşşşşşşş"), Ok(()));
    }

    #[test]
    fn password_store_roundtrip_and_private_file() {
        let dir = std::env::temp_dir().join(format!("dede-admin-{}", random_token()));
        fs::create_dir(&dir).unwrap();
        let store = PasswordStore::new(dir.join("password"));
        assert!(!store.is_set());
        assert!(!store.verify("irgendwas-langes"));
        store.set("Sehr-geheim-123").unwrap();
        assert!(store.is_set());
        assert!(store.verify("Sehr-geheim-123"));
        assert!(!store.verify("sehr-geheim-123"));
        let mode = fs::metadata(dir.join("password"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        let text = fs::read_to_string(dir.join("password")).unwrap();
        assert!(text.starts_with("$argon2id$") && !text.contains("Sehr-geheim"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn corrupt_password_file_counts_as_unset() {
        let dir = std::env::temp_dir().join(format!("dede-admin-{}", random_token()));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("password"), "kein hash").unwrap();
        let store = PasswordStore::new(dir.join("password"));
        assert!(!store.is_set() && !store.verify("kein hash"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn setup_code_matching_and_expiry() {
        let code = SetupCode::generate();
        let now = Instant::now();
        assert_eq!(code.code().len(), 8);
        assert!(code.code().bytes().all(|b| b.is_ascii_digit()));
        let spaced = format!("{} {}", &code.code()[..4], &code.code()[4..]);
        assert!(code.matches(&spaced, now));
        assert!(!code.matches("", now));
        assert!(!code.matches(&code.code()[..7], now));
        assert!(!code.matches(code.code(), now + SETUP_CODE_TTL));
    }

    #[test]
    fn sessions_expire_and_are_bounded() {
        let mut sessions = Sessions::default();
        let now = Instant::now();
        let token = sessions.create(now);
        assert_eq!(token.len(), 64);
        assert!(sessions.get(&token, now).is_some());
        assert!(sessions.get(&token, now + SESSION_TTL).is_none());
        for _ in 0..MAX_SESSIONS + 10 {
            sessions.create(now);
        }
        assert!(sessions.map.len() <= MAX_SESSIONS);
        sessions.clear();
        assert!(sessions.get(&token, now).is_none());
    }

    #[test]
    fn throttle_locks_out_after_failures() {
        let mut throttle = Throttle::default();
        let a: IpAddr = "192.168.1.20".parse().unwrap();
        let b: IpAddr = "192.168.1.21".parse().unwrap();
        let now = Instant::now();
        for _ in 0..MAX_FAILURES {
            assert!(throttle.allowed(a, now));
            throttle.failed(a, now);
        }
        assert!(!throttle.allowed(a, now));
        assert!(throttle.allowed(b, now));
        assert!(throttle.allowed(a, now + LOCKOUT));
    }

    #[test]
    fn throttle_forgets_old_partial_failures() {
        let mut throttle = Throttle::default();
        let address = IpAddr::from([192, 168, 1, 20]);
        let now = Instant::now();
        for _ in 0..MAX_FAILURES - 1 {
            throttle.failed(address, now);
        }
        let later = now + LOCKOUT;
        assert!(throttle.allowed(address, later));
        throttle.failed(address, later);
        assert!(throttle.allowed(address, later));
    }

    #[test]
    fn throttle_has_a_global_cap() {
        let mut throttle = Throttle::default();
        let now = Instant::now();
        for i in 0..GLOBAL_FAILURES_PER_MINUTE {
            let address = IpAddr::from([10, 0, 0, i as u8]);
            throttle.failed(address, now);
        }
        assert!(!throttle.allowed("10.0.1.1".parse().unwrap(), now));
        assert!(throttle.allowed("10.0.1.1".parse().unwrap(), now + Duration::from_secs(61)));
    }

    #[test]
    fn constant_time_compare() {
        assert!(constant_time_eq(b"12345678", b"12345678"));
        assert!(!constant_time_eq(b"12345678", b"12345679"));
        assert!(!constant_time_eq(b"1234", b"12345678"));
    }
}
