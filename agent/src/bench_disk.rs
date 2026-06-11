use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::time::Instant;

const SEQ_SIZE: usize = 256 * 1024 * 1024; // 256 MiB sequential test file
const RAND_BLOCK: usize = 4096; // 4K random IO
const RAND_OPS: usize = 4000;

/// Returns (sequential_MB_per_sec, random_4k_IOPS).
///
/// NOTE: this measures the real filesystem in the system temp dir. It is an
/// approximation of raw disk speed (OS caching can inflate numbers). A future
/// version can use direct/unbuffered IO for more accurate figures.
pub fn run() -> (f64, f64) {
    let path = std::env::temp_dir().join("bench_agent_diskprobe.bin");

    let seq = sequential(&path).unwrap_or(0.0);
    let iops = random_iops(&path).unwrap_or(0.0);

    let _ = std::fs::remove_file(&path);
    (seq, iops)
}

fn sequential(path: &std::path::Path) -> std::io::Result<f64> {
    let chunk = vec![0xABu8; 8 * 1024 * 1024];
    let mut written = 0usize;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;

    let start = Instant::now();
    while written < SEQ_SIZE {
        file.write_all(&chunk)?;
        written += chunk.len();
    }
    file.flush()?;
    file.sync_all()?;
    let write_elapsed = start.elapsed().as_secs_f64();
    drop(file);

    // Sequential read back.
    let mut rfile = File::open(path)?;
    let mut rbuf = vec![0u8; 8 * 1024 * 1024];
    let mut read_total = 0usize;
    let rstart = Instant::now();
    loop {
        let n = rfile.read(&mut rbuf)?;
        if n == 0 {
            break;
        }
        read_total += n;
    }
    let read_elapsed = rstart.elapsed().as_secs_f64();

    let write_mbs = (written as f64 / 1e6) / write_elapsed;
    let read_mbs = (read_total as f64 / 1e6) / read_elapsed;
    // Report the average of read and write throughput.
    Ok((write_mbs + read_mbs) / 2.0)
}

fn random_iops(path: &std::path::Path) -> std::io::Result<f64> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let file_len = file.metadata()?.len();
    if file_len < RAND_BLOCK as u64 {
        return Ok(0.0);
    }
    let max_block = (file_len / RAND_BLOCK as u64) as u64;

    let mut block = vec![0u8; RAND_BLOCK];
    // simple LCG for pseudo-random offsets
    let mut rng: u64 = 0x9E3779B97F4A7C15;
    let start = Instant::now();
    for _ in 0..RAND_OPS {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = rng % max_block;
        file.seek(SeekFrom::Start(idx * RAND_BLOCK as u64))?;
        file.read_exact(&mut block)?;
    }
    let elapsed = start.elapsed().as_secs_f64();
    Ok(RAND_OPS as f64 / elapsed)
}
