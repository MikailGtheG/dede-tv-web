//! HTTP front end for the administration console.
//!
//! Each request runs in its own thread; the app state sits behind a mutex.
//! Film uploads stream to disk without holding the lock.
use dede_tv_admin::app::{same_origin, App, Config, Request, Response, MAX_FORM_BYTES};
use dede_tv_admin::{channels, html, media};
use std::env;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const MAX_THREADS: usize = 16;

fn setting(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

fn header(request: &tiny_http::Request, name: &'static str) -> Option<String> {
    request
        .headers()
        .iter()
        .find(|h| h.field.equiv(name))
        .map(|h| h.value.as_str().to_string())
}

fn respond(request: tiny_http::Request, reply: Response) {
    let reply = reply.with_security_headers();
    let mut response = tiny_http::Response::from_data(reply.body).with_status_code(reply.status);
    for (name, value) in reply.headers {
        if let Ok(h) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
            response.add_header(h);
        }
    }
    let _ = request.respond(response);
}

fn redirect(to: String) -> Response {
    Response {
        status: 303,
        headers: vec![("Location", to)],
        body: Vec::new(),
    }
}

/// `/channels/<n>/upload` → n
fn upload_channel(path: &str) -> Option<u32> {
    let rest = path.strip_prefix("/channels/")?.strip_suffix("/upload")?;
    channels::parse_number(rest)
}

fn upload(app: &Mutex<App>, mut request: tiny_http::Request, envelope: Request, number: u32) {
    let (csrf, dir, lang) = {
        let app = app.lock().unwrap();
        let lang = app.lang_now();
        let csrf = app.session_csrf(envelope.cookie.as_deref());
        (csrf, app.upload_target(number), lang)
    };
    let back = format!("/channels/{number}");
    let (Some(csrf), true) = (csrf, same_origin(&envelope)) else {
        return respond(request, Response::forbidden(html::rejected(lang)));
    };
    let Some(dir) = dir else {
        return respond(request, redirect(format!("{back}?n=failed")));
    };
    let Some(boundary) = header(&request, "Content-Type").and_then(|t| media::boundary(&t)) else {
        return respond(request, Response::forbidden(html::rejected(lang)));
    };
    let result = media::receive(request.as_reader(), &boundary, &csrf, &dir);
    let code = match &result {
        Ok(stored) => {
            eprintln!(
                "dede-tv-admin: {} film(s) uploaded to channel {number} from {}",
                stored.len(),
                envelope.remote
            );
            "uploaded"
        }
        Err(media::UploadError::Forbidden) => {
            return respond(request, Response::forbidden(html::rejected(lang)))
        }
        Err(media::UploadError::BadName) => "bad-name",
        Err(media::UploadError::Exists) => "exists",
        Err(e) => {
            eprintln!("dede-tv-admin: upload to channel {number} failed: {e:?}");
            "failed"
        }
    };
    respond(request, redirect(format!("{back}?n={code}")));
}

fn serve(app: &Mutex<App>, mut request: tiny_http::Request) {
    let Some(remote) = request.remote_addr().map(|a| a.ip()) else {
        return;
    };
    let (path, query) = match request.url().split_once('?') {
        Some((p, q)) => (p.to_string(), q.to_string()),
        None => (request.url().to_string(), String::new()),
    };
    let mut envelope = Request {
        method: request.method().as_str().to_string(),
        path,
        query,
        host: header(&request, "Host"),
        origin: header(&request, "Origin"),
        cookie: header(&request, "Cookie"),
        remote,
        body: Vec::new(),
    };
    if envelope.method == "POST" {
        if let Some(number) = upload_channel(&envelope.path) {
            return upload(app, request, envelope, number);
        }
    }
    let too_large = request.body_length().is_some_and(|n| n > MAX_FORM_BYTES);
    if !too_large {
        let _ = request
            .as_reader()
            .take(MAX_FORM_BYTES as u64 + 1)
            .read_to_end(&mut envelope.body);
    }
    if too_large || envelope.body.len() > MAX_FORM_BYTES {
        let _ = request.respond(tiny_http::Response::empty(413));
        return;
    }
    let reply = app.lock().unwrap().handle(&envelope);
    respond(request, reply);
}

fn main() {
    let listen = setting("DEDE_ADMIN_LISTEN", "0.0.0.0:8080");
    let app = Arc::new(Mutex::new(App::new(Config {
        state_dir: PathBuf::from(setting("DEDE_ADMIN_STATE", "/var/lib/dede-tv-admin")),
        runtime_dir: PathBuf::from(setting("DEDE_ADMIN_RUNTIME", "/run/dede-tv-admin")),
        ui_config: PathBuf::from(setting("DEDE_UI_CONFIG", "/etc/dede-tv/ui.conf")),
        channels_config: PathBuf::from(setting(
            "DEDE_CHANNELS_CONFIG",
            "/etc/dede-tv/virtual-channels.conf",
        )),
        media_root: PathBuf::from(setting("DEDE_MEDIA_ROOT", "/srv/dede-tv/media")),
    })));
    let server = match tiny_http::Server::http(&listen) {
        Ok(server) => Arc::new(server),
        Err(error) => {
            eprintln!("dede-tv-admin: cannot listen on {listen}: {error}");
            std::process::exit(1);
        }
    };
    eprintln!("dede-tv-admin: listening on {listen}");
    let workers: Vec<_> = (0..MAX_THREADS)
        .map(|_| {
            let (server, app) = (Arc::clone(&server), Arc::clone(&app));
            std::thread::spawn(move || {
                for request in server.incoming_requests() {
                    serve(&app, request);
                }
            })
        })
        .collect();
    for worker in workers {
        let _ = worker.join();
    }
}
