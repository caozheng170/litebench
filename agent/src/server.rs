use crate::state::AppState;
use include_dir::{include_dir, Dir};
use std::sync::{Arc, Mutex};
use tiny_http::{Header, Method, Response, Server};

/// The production web UI is built into `web/dist` and embedded into the exe at
/// compile time, so the single binary serves its own interface.
static ASSETS: Dir = include_dir!("$CARGO_MANIFEST_DIR/../web/dist");

type Resp = Response<std::io::Cursor<Vec<u8>>>;

fn mime_for(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "wasm" => "application/wasm",
        "map" => "application/json",
        _ => "application/octet-stream",
    }
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).unwrap()
}

fn with_cors(resp: Resp) -> Resp {
    resp.with_header(header("Access-Control-Allow-Origin", "*"))
        .with_header(header("Access-Control-Allow-Methods", "GET, OPTIONS"))
        .with_header(header("Access-Control-Allow-Headers", "Content-Type"))
}

fn json(code: u16, body: String) -> Resp {
    with_cors(Response::from_string(body))
        .with_status_code(code)
        .with_header(header("Content-Type", "application/json; charset=utf-8"))
}

fn serve_asset(path: &str) -> Resp {
    let clean = path.trim_start_matches('/');
    let key = if clean.is_empty() { "index.html" } else { clean };

    match ASSETS.get_file(key) {
        Some(file) => with_cors(Response::from_data(file.contents().to_vec()))
            .with_header(header("Content-Type", mime_for(key))),
        None => {
            // SPA fallback: unknown non-API path -> serve index.html if present.
            match ASSETS.get_file("index.html") {
                Some(index) => with_cors(Response::from_data(index.contents().to_vec()))
                    .with_header(header("Content-Type", "text/html; charset=utf-8")),
                None => with_cors(Response::from_string(
                    "未找到内嵌界面（构建时缺少 web/dist）。API 仍可用：/status /result",
                ))
                .with_status_code(404),
            }
        }
    }
}

/// Run the blocking HTTP server. Exposes:
///   GET /health  -> { ok: true }
///   GET /status  -> Progress JSON
///   GET /result  -> BenchResult JSON (202 until done)
///   GET /*       -> embedded web UI
pub fn serve(addr: &str, state: Arc<Mutex<AppState>>) {
    let server = match Server::http(addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("无法在 {} 启动本地服务: {}", addr, e);
            return;
        }
    };
    eprintln!("本地服务已启动: http://{}", addr);

    for request in server.incoming_requests() {
        let response = match (request.method(), request.url()) {
            (Method::Options, _) => json(204, String::new()),
            (Method::Get, "/health") => json(200, r#"{"ok":true}"#.to_string()),
            (Method::Get, "/status") => {
                let st = state.lock().unwrap();
                json(
                    200,
                    serde_json::to_string(&st.progress).unwrap_or_else(|_| "{}".into()),
                )
            }
            (Method::Get, "/result") => {
                let st = state.lock().unwrap();
                match &st.result {
                    Some(r) => json(200, serde_json::to_string(r).unwrap_or_else(|_| "{}".into())),
                    None => json(202, r#"{"error":"benchmark still running"}"#.to_string()),
                }
            }
            (Method::Get, path) => serve_asset(path),
            _ => json(404, r#"{"error":"not found"}"#.to_string()),
        };
        let _ = request.respond(response);
    }
}
