//! Request handling, independent of the HTTP server library.
use crate::apply;
use crate::auth::{
    check_new_password, constant_time_eq, random_token, PasswordProblem, PasswordStore, Sessions,
    SetupCode, Throttle,
};
use crate::channels::{self, Channel};
use crate::html::{self, Notice};
use crate::i18n::{Lang, Text};
use crate::media;
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
    pub channels_config: PathBuf,
    pub media_root: PathBuf,
}

pub struct Request {
    pub method: String,
    pub path: String,
    pub query: String,
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

    pub fn forbidden(body: String) -> Self {
        Self::html(403, body)
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
            _ => match session {
                Some(token) => {
                    let csrf = self.sessions.get(&token, now).unwrap().csrf.clone();
                    self.signed_in(request, lang, &csrf)
                }
                None if self.passwords.is_set() => Response::redirect("/login"),
                None => Response::redirect("/setup"),
            },
        }
    }

    /// Session token's CSRF value if the request carries a valid session.
    pub fn session_csrf(&self, cookie_header: Option<&str>) -> Option<String> {
        let request = Request {
            method: String::new(),
            path: String::new(),
            query: String::new(),
            host: None,
            origin: None,
            cookie: cookie_header.map(String::from),
            remote: IpAddr::from([0, 0, 0, 0]),
            body: Vec::new(),
        };
        let token = cookie(&request, SESSION_COOKIE)?;
        self.sessions
            .get(&token, Instant::now())
            .map(|s| s.csrf.clone())
    }

    fn channel_list(&self) -> Result<Vec<Channel>, ()> {
        channels::read(&self.config.channels_config).map_err(|e| {
            eprintln!("dede-tv-admin: {e}");
        })
    }

    /// Folder of channel `number`, for streaming uploads outside the lock.
    pub fn upload_target(&self, number: u32) -> Option<PathBuf> {
        let list = self.channel_list().ok()?;
        let channel = list.iter().find(|c| c.number == number)?;
        media::channel_dir(&self.config.media_root, &channel.folder).ok()
    }

    pub fn lang_now(&self) -> Lang {
        self.lang()
    }

    fn signed_in(&mut self, request: &Request, lang: Lang, csrf: &str) -> Response {
        if request.method == "POST" {
            let form = parse_form(&request.body);
            if !constant_time_eq(field(&form, "csrf").as_bytes(), csrf.as_bytes()) {
                return Response::html(403, html::rejected(lang));
            }
            return self.signed_in_post(request, lang, csrf, &form);
        }
        if request.method != "GET" {
            return Response::html(405, html::rejected(lang));
        }
        let notice = notice_from_query(&request.query);
        let parts: Vec<&str> = request.path.trim_matches('/').split('/').collect();
        match parts.as_slice() {
            ["channels"] => match self.channel_list() {
                Ok(list) => Response::html(200, html::channels(lang, csrf, &list, notice)),
                Err(()) => Response::html(
                    500,
                    html::channels(lang, csrf, &[], Some(Notice::Error(Text::ConfigUnreadable))),
                ),
            },
            ["channels", "new"] => {
                let list = self.channel_list().unwrap_or_default();
                let number = channels::suggest_number(&list).to_string();
                Response::html(200, html::new_channel(lang, csrf, "", &number, notice))
            }
            ["channels", n] => {
                let Some((channel, films)) = self.channel_with_films(n) else {
                    return Response::html(404, html::rejected(lang));
                };
                Response::html(200, html::channel(lang, csrf, &channel, &films, notice))
            }
            ["apply", id] => {
                let back = query_value(&request.query, "back")
                    .filter(|b| b.starts_with("/channels"))
                    .unwrap_or("/channels".into());
                let state = match apply::outcome(&self.config.state_dir, id) {
                    apply::Outcome::Pending => html::ApplyState::Pending,
                    apply::Outcome::Done => html::ApplyState::Done,
                    apply::Outcome::Failed(code) => html::ApplyState::Failed(code),
                };
                Response::html(200, html::applying(lang, csrf, state, &back))
            }
            _ => Response::html(404, html::rejected(lang)),
        }
    }

    fn channel_with_films(&self, n: &str) -> Option<(Channel, Vec<media::Film>)> {
        let number = channels::parse_number(n)?;
        let channel = self
            .channel_list()
            .ok()?
            .into_iter()
            .find(|c| c.number == number)?;
        let films = media::channel_dir(&self.config.media_root, &channel.folder)
            .and_then(|dir| media::list(&dir))
            .unwrap_or_default();
        Some((channel, films))
    }

    fn submit(&self, list: &[Channel], back: &str) -> Response {
        match apply::submit_channels(&self.config.state_dir, list) {
            Ok(id) => {
                eprintln!("dede-tv-admin: channel change requested ({id})");
                Response::redirect(&format!("/apply/{id}?back={back}"))
            }
            Err(e) => {
                eprintln!("dede-tv-admin: cannot queue change: {e}");
                Response::redirect(&format!("{back}?n=failed"))
            }
        }
    }

    fn signed_in_post(
        &mut self,
        request: &Request,
        lang: Lang,
        csrf: &str,
        form: &[(String, String)],
    ) -> Response {
        let parts: Vec<&str> = request.path.trim_matches('/').split('/').collect();
        let Ok(mut list) = self.channel_list() else {
            return Response::html(
                500,
                html::channels(lang, csrf, &[], Some(Notice::Error(Text::ConfigUnreadable))),
            );
        };
        match parts.as_slice() {
            ["channels", "new"] => {
                let name = field(form, "name").trim().to_string();
                let raw_number = field(form, "number");
                let again = |text| {
                    Response::html(
                        200,
                        html::new_channel(lang, csrf, &name, raw_number, Some(Notice::Error(text))),
                    )
                };
                let number =
                    match channels::check_new(&list, &name, channels::parse_number(raw_number)) {
                        Ok(n) => n,
                        Err(channels::Problem::Name) => return again(Text::NameInvalid),
                        Err(channels::Problem::Number) => return again(Text::NumberInvalid),
                        Err(channels::Problem::NumberTaken) => return again(Text::NumberTaken),
                        Err(channels::Problem::TooMany) => return again(Text::TooManyChannels),
                    };
                let folder = channels::unique_folder(&name, &self.config.media_root, &list);
                if let Err(e) = fs::create_dir(self.config.media_root.join(&folder)) {
                    eprintln!("dede-tv-admin: cannot create channel folder: {e}");
                    return again(Text::FolderFailed);
                }
                list.push(Channel {
                    name,
                    number,
                    folder,
                });
                eprintln!(
                    "dede-tv-admin: channel {number} created from {}",
                    request.remote
                );
                self.submit(&list, &format!("/channels/{number}"))
            }
            ["apply"] => {
                let back = field(form, "back");
                let back = if back.starts_with("/channels/")
                    && back[10..].bytes().all(|b| b.is_ascii_digit())
                {
                    back.to_string()
                } else {
                    "/channels".to_string()
                };
                self.submit(&list, &back)
            }
            ["channels", n, "delete"] => {
                let Some(number) = channels::parse_number(n) else {
                    return Response::html(404, html::rejected(lang));
                };
                let before = list.len();
                list.retain(|c| c.number != number);
                if list.len() == before {
                    return Response::html(404, html::rejected(lang));
                }
                eprintln!(
                    "dede-tv-admin: channel {number} deleted from {}",
                    request.remote
                );
                self.submit(&list, "/channels")
            }
            ["channels", n, "films", "delete"] => {
                let Some((channel, _)) = self.channel_with_films(n) else {
                    return Response::html(404, html::rejected(lang));
                };
                let result = media::channel_dir(&self.config.media_root, &channel.folder)
                    .and_then(|dir| media::delete(&dir, field(form, "film")));
                let back = format!("/channels/{}", channel.number);
                match result {
                    Ok(()) => {
                        eprintln!(
                            "dede-tv-admin: film deleted in channel {} from {}",
                            channel.number, request.remote
                        );
                        Response::redirect(&format!("{back}?n=film-deleted"))
                    }
                    Err(_) => Response::redirect(&format!("{back}?n=failed")),
                }
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

/// Notices after a redirect, chosen by a fixed code (never echoed text).
fn notice_from_query(query: &str) -> Option<Notice> {
    match query_value(query, "n")?.as_str() {
        "uploaded" => Some(Notice::Ok(Text::UploadDone)),
        "film-deleted" => Some(Notice::Ok(Text::FilmDeleted)),
        "bad-name" => Some(Notice::Error(Text::UploadBadName)),
        "exists" => Some(Notice::Error(Text::UploadExists)),
        "failed" => Some(Notice::Error(Text::UploadFailed)),
        _ => None,
    }
}

pub fn query_value(query: &str, name: &str) -> Option<String> {
    parse_form(query.as_bytes())
        .into_iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v)
}

/// POSTs must come from a page of this server (browser-set Origin).
pub fn same_origin(request: &Request) -> bool {
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
        fs::create_dir_all(dir.join("media")).unwrap();
        let app = App::new(Config {
            state_dir: dir.join("state"),
            runtime_dir: dir.join("run"),
            ui_config: dir.join("ui.conf"),
            channels_config: dir.join("virtual-channels.conf"),
            media_root: dir.join("media"),
        });
        Fixture { app, dir }
    }

    fn get(app: &mut App, path: &str, cookie: Option<&str>) -> Response {
        app.handle(&Request {
            method: "GET".into(),
            path: path.split('?').next().unwrap().into(),
            query: path
                .split_once('?')
                .map(|(_, q)| q.to_string())
                .unwrap_or_default(),
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
            path: path.split('?').next().unwrap().into(),
            query: path
                .split_once('?')
                .map(|(_, q)| q.to_string())
                .unwrap_or_default(),
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
            query: String::new(),
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

    fn signed_in(f: &mut Fixture) -> (String, String) {
        let session = complete_setup(f);
        let cookie = format!("{SESSION_COOKIE}={session}");
        let home = text(&get(&mut f.app, "/", Some(&cookie)));
        let csrf = home
            .split(r#"name="csrf" value=""#)
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap()
            .to_string();
        (cookie, csrf)
    }

    fn pending_request(f: &Fixture) -> serde_json::Value {
        let dir = f.dir.join("state/requests");
        let entry = fs::read_dir(&dir).unwrap().next().unwrap().unwrap();
        let value = serde_json::from_slice(&fs::read(entry.path()).unwrap()).unwrap();
        fs::remove_file(entry.path()).unwrap();
        value
    }

    #[test]
    fn channel_pages_require_a_session() {
        let mut f = fixture();
        complete_setup(&mut f);
        for path in ["/channels", "/channels/new", "/channels/5", "/apply/00"] {
            assert_eq!(
                header(&get(&mut f.app, path, None), "Location"),
                Some("/login"),
                "{path}"
            );
        }
    }

    #[test]
    fn creating_a_channel_makes_its_folder_and_requests_the_change() {
        let mut f = fixture();
        let (cookie, csrf) = signed_in(&mut f);
        let page = text(&get(&mut f.app, "/channels/new", Some(&cookie)));
        assert!(page.contains(r#"name="number" value="5""#));
        let reply = post(
            &mut f.app,
            "/channels/new",
            &cookie,
            &format!("csrf={csrf}&name=%C3%87izgi+Film&number=7"),
        );
        let location = header(&reply, "Location").unwrap().to_string();
        assert!(
            location.starts_with("/apply/") && location.ends_with("?back=/channels/7"),
            "{location}"
        );
        assert!(f.dir.join("media/cizgi-film").is_dir());
        let request = pending_request(&f);
        assert_eq!(request["kind"], "virtual-channels");
        assert_eq!(
            request["channels"],
            serde_json::json!([{"name": "Çizgi Film", "number": 7, "folder": "cizgi-film"}])
        );
        // Pending until the root helper answers, then done.
        let status = text(&get(&mut f.app, &location, Some(&cookie)));
        assert!(status.contains(r#"http-equiv="refresh""#));
        let id = location[7..39].to_string();
        fs::create_dir_all(f.dir.join("state/results")).unwrap();
        fs::write(
            f.dir.join(format!("state/results/{id}.json")),
            r#"{"ok":true}"#,
        )
        .unwrap();
        let status = text(&get(&mut f.app, &location, Some(&cookie)));
        assert!(status.contains("Fertig.") && !status.contains("refresh"));
    }

    #[test]
    fn invalid_or_duplicate_channels_are_refused() {
        let mut f = fixture();
        fs::write(
            f.dir.join("virtual-channels.conf"),
            "[Dede Film]\nnumber = 5\nfolder = film\n",
        )
        .unwrap();
        let (cookie, csrf) = signed_in(&mut f);
        for (body, expected) in [
            ("name=Neu&number=5", "schon ein anderer"),
            ("name=Dede+Film&number=6", "neuen Namen"),
            ("name=A%5B1%5D&number=6", "neuen Namen"),
            ("name=Neu&number=0", "1 bis 9999"),
        ] {
            let reply = text(&post(
                &mut f.app,
                "/channels/new",
                &cookie,
                &format!("csrf={csrf}&{body}"),
            ));
            assert!(reply.contains(expected), "{body}");
        }
        assert!(!f.dir.join("state/requests").exists());
        // Without the CSRF token nothing happens.
        assert_eq!(
            post(&mut f.app, "/channels/new", &cookie, "name=Neu&number=6").status,
            403
        );
    }

    #[test]
    fn channel_page_lists_films_and_deletes_them() {
        let mut f = fixture();
        fs::write(
            f.dir.join("virtual-channels.conf"),
            "[Dede Film]\nnumber = 5\nfolder = film\n",
        )
        .unwrap();
        fs::create_dir_all(f.dir.join("media/film")).unwrap();
        fs::write(f.dir.join("media/film/a.mp4"), b"x").unwrap();
        fs::write(f.dir.join("media/film/notes.txt"), b"x").unwrap();
        let (cookie, csrf) = signed_in(&mut f);
        let page = text(&get(&mut f.app, "/channels/5", Some(&cookie)));
        assert!(page.contains("a.mp4") && !page.contains("notes.txt"));
        assert!(page.contains(r#"enctype="multipart/form-data""#));
        let reply = post(
            &mut f.app,
            "/channels/5/films/delete",
            &cookie,
            &format!("csrf={csrf}&film=a.mp4"),
        );
        assert_eq!(
            header(&reply, "Location"),
            Some("/channels/5?n=film-deleted")
        );
        assert!(!f.dir.join("media/film/a.mp4").exists());
        let reply = post(
            &mut f.app,
            "/channels/5/films/delete",
            &cookie,
            &format!("csrf={csrf}&film=..%2Fx.mp4"),
        );
        assert_eq!(header(&reply, "Location"), Some("/channels/5?n=failed"));
        // "Send to the TV" re-applies the unchanged list.
        let reply = post(
            &mut f.app,
            "/apply",
            &cookie,
            &format!("csrf={csrf}&back=/channels/5"),
        );
        assert!(header(&reply, "Location")
            .unwrap()
            .ends_with("?back=/channels/5"));
        assert_eq!(pending_request(&f)["channels"][0]["number"], 5);
        // Deleting the channel keeps its films' folder.
        let reply = post(
            &mut f.app,
            "/channels/5/delete",
            &cookie,
            &format!("csrf={csrf}"),
        );
        assert!(header(&reply, "Location").unwrap().starts_with("/apply/"));
        assert_eq!(pending_request(&f)["channels"], serde_json::json!([]));
        assert!(f.dir.join("media/film").is_dir());
    }

    #[test]
    fn upload_target_and_session_lookup() {
        let mut f = fixture();
        fs::write(
            f.dir.join("virtual-channels.conf"),
            "[Dede Film]\nnumber = 5\nfolder = film\n",
        )
        .unwrap();
        fs::create_dir_all(f.dir.join("media/film")).unwrap();
        let (cookie, csrf) = signed_in(&mut f);
        assert_eq!(f.app.session_csrf(Some(&cookie)), Some(csrf));
        assert_eq!(f.app.session_csrf(Some("dede_session=nope")), None);
        assert_eq!(f.app.upload_target(5), Some(f.dir.join("media/film")));
        assert_eq!(f.app.upload_target(6), None);
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
