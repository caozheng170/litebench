/// Median of a sample set; returns 0.0 for an empty set.
pub fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = v.len() / 2;
    if v.len() % 2 == 0 {
        (v[mid - 1] + v[mid]) / 2.0
    } else {
        v[mid]
    }
}

/// Run `f` `rounds` times and take the median, suppressing one-off noise
/// (background processes, scheduler hiccups, thermal blips).
pub fn median_of(rounds: usize, mut f: impl FnMut() -> f64) -> f64 {
    median((0..rounds).map(|_| f()).collect())
}
