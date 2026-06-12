use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SEQ_SIZE: usize = 128 * 1024 * 1024; // sequential test file size
const CHUNK: usize = 8 * 1024 * 1024; // sequential IO chunk
const ALIGN: usize = 4096; // sector alignment for unbuffered IO
const RAND_BLOCK: usize = 4096; // 4K random reads
const RAND_SECONDS: f64 = 2.5; // time budget for the random-read phase

/// A heap buffer whose data pointer is aligned to `ALIGN`, as required by
/// Windows unbuffered (FILE_FLAG_NO_BUFFERING) IO.
struct AlignedBuf {
    raw: Vec<u8>,
    offset: usize,
    len: usize,
}

impl AlignedBuf {
    fn new(len: usize, fill: u8) -> Self {
        let raw = vec![fill; len + ALIGN];
        let offset = raw.as_ptr().align_offset(ALIGN);
        AlignedBuf { raw, offset, len }
    }
    fn slice(&self) -> &[u8] {
        &self.raw[self.offset..self.offset + self.len]
    }
    fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.raw[self.offset..self.offset + self.len]
    }
}

#[cfg(windows)]
fn open_write_direct(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_NO_BUFFERING: u32 = 0x2000_0000;
    const FILE_FLAG_WRITE_THROUGH: u32 = 0x8000_0000;
    OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .custom_flags(FILE_FLAG_NO_BUFFERING | FILE_FLAG_WRITE_THROUGH)
        .open(path)
}

#[cfg(windows)]
fn open_read_direct(path: &Path) -> std::io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_NO_BUFFERING: u32 = 0x2000_0000;
    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_NO_BUFFERING)
        .open(path)
}

// On non-Windows platforms fall back to buffered IO (O_DIRECT has stricter,
// fs-dependent semantics; acceptable for now since precise detection is
// Windows-first anyway). Numbers there may be cache-inflated.
#[cfg(not(windows))]
fn open_write_direct(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().create(true).write(true).truncate(true).open(path)
}

#[cfg(not(windows))]
fn open_read_direct(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().read(true).open(path)
}

/// Returns (sequential_MB_per_sec, random_4k_read_IOPS).
///
/// Accuracy design: IO bypasses the OS page cache on Windows
/// (FILE_FLAG_NO_BUFFERING + WRITE_THROUGH), so this measures the actual
/// device, not RAM. The random phase is time-bounded so slow HDDs finish
/// quickly instead of grinding through a fixed op count.
pub fn run() -> (f64, f64) {
    // The temp drive can be nearly full (needs SEQ_SIZE free); fall back to
    // the working directory, which on a portable exe is usually the download
    // location the user actually cares about.
    let candidates: Vec<PathBuf> = vec![
        std::env::temp_dir(),
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    ];

    for dir in candidates {
        let path = dir.join("bench_agent_diskprobe.bin");
        match measure(&path) {
            Ok(result) => {
                let _ = std::fs::remove_file(&path);
                return result;
            }
            Err(_) => {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    (0.0, 0.0)
}

fn measure(path: &Path) -> std::io::Result<(f64, f64)> {
    // --- Sequential write ---
    let wbuf = AlignedBuf::new(CHUNK, 0xAB);
    let mut file = open_write_direct(path)?;
    let mut written = 0usize;
    let wstart = Instant::now();
    while written < SEQ_SIZE {
        file.write_all(wbuf.slice())?;
        written += CHUNK;
    }
    file.sync_all()?;
    let write_secs = wstart.elapsed().as_secs_f64();
    drop(file);

    // --- Sequential read ---
    let mut rbuf = AlignedBuf::new(CHUNK, 0);
    let mut rfile = open_read_direct(path)?;
    let mut read_total = 0usize;
    let rstart = Instant::now();
    loop {
        let n = rfile.read(rbuf.slice_mut())?;
        if n == 0 {
            break;
        }
        read_total += n;
    }
    let read_secs = rstart.elapsed().as_secs_f64();
    drop(rfile);

    let write_mbs = (written as f64 / 1e6) / write_secs;
    let read_mbs = (read_total as f64 / 1e6) / read_secs;
    let seq_mbs = (write_mbs + read_mbs) / 2.0;

    // --- Random 4K reads (time-bounded) ---
    let mut file = open_read_direct(path)?;
    let max_block = (SEQ_SIZE / RAND_BLOCK) as u64;
    let mut block = AlignedBuf::new(RAND_BLOCK, 0);
    let mut rng: u64 = 0x9E3779B97F4A7C15;
    let budget = Duration::from_secs_f64(RAND_SECONDS);
    let mut ops = 0u64;
    let start = Instant::now();
    while start.elapsed() < budget {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = rng % max_block;
        file.seek(SeekFrom::Start(idx * RAND_BLOCK as u64))?;
        file.read_exact(block.slice_mut())?;
        ops += 1;
    }
    let elapsed = start.elapsed().as_secs_f64();
    let iops = ops as f64 / elapsed;

    Ok((seq_mbs, iops))
}
