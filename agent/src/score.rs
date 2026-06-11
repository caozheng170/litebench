use crate::types::{RawMetrics, SubScores};

// Baseline constants, calibrated so a mid-range reference machine scores ~1000
// per subsystem. These are first-pass estimates and should be re-calibrated
// once real-world distributions are collected via the backend.
const BASE_SINGLE: f64 = 120_000.0; // single-core ops/sec
const BASE_MULTI: f64 = 900_000.0; // multi-core ops/sec
const BASE_MEM_GBS: f64 = 18.0; // memory bandwidth GB/s
const BASE_DISK_SEQ: f64 = 1500.0; // sequential MB/s
const BASE_DISK_RAND: f64 = 12_000.0; // random 4K IOPS

// Subsystem weights for the overall score.
const W_CPU: f64 = 0.5;
const W_MEM: f64 = 0.2;
const W_DISK: f64 = 0.3;

pub fn compute(raw: &RawMetrics) -> (SubScores, f64) {
    let cpu = (raw.cpu_single_ops / BASE_SINGLE * 0.4 + raw.cpu_multi_ops / BASE_MULTI * 0.6)
        * 1000.0;
    let memory = raw.mem_bandwidth_gbs / BASE_MEM_GBS * 1000.0;
    let disk = (raw.disk_seq_mbs / BASE_DISK_SEQ * 0.5 + raw.disk_rand_iops / BASE_DISK_RAND * 0.5)
        * 1000.0;

    let total = cpu * W_CPU + memory * W_MEM + disk * W_DISK;

    let round1 = |x: f64| (x * 10.0).round() / 10.0;
    (
        SubScores {
            cpu: round1(cpu),
            memory: round1(memory),
            disk: round1(disk),
        },
        round1(total),
    )
}
