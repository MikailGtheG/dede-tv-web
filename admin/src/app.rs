//! Request handling, independent of the HTTP server library.
use crate::auth::{
    check_new_password, constant_time_eq, random_token, PasswordProblem, PasswordStore, Sessions,
    SetupCode, Throttle,
};
use crate::html::{self, Notice};
use crate::i18n::{Lang, Text};
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::time::Instant;

pub const MAX_FORM_BYTES: usize = 8 * 1024;
const SESSION_COOKIE: &str = "dede_session";
const FORM_COOKIE: &str = "dede_form";
const STYLE: &str = include_str!("../static/style.css");

pub struct Config {
    pub state_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub ui_config: PathBuf,
}

pub struct Request {
    pub method: String,
    pub path: String,
    pub host: Option<String>,
    pub origin: Option<String>,
    pub cookie: Option<String>,
    pub remote: IpAddr,
    pub body: Vec<u8>,
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(&'static str, String)>,
    pub body: Vec<u8>,
}

impl Response {
    fn html(status: u16, body: String) -> Self {
        Self {
            status,
            headers: vec![("Content-Type", "text/html; charset=utf-8".into())],
            body: body.into_bytes(),
        }
    }

    fn redirect(to: &str) -> Self {
        Self {
            status: 303,
            headers: vec![("Location", to.into())],
            body: Vec::new(),
        }
    }

    fn cookie(mut self, name: &str, value: &str, max_age: u64) -> Self {
        self.headers.push((
            "Set-Cookie",
            format!("{name}={value}; Path=/; HttpOnly; SameSite=Strict; Max-Age={max_age}"),
        ));
        self
    }

    pub fn with_security_headers(mut self) -> Self {
        for (name, value) in [
            (
                "Content-Security-Policy",
                "default-src 'none'; style-src 'self'; img-src 'self'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'",
            ),
            ("X-Content-Type-Options", "nosniff"),
            ("X-Frame-Options", "DENY"),
            ("Referrer-Policy", "no-referrer"),
            ("Cache-Control", "no-store"),
        ] {
            self.headers.push((name, value.into()));
        }
        self
    }
}

pub struct App {
    config: Config,
    passwords: PasswordStore,
    setup_code: Option<SetupCode>,
    sessions: Sessions,
    throttle: Throttle,
}

impl App {
    pub fn new(config: Config) -> Self {
        let passwords = PasswordStore::new(config.state_dir.join("password"));
        let mut app = Self {
            config,
            passwords,
            setup_code: None,
            sessions: Sessions::default(),
            throttle: Throttle::default(),
        };
        app.refresh_setup_code(Instant::now());
        app
    }

    fn code_path(&self) -> PathBuf {
        self.config.runtime_dir.join("setup-code")
    }

    /// Keep a valid code published for the TV while no password exists.
    fn refresh_setup_code(&mut self, now: Instant) {
        if self.passwords.is_set() {
            self.setup_code = None;
            let _ = fs::remove_file(self.code_path());
            return;
        }
        if self.setup_code.as_ref().is_none_or(|c| c.expired(now)) {
            let code = SetupCode::generate();
            if let Err(error) = code.publish(&self.code_path()) {
                eprintln!("dede-tv-admin: cannot publish setup code: {error}");
            }
            self.setup_code = Some(code);
        }
    }

    #[cfg(test)]
    pub fn current_setup_code(&self) -> Option<String> {
        self.setup_code.as_ref().map(|c| c.code().to_string())
    }

    fn lang(&self) -> Lang {
        Lang::from_config(&self.config.ui_config)
    }

