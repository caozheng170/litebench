use std::time::Instant;

const BUF_BYTES: usize = 128 * 1024 * 1024;

/// One full read+write pass over the buffer; returns a checksum to keep the
/// work observable.
fn pass(buf: &mut [u64], salt: u64) -> u64 {
    let mut acc = 0u64;
    for v in buf.iter_mut() {
        *v = v.wrapping_add(salt).wrapping_mul(2654435761);
        acc ^= *v;
    }
    acc
}

/// Estimate memory bandwidth in GB/s over a 128 MiB buffer (large enough to
/// defeat CPU caches).
///
/// Stability design: one discarded warm-up pass (page faults, cache state),
/// then 5 individually-timed passes with the median reported.
pub fn run() -> f64 {
    let len = BUF_BYTES / std::mem::size_of::<u64>();
    let mut buf: Vec<u64> = vec![0u64; len];

    for (i, v) in buf.iter_mut().enumerate() {
        *v = i as u64;
    }

    let mut sink = pass(&mut buf, 0); // warm-up, discarded

    let bytes_per_pass = (len * std::mem::size_of::<u64>()) as f64 * 2.0; // read + write
    let samples: Vec<f64> = (1..=5u64)
        .map(|p| {
            let start = Instant::now();
            sink ^= pass(&mut buf, p);
            let elapsed = start.elapsed().as_secs_f64();
            (bytes_per_pass / elapsed) / 1e9
        })
        .collect();

    std::hint::black_box(sink);
    crate::util::median(samples)
}
