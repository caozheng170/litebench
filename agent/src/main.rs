mod bench_cpu;
mod bench_disk;
mod bench_mem;
mod browser;
mod dmi;
mod score;
mod server;
mod state;
mod types;
mod util;

use state::AppState;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use types::{BenchResult, Hardware, RawMetrics};

const DEFAULT_ADDR: &str = "127.0.0.1:38291";

fn main() {
    let addr = std::env::var("BENCH_AGENT_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let out_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "result.json".to_string());

    eprintln!("== bench-agent ==");

    let state = Arc::new(Mutex::new(AppState::default()));

    // Run the benchmark in the background so the HTTP server can report live
    // progress to the web page that is polling it.
    let bench_state = Arc::clone(&state);
    let bench_handle = std::thread::spawn(move || {
        run_benchmark(bench_state, &out_path);
    });

    // Serve the embedded UI + API until the user closes the agent.
    let server_state = Arc::clone(&state);
    let serve_addr = addr.clone();
    std::thread::spawn(move || {
        server::serve(&serve_addr, server_state);
    });

    // Wait until the HTTP server is actually listening, then launch the default
    // browser (with retries — a fixed short sleep was unreliable on slow disks).
    let url = format!("http://{}", addr);
    if browser::wait_for_server(&addr, Duration::from_secs(10)) {
        eprintln!("正在打开浏览器: {}", url);
        browser::open(&url);
    } else {
        eprintln!("本地服务启动超时，请手动打开: {}", url);
    }

    let _ = bench_handle.join();
    eprintln!("跑分完成。本地服务保持运行，关闭此窗口即可退出。");

    // Keep the process (and HTTP server) alive after the benchmark finishes so
    // the page can still fetch /result.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

fn run_benchmark(state: Arc<Mutex<AppState>>, out_path: &str) {
    macro_rules! phase {
        ($p:expr, $l:expr, $f:expr) => {{
            let mut s = state.lock().unwrap();
            s.set_phase($p, $l, $f);
            eprintln!("[{}] {}", $p, $l);
        }};
    }

    phase!("hardware", "读取精准硬件信息…", 0.05);
    let (detail, ov) = dmi::collect();

    let threads_fallback = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let hardware = Hardware {
        cpu_brand: ov.cpu_name.clone().unwrap_or_else(|| "Unknown CPU".to_string()),
        cpu_cores: ov.cpu_cores.unwrap_or(threads_fallback),
        cpu_threads: ov.cpu_threads.unwrap_or(threads_fallback),
        cpu_max_clock_mhz: ov.cpu_max_clock_mhz.unwrap_or(0),
        mem_total_gb: ov.total_mem_gb.unwrap_or(0.0),
        gpu_name: ov
            .gpu_name
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Unknown GPU".to_string()),
        os: ov.os.clone().unwrap_or_else(|| "Unknown OS".to_string()),
    };

    eprintln!("    CPU: {}", hardware.cpu_brand);
    if let Some(sys) = &detail.system {
        eprintln!("    整机: {} {}", sys.manufacturer, sys.model);
    }
    eprintln!("    内存: {} GB | OS: {}", hardware.mem_total_gb, hardware.os);

    phase!("cpu", "CPU 跑分（预热 + 多轮取中位数）…", 0.15);
    let (cpu_single, cpu_multi) = bench_cpu::run();

    phase!("memory", "内存带宽测试（多轮取中位数）…", 0.6);
    let mem_bw = bench_mem::run();

    phase!("disk", "磁盘真实读写测试（绕过系统缓存）…", 0.8);
    let (disk_seq, disk_rand) = bench_disk::run();

    phase!("scoring", "计算综合评分…", 0.95);
    let raw = RawMetrics {
        cpu_single_ops: cpu_single,
        cpu_multi_ops: cpu_multi,
        mem_bandwidth_gbs: mem_bw,
        disk_seq_mbs: disk_seq,
        disk_rand_iops: disk_rand,
    };
    let (subscores, total_score, disk_tier) = score::compute(&raw);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let result = BenchResult {
        schema: "bench-result/v2".to_string(),
        hardware,
        detail,
        subscores,
        total_score,
        disk_tier: disk_tier.as_str().to_string(),
        raw,
        timestamp,
    };

    let json = serde_json::to_string_pretty(&result).expect("serialize result");
    if let Err(e) = std::fs::write(out_path, &json) {
        eprintln!("写入 {} 失败: {}", out_path, e);
    }

    {
        let mut s = state.lock().unwrap();
        s.progress.phase = "done".to_string();
        s.progress.label = "完成".to_string();
        s.progress.progress = 1.0;
        s.progress.done = true;
        s.result = Some(result.clone());
    }

    eprintln!();
    eprintln!("综合评分: {}", result.total_score);
    eprintln!(
        "  CPU {} | 内存 {} | 存储 {}",
        result.subscores.cpu, result.subscores.memory, result.subscores.disk
    );
    eprintln!("结果已写入 {}", out_path);
}