    pub fn handle(&mut self, request: &Request) -> Response {
        let now = Instant::now();
        let lang = self.lang();
        if request.method == "POST" && !same_origin(request) {
            return Response::html(403, html::rejected(lang));
        }
        let session =
            cookie(request, SESSION_COOKIE).filter(|token| self.sessions.get(token, now).is_some());
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/style.css") => Response {
                status: 200,
                headers: vec![("Content-Type", "text/css; charset=utf-8".into())],
                body: STYLE.as_bytes().to_vec(),
            },
            ("GET", "/") => {
                if !self.passwords.is_set() {
                    Response::redirect("/setup")
                } else if let Some(token) = session {
                    let csrf = self.sessions.get(&token, now).unwrap().csrf.clone();
                    Response::html(200, html::home(lang, &csrf, None))
                } else {
                    Response::redirect("/login")
                }
            }
            ("GET", "/setup") => {
                if self.passwords.is_set() {
                    return Response::redirect("/");
                }
                self.refresh_setup_code(now);
                let token = random_token();
                Response::html(200, html::setup(lang, &token, None)).cookie(
                    FORM_COOKIE,
                    &token,
                    3600,
                )
            }
            ("POST", "/setup") => self.post_setup(request, lang, now),
            ("GET", "/login") => {
                if !self.passwords.is_set() {
                    return Response::redirect("/setup");
                }
                let token = random_token();
                Response::html(200, html::login(lang, &token, None)).cookie(
                    FORM_COOKIE,
                    &token,
                    3600,
                )
            }
            ("POST", "/login") => self.post_login(request, lang, now),
            ("POST", "/logout") => {
                if let Some(token) = session {
                    let form = parse_form(&request.body);
                    let csrf = &self.sessions.get(&token, now).unwrap().csrf;
                    if !constant_time_eq(field(&form, "csrf").as_bytes(), csrf.as_bytes()) {
                        return Response::html(403, html::rejected(lang));
                    }
                    self.sessions.remove(&token);
                }
                Response::redirect("/login").cookie(SESSION_COOKIE, "", 0)
            }
            _ => Response::html(404, html::rejected(lang)),
        }
    }

    fn post_setup(&mut self, request: &Request, lang: Lang, now: Instant) -> Response {
        if self.passwords.is_set() {
            return Response::redirect("/");
        }
        let form = parse_form(&request.body);
        let Some(token) = form_token(request, &form) else {
            return Response::html(403, html::rejected(lang));
        };
        let again =
            |message| Response::html(200, html::setup(lang, &token, Some(Notice::Error(message))));
        if !self.throttle.allowed(request.remote, now) {
            return again(Text::TooManyAttempts);
        }
        let expired = self.setup_code.as_ref().is_none_or(|c| c.expired(now));
        if expired {
            self.refresh_setup_code(now);
            return again(Text::CodeExpired);
        }
        let code_ok = self
            .setup_code
            .as_ref()
            .is_some_and(|c| c.matches(field(&form, "code"), now));
        if !code_ok {
            self.throttle.failed(request.remote, now);
            return again(Text::WrongCode);
        }
        if let Err(problem) = check_new_password(field(&form, "password"), field(&form, "repeat")) {
            return again(match problem {
                PasswordProblem::TooShort => Text::PasswordTooShort,
                PasswordProblem::TooLong => Text::PasswordTooLong,
                PasswordProblem::Mismatch => Text::PasswordMismatch,
            });
        }
        if let Err(error) = self.passwords.set(field(&form, "password")) {
            eprintln!("dede-tv-admin: cannot store password: {error}");
            return again(Text::SavingFailed);
        }
        eprintln!(
            "dede-tv-admin: administrator password set from {}",
            request.remote
        );
        self.throttle.succeeded(request.remote);
        self.refresh_setup_code(now);
        self.sessions.clear();
        let session = self.sessions.create(now);
        let csrf = self.sessions.get(&session, now).unwrap().csrf.clone();
        Response::html(
            200,
            html::home(lang, &csrf, Some(Notice::Ok(Text::PasswordSaved))),
        )
        .cookie(SESSION_COOKIE, &session, crate::auth::SESSION_TTL.as_secs())
        .cookie(FORM_COOKIE, "", 0)
    }

    fn post_login(&mut self, request: &Request, lang: Lang, now: Instant) -> Response {
        if !self.passwords.is_set() {
            return Response::redirect("/setup");
        }
        let form = parse_form(&request.body);
        let Some(token) = form_token(request, &form) else {
            return Response::html(403, html::rejected(lang));
        };
        if !self.throttle.allowed(request.remote, now) {
            return Response::html(
                200,
                html::login(lang, &token, Some(Notice::Error(Text::TooManyAttempts))),
            );
        }
        if !self.passwords.verify(field(&form, "password")) {
            self.throttle.failed(request.remote, now);
            eprintln!("dede-tv-admin: failed sign-in from {}", request.remote);
            return Response::html(
                200,
                html::login(lang, &token, Some(Notice::Error(Text::WrongPassword))),
            );
        }
        self.throttle.succeeded(request.remote);
        eprintln!("dede-tv-admin: signed in from {}", request.remote);
        let session = self.sessions.create(now);
        Response::redirect("/")
            .cookie(SESSION_COOKIE, &session, crate::auth::SESSION_TTL.as_secs())
            .cookie(FORM_COOKIE, "", 0)
    }
}

