use std::time::Instant;

/// Estimate memory bandwidth in GB/s via large sequential read+write passes.
pub fn run() -> f64 {
    // 128 MiB buffer of u64 (large enough to defeat most caches).
    let len = 128 * 1024 * 1024 / std::mem::size_of::<u64>();
    let mut buf: Vec<u64> = vec![0u64; len];

    // Warm up / touch all pages.
    for (i, v) in buf.iter_mut().enumerate() {
        *v = i as u64;
    }

    let passes = 6;
    let start = Instant::now();
    let mut acc = 0u64;
    for p in 0..passes {
        // read + write each element
        for v in buf.iter_mut() {
            *v = v.wrapping_add(p as u64).wrapping_mul(2654435761);
            acc ^= *v;
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    std::hint::black_box(acc);

    // Each pass touches the whole buffer for read and write.
    let bytes = (len * std::mem::size_of::<u64>()) as f64 * passes as f64 * 2.0;
    (bytes / elapsed) / 1e9
}
