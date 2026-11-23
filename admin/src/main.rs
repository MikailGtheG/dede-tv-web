//! HTTP front end for the administration console (single-threaded).
use dede_tv_admin::app::{App, Config, Request, MAX_FORM_BYTES};
use std::env;
use std::io::Read;
use std::path::PathBuf;

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

fn main() {
    let listen = setting("DEDE_ADMIN_LISTEN", "0.0.0.0:8080");
    let mut app = App::new(Config {
        state_dir: PathBuf::from(setting("DEDE_ADMIN_STATE", "/var/lib/dede-tv-admin")),
        runtime_dir: PathBuf::from(setting("DEDE_ADMIN_RUNTIME", "/run/dede-tv-admin")),
        ui_config: PathBuf::from(setting("DEDE_UI_CONFIG", "/etc/dede-tv/ui.conf")),
    });
    let server = match tiny_http::Server::http(&listen) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("dede-tv-admin: cannot listen on {listen}: {error}");
            std::process::exit(1);
        }
    };
    eprintln!("dede-tv-admin: listening on {listen}");
    for mut request in server.incoming_requests() {
        let Some(remote) = request.remote_addr().map(|a| a.ip()) else {
            continue;
        };
        let mut body = Vec::new();
        let too_large = request.body_length().is_some_and(|n| n > MAX_FORM_BYTES);
        if !too_large {
            let _ = request
                .as_reader()
                .take(MAX_FORM_BYTES as u64 + 1)
                .read_to_end(&mut body);
        }
        if too_large || body.len() > MAX_FORM_BYTES {
            let _ = request.respond(tiny_http::Response::empty(413));
            continue;
        }
        let path = request.url().split('?').next().unwrap_or("/").to_string();
        let reply = app
            .handle(&Request {
                method: request.method().as_str().to_string(),
                path,
                host: header(&request, "Host"),
                origin: header(&request, "Origin"),
                cookie: header(&request, "Cookie"),
                remote,
                body,
            })
            .with_security_headers();
        let mut response =
            tiny_http::Response::from_data(reply.body).with_status_code(reply.status);
        for (name, value) in reply.headers {
            if let Ok(h) = tiny_http::Header::from_bytes(name.as_bytes(), value.as_bytes()) {
                response.add_header(h);
            }
        }
        let _ = request.respond(response);
    }
}