/// POSTs must come from a page of this server (browser-set Origin).
fn same_origin(request: &Request) -> bool {
    match (&request.origin, &request.host) {
        (Some(origin), Some(host)) => {
            origin == &format!("http://{host}") || origin == &format!("https://{host}")
        }
        // Browsers always send Origin on form POSTs; refuse otherwise.
        _ => false,
    }
}

/// Pre-login forms: the hidden field must equal the form cookie.
fn form_token(request: &Request, form: &[(String, String)]) -> Option<String> {
    let cookie = cookie(request, FORM_COOKIE)?;
    let submitted = field(form, "form");
    (cookie.len() == 64 && constant_time_eq(cookie.as_bytes(), submitted.as_bytes()))
        .then_some(cookie)
}

fn cookie(request: &Request, name: &str) -> Option<String> {
    request
        .cookie
        .as_deref()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_string())
        .filter(|value| !value.is_empty())
}

fn field<'a>(form: &'a [(String, String)], name: &str) -> &'a str {
    form.iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
        .unwrap_or("")
}

pub fn parse_form(body: &[u8]) -> Vec<(String, String)> {
    if body.len() > MAX_FORM_BYTES {
        return Vec::new();
    }
    body.split(|b| *b == b'&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, |b| *b == b'=');
            let key = percent_decode(parts.next()?)?;
            let value = percent_decode(parts.next().unwrap_or(b""))?;
            Some((key, value))
        })
        .take(32)
        .collect()
}

