use crate::types::{RawMetrics, SubScores};

/// Disk tier detected from raw throughput — HDD and SSD are scored on separate
/// baselines so a 5400 RPM laptop HDD is not compared against NVMe constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskTier {
    Hdd,
    Ssd,
}

impl DiskTier {
    pub fn as_str(self) -> &'static str {
        match self {
            DiskTier::Hdd => "hdd",
            DiskTier::Ssd => "ssd",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            DiskTier::Hdd => "机械硬盘 (HDD)",
            DiskTier::Ssd => "固态硬盘 (SSD)",
        }
    }
}

/// Classify the tested storage from unbuffered benchmark numbers.
pub fn detect_disk_tier(seq_mbs: f64, rand_iops: f64) -> DiskTier {
    if seq_mbs >= 350.0 || rand_iops >= 2500.0 {
        DiskTier::Ssd
    } else {
        DiskTier::Hdd
    }
}

// CPU / memory baselines — tuned so a mid-range desktop ≈ 650–850 per subsystem.
const BASE_SINGLE: f64 = 120_000.0;
const BASE_MULTI: f64 = 800_000.0; // slightly lower than v0.1 so 6-core mobiles aren't over-penalised
const BASE_MEM_GBS: f64 = 18.0;

// SSD baselines (unbuffered): mainstream SATA/NVMe ≈ 1000 disk score.
const BASE_DISK_SEQ_SSD: f64 = 1500.0;
const BASE_DISK_RAND_SSD: f64 = 12_000.0;

// HDD baselines (unbuffered): 5400 RPM laptop HDD ≈ 650–800 disk score.
const BASE_DISK_SEQ_HDD: f64 = 180.0;
const BASE_DISK_RAND_HDD: f64 = 270.0;

const W_CPU: f64 = 0.5;
const W_MEM: f64 = 0.2;
const W_DISK: f64 = 0.3;

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

fn disk_score(raw: &RawMetrics, tier: DiskTier) -> f64 {
    let (seq_base, rand_base) = match tier {
        DiskTier::Ssd => (BASE_DISK_SEQ_SSD, BASE_DISK_RAND_SSD),
        DiskTier::Hdd => (BASE_DISK_SEQ_HDD, BASE_DISK_RAND_HDD),
    };
    (raw.disk_seq_mbs / seq_base * 0.5 + raw.disk_rand_iops / rand_base * 0.5) * 1000.0
}

pub fn compute(raw: &RawMetrics) -> (SubScores, f64, DiskTier) {
    let cpu = (raw.cpu_single_ops / BASE_SINGLE * 0.4 + raw.cpu_multi_ops / BASE_MULTI * 0.6)
        * 1000.0;
    let memory = raw.mem_bandwidth_gbs / BASE_MEM_GBS * 1000.0;
    let tier = detect_disk_tier(raw.disk_seq_mbs, raw.disk_rand_iops);
    let disk = disk_score(raw, tier);

    let total = cpu * W_CPU + memory * W_MEM + disk * W_DISK;

    (
        SubScores {
            cpu: round1(cpu),
            memory: round1(memory),
            disk: round1(disk),
        },
        round1(total),
        tier,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hdd_laptop_scores_reasonably() {
        let raw = RawMetrics {
            cpu_single_ops: 72_000.0,
            cpu_multi_ops: 460_000.0,
            mem_bandwidth_gbs: 13.0,
            disk_seq_mbs: 130.0,
            disk_rand_iops: 230.0,
        };
        let (sub, total, tier) = compute(&raw);
        assert_eq!(tier, DiskTier::Hdd);
        assert!(sub.disk > 500.0, "HDD disk score should not be near zero: {}", sub.disk);
        assert!(total > 600.0 && total < 900.0, "gaming laptop total: {}", total);
    }

    #[test]
    fn ssd_mainstream_near_1000_disk() {
        let raw = RawMetrics {
            cpu_single_ops: 100_000.0,
            cpu_multi_ops: 800_000.0,
            mem_bandwidth_gbs: 18.0,
            disk_seq_mbs: 1500.0,
            disk_rand_iops: 12_000.0,
        };
        let (sub, _, tier) = compute(&raw);
        assert_eq!(tier, DiskTier::Ssd);
        assert!((sub.disk - 1000.0).abs() < 50.0);
    }
}
