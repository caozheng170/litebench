mod bench_cpu;
mod bench_disk;
mod bench_mem;
mod benchmark;
mod browser;
mod dmi;
mod score;
mod server;
mod state;
mod types;
mod util;

use state::AppState;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const DEFAULT_ADDR: &str = "127.0.0.1:38291";

fn main() {
    let addr = std::env::var("BENCH_AGENT_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "result.json".to_string());

    eprintln!("== bench-agent ==");

    let state = Arc::new(Mutex::new(AppState::default()));
    let out = out_path.clone();

    // First benchmark run in the background.
    {
        let state = Arc::clone(&state);
        let out = out.clone();
        std::thread::spawn(move || benchmark::run(state, &out));
    }

    // HTTP server (UI + API + POST /rerun for a fresh benchmark).
    let server_state = Arc::clone(&state);
    let server_addr = addr.clone();
    std::thread::spawn(move || server::serve(&server_addr, server_state, out));

    let url = format!("http://{}", addr);
    if browser::wait_for_server(&addr, Duration::from_secs(10)) {
        eprintln!("正在打开浏览器: {}", url);
        browser::open(&url);
    } else {
        eprintln!("本地服务启动超时，请手动打开: {}", url);
    }

    eprintln!("助手运行中。关闭此窗口即可退出。");
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}
