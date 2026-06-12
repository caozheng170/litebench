use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::time::Instant;

/// One unit of mixed CPU work: integer mixing + floating point + SHA-256 hashing.
/// Returns a checksum so the optimizer cannot elide the work.
fn work_unit(seed: u64) -> u64 {
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let mut f = (seed as f64).sqrt() + 1.0;

    for i in 0..256u64 {
        // integer mixing (xorshift-ish)
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x = x.wrapping_add(i.wrapping_mul(2654435761));

        // floating point work
        f = (f * 1.0000001 + 0.5).sqrt() + (f.sin() * f.cos());
    }

    let mut hasher = Sha256::new();
    hasher.update(x.to_le_bytes());
    hasher.update(f.to_le_bytes());
    let digest = hasher.finalize();

    let mut acc = 0u64;
    for chunk in digest.chunks_exact(8) {
        acc ^= u64::from_le_bytes(chunk.try_into().unwrap());
    }
    acc
}

fn count_ops(duration_secs: f64, threads: usize) -> f64 {
    let start = Instant::now();
    let target = std::time::Duration::from_secs_f64(duration_secs);

    // Run in batches so we measure ops/sec; checksum keeps work alive.
    let mut total: u64 = 0;
    let mut checksum: u64 = 0;
    let batch: u64 = 4096;

    while start.elapsed() < target {
        if threads <= 1 {
            for i in 0..batch {
                checksum ^= work_unit(total.wrapping_add(i));
            }
        } else {
            checksum ^= (0..batch)
                .into_par_iter()
                .map(|i| work_unit(total.wrapping_add(i)))
                .reduce(|| 0u64, |a, b| a ^ b);
        }
        total += batch;
    }

    let elapsed = start.elapsed().as_secs_f64();
    // Keep checksum observable.
    std::hint::black_box(checksum);
    total as f64 / elapsed
}

/// Returns (single_core_ops_per_sec, multi_core_ops_per_sec).
///
/// Stability design: a discarded warm-up lets CPU clocks ramp up first, then
/// each mode runs 3 timed rounds and reports the median, so one-off background
/// interference doesn't swing the final score.
pub fn run() -> (f64, f64) {
    let threads = rayon::current_num_threads().max(1);

    let _ = count_ops(0.5, 1); // warm-up, discarded
    let single = crate::util::median_of(3, || count_ops(2.0, 1));

    let _ = count_ops(0.5, threads); // warm-up, discarded
    let multi = crate::util::median_of(3, || count_ops(2.0, threads));

    (single, multi)
}