fn percent_decode(input: &[u8]) -> Option<String> {
    let mut out = Vec::with_capacity(input.len());
    let mut i = 0;
    while i < input.len() {
        match input[i] {
            b'+' => out.push(b' '),
            b'%' => {
                let hex = input.get(i + 1..i + 3)?;
                out.push(u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?);
                i += 2;
            }
            byte => out.push(byte),
        }
        i += 1;
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        app: App,
        dir: PathBuf,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn fixture() -> Fixture {
        let dir = std::env::temp_dir().join(format!("dede-app-{}", random_token()));
        fs::create_dir_all(dir.join("state")).unwrap();
        fs::create_dir_all(dir.join("run")).unwrap();
        let app = App::new(Config {
            state_dir: dir.join("state"),
            runtime_dir: dir.join("run"),
            ui_config: dir.join("ui.conf"),
        });
        Fixture { app, dir }
    }

    fn get(app: &mut App, path: &str, cookie: Option<&str>) -> Response {
        app.handle(&Request {
            method: "GET".into(),
            path: path.into(),
            host: Some("tv:8080".into()),
            origin: None,
            cookie: cookie.map(String::from),
            remote: "192.168.1.5".parse().unwrap(),
            body: Vec::new(),
        })
    }

    fn post(app: &mut App, path: &str, cookie: &str, body: &str) -> Response {
        app.handle(&Request {
            method: "POST".into(),
            path: path.into(),
            host: Some("tv:8080".into()),
            origin: Some("http://tv:8080".into()),
            cookie: Some(cookie.into()),
            remote: "192.168.1.5".parse().unwrap(),
            body: body.as_bytes().to_vec(),
        })
    }

    fn set_cookie(response: &Response, name: &str) -> Option<String> {
        response
            .headers
            .iter()
            .filter(|(k, _)| *k == "Set-Cookie")
            .filter_map(|(_, v)| v.strip_prefix(&format!("{name}=")))
            .map(|v| v.split(';').next().unwrap().to_string())
            .next()
    }

    fn text(response: &Response) -> String {
        String::from_utf8(response.body.clone()).unwrap()
    }

    fn header<'a>(response: &'a Response, name: &str) -> Option<&'a str> {
        response
            .headers
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| v.as_str())
    }

    fn complete_setup(f: &mut Fixture) -> String {
        let page = get(&mut f.app, "/setup", None);
        let token = set_cookie(&page, FORM_COOKIE).unwrap();
        let code = f.app.current_setup_code().unwrap();
        let reply = post(
            &mut f.app,
            "/setup",
            &format!("{FORM_COOKIE}={token}"),
            &format!("form={token}&code={code}&password=Mein+Passwort%21&repeat=Mein+Passwort%21"),
        );
        assert!(text(&reply).contains("Das Passwort ist gespeichert."));
        set_cookie(&reply, SESSION_COOKIE).unwrap()
    }

    #[test]
    fn first_start_publishes_a_code_and_requires_setup() {
        let mut f = fixture();
        let published = fs::read_to_string(f.dir.join("run/setup-code")).unwrap();
        assert_eq!(Some(published), f.app.current_setup_code());
        let reply = get(&mut f.app, "/", None);
        assert_eq!(reply.status, 303);
        assert_eq!(header(&reply, "Location"), Some("/setup"));
        assert_eq!(
            header(&get(&mut f.app, "/login", None), "Location"),
            Some("/setup")
        );
    }

    #[test]
    fn setup_sets_password_signs_in_and_retires_the_code() {
        let mut f = fixture();
        let session = complete_setup(&mut f);
        assert!(!f.dir.join("run/setup-code").exists());
        assert!(f.app.current_setup_code().is_none());
        let home = get(
            &mut f.app,
            "/",
            Some(&format!("{SESSION_COOKIE}={session}")),
        );
        assert_eq!(home.status, 200);
        assert!(text(&home).contains("Verwaltung"));
        // A second setup is impossible once a password exists.
        assert_eq!(
            header(&get(&mut f.app, "/setup", None), "Location"),
            Some("/")
        );
    }

    #[test]
    fn wrong_code_and_weak_passwords_are_refused() {
        let mut f = fixture();
        let token = set_cookie(&get(&mut f.app, "/setup", None), FORM_COOKIE).unwrap();
        let cookie = format!("{FORM_COOKIE}={token}");
        let code = f.app.current_setup_code().unwrap();
        let wrong = if code == "00000000" {
            "11111111"
        } else {
            "00000000"
        };
        let reply = post(
            &mut f.app,
            "/setup",
            &cookie,
            &format!("form={token}&code={wrong}&password=langes-passwort&repeat=langes-passwort"),
        );
        assert!(text(&reply).contains("Der Code stimmt nicht."));
        let reply = post(
            &mut f.app,
            "/setup",
            &cookie,
            &format!("form={token}&code={code}&password=kurz&repeat=kurz"),
        );
        assert!(text(&reply).contains("zu kurz"));
        let reply = post(
            &mut f.app,
            "/setup",
            &cookie,
            &format!("form={token}&code={code}&password=langes-passwort&repeat=anderes-passwort"),
        );
        assert!(text(&reply).contains("nicht gleich"));
        assert!(!f.app.passwords.is_set());
    }

    #[test]
    fn setup_code_guessing_is_throttled() {
        let mut f = fixture();
        let token = set_cookie(&get(&mut f.app, "/setup", None), FORM_COOKIE).unwrap();
        let cookie = format!("{FORM_COOKIE}={token}");
        let code = f.app.current_setup_code().unwrap();
        let wrong = if code == "00000000" {
            "11111111"
        } else {
            "00000000"
        };
        for _ in 0..5 {
            post(
                &mut f.app,
                "/setup",
                &cookie,
                &format!(
                    "form={token}&code={wrong}&password=langes-passwort&repeat=langes-passwort"
                ),
            );
        }
        let reply = post(
            &mut f.app,
            "/setup",
            &cookie,
            &format!("form={token}&code={code}&password=langes-passwort&repeat=langes-passwort"),
        );
        assert!(text(&reply).contains("Zu viele Versuche"));
        assert!(!f.app.passwords.is_set());
    }

    #[test]
    fn login_and_logout() {
        let mut f = fixture();
        complete_setup(&mut f);
        let token = set_cookie(&get(&mut f.app, "/login", None), FORM_COOKIE).unwrap();
        let cookie = format!("{FORM_COOKIE}={token}");
        let wrong = post(
            &mut f.app,
            "/login",
            &cookie,
            &format!("form={token}&password=falsch"),
        );
        assert!(text(&wrong).contains("Das Passwort stimmt nicht."));
        assert!(set_cookie(&wrong, SESSION_COOKIE).is_none());
        let ok = post(
            &mut f.app,
            "/login",
            &cookie,
            &format!("form={token}&password=Mein+Passwort%21"),
        );
        assert_eq!(header(&ok, "Location"), Some("/"));
        let session = set_cookie(&ok, SESSION_COOKIE).unwrap();
        let session_cookie = format!("{SESSION_COOKIE}={session}");
        let home = text(&get(&mut f.app, "/", Some(&session_cookie)));
        let csrf = home
            .split(r#"name="csrf" value=""#)
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        // Logout without the CSRF token is refused.
        assert_eq!(
            post(&mut f.app, "/logout", &session_cookie, "csrf=nope").status,
            403
        );
        let out = post(
            &mut f.app,
            "/logout",
            &session_cookie,
            &format!("csrf={csrf}"),
        );
        assert_eq!(header(&out, "Location"), Some("/login"));
        assert_eq!(
            header(&get(&mut f.app, "/", Some(&session_cookie)), "Location"),
            Some("/login")
        );
    }

    #[test]
    fn cross_site_and_tokenless_posts_are_refused() {
        let mut f = fixture();
        let token = set_cookie(&get(&mut f.app, "/setup", None), FORM_COOKIE).unwrap();
        let code = f.app.current_setup_code().unwrap();
        let body =
            format!("form={token}&code={code}&password=langes-passwort&repeat=langes-passwort");
        let mut foreign = Request {
            method: "POST".into(),
            path: "/setup".into(),
            host: Some("tv:8080".into()),
            origin: Some("http://evil.example".into()),
            cookie: Some(format!("{FORM_COOKIE}={token}")),
            remote: "192.168.1.5".parse().unwrap(),
            body: body.into_bytes(),
        };
        assert_eq!(f.app.handle(&foreign).status, 403);
        foreign.origin = None;
        assert_eq!(f.app.handle(&foreign).status, 403);
        // Same origin, but the form token does not match the cookie.
        let reply = post(
            &mut f.app,
            "/setup",
            &format!("{FORM_COOKIE}={}", "a".repeat(64)),
            &format!("form={token}&code={code}&password=langes-passwort&repeat=langes-passwort"),
        );
        assert_eq!(reply.status, 403);
        assert!(!f.app.passwords.is_set());
    }

    #[test]
    fn interface_follows_the_configured_language() {
        let mut f = fixture();
        fs::write(f.dir.join("ui.conf"), "LANGUAGE=tr\n").unwrap();
        assert!(text(&get(&mut f.app, "/setup", None)).contains("Kurulum"));
        fs::write(f.dir.join("ui.conf"), "LANGUAGE=en\n").unwrap();
        assert!(text(&get(&mut f.app, "/setup", None)).contains("Setup code from the TV"));
    }

    #[test]
    fn form_parsing() {
        assert_eq!(
            parse_form(b"a=1&b=%C5%9Fifre+x&c"),
            vec![
                ("a".into(), "1".into()),
                ("b".into(), "şifre x".into()),
                ("c".into(), "".into())
            ]
        );
        assert!(parse_form(b"a=%ZZ").is_empty());
        assert!(parse_form(&vec![b'a'; MAX_FORM_BYTES + 1]).is_empty());
    }
}
