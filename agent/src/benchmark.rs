use crate::state::AppState;
use crate::types::{BenchResult, Hardware, RawMetrics};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Run a full benchmark pass and write `out_path`. Safe to call from any thread;
/// concurrent runs are prevented by checking `AppState::running` before spawn.
pub fn run(state: Arc<Mutex<AppState>>, out_path: &str) {
    {
        let mut s = state.lock().unwrap();
        s.running = true;
        s.result = None;
        s.progress.done = false;
    }

    let result = match run_inner(&state) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("跑分异常: {}", e);
            let mut s = state.lock().unwrap();
            s.running = false;
            s.progress.phase = "error".to_string();
            s.progress.label = format!("检测失败: {}", e);
            s.progress.done = false;
            return;
        }
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
        s.running = false;
    }

    eprintln!();
    eprintln!("综合评分: {}", result.total_score);
    eprintln!(
        "  CPU {} | 内存 {} | 存储 {}",
        result.subscores.cpu, result.subscores.memory, result.subscores.disk
    );
    eprintln!("结果已写入 {}", out_path);
}

fn run_inner(state: &Arc<Mutex<AppState>>) -> Result<BenchResult, &'static str> {
    macro_rules! phase {
        ($p:expr, $l:expr, $f:expr) => {{
            let mut s = state.lock().unwrap();
            s.set_phase($p, $l, $f);
            eprintln!("[{}] {}", $p, $l);
        }};
    }

    phase!("hardware", "读取精准硬件信息…", 0.05);
    let (detail, ov) = crate::dmi::collect();

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
    let (cpu_single, cpu_multi) = crate::bench_cpu::run();

    phase!("memory", "内存带宽测试（多轮取中位数）…", 0.6);
    let mem_bw = crate::bench_mem::run();

    phase!("disk", "磁盘真实读写测试（绕过系统缓存）…", 0.8);
    let (disk_seq, disk_rand) = crate::bench_disk::run();

    phase!("scoring", "计算综合评分…", 0.95);
    let raw = RawMetrics {
        cpu_single_ops: cpu_single,
        cpu_multi_ops: cpu_multi,
        mem_bandwidth_gbs: mem_bw,
        disk_seq_mbs: disk_seq,
        disk_rand_iops: disk_rand,
    };
    let (subscores, total_score, disk_tier) = crate::score::compute(&raw);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    Ok(BenchResult {
        schema: "bench-result/v2".to_string(),
        hardware,
        detail,
        subscores,
        total_score,
        disk_tier: disk_tier.as_str().to_string(),
        raw,
        timestamp,
    })
}

/// Returns true if a new run was scheduled (caller should spawn `benchmark::run`).
pub fn request_rerun(state: &mut AppState) -> bool {
    if state.running {
        return false;
    }
    state.result = None;
    state.progress.phase = "starting".to_string();
    state.progress.label = "正在重新开始检测…".to_string();
    state.progress.progress = 0.0;
    state.progress.done = false;
    true
}
