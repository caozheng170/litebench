use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hardware {
    #[serde(rename = "cpuBrand")]
    pub cpu_brand: String,
    #[serde(rename = "cpuCores")]
    pub cpu_cores: usize,
    #[serde(rename = "cpuThreads")]
    pub cpu_threads: usize,
    #[serde(rename = "cpuMaxClockMHz")]
    pub cpu_max_clock_mhz: u64,
    #[serde(rename = "memTotalGB")]
    pub mem_total_gb: f64,
    #[serde(rename = "gpuName")]
    pub gpu_name: String,
    pub os: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Marketing manufacturer, e.g. "Dell Inc."
    pub manufacturer: String,
    /// Marketing model as shown in Windows "System" page, e.g. "G3 3590".
    pub model: String,
    /// Product family / friendly name, when available.
    pub family: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Motherboard {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryModule {
    pub slot: String,
    pub manufacturer: String,
    #[serde(rename = "partNumber")]
    pub part_number: String,
    #[serde(rename = "capacityGB")]
    pub capacity_gb: f64,
    #[serde(rename = "speedMHz")]
    pub speed_mhz: u64,
    pub serial: String,
    #[serde(rename = "estProductionYear")]
    pub est_production_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gpu {
    pub name: String,
    #[serde(rename = "vramGB")]
    pub vram_gb: f64,
    #[serde(rename = "driverVersion")]
    pub driver_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Disk {
    pub model: String,
    pub serial: String,
    #[serde(rename = "sizeGB")]
    pub size_gb: f64,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "interfaceType")]
    pub interface_type: String,
    #[serde(rename = "estProductionYear")]
    pub est_production_year: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HwDetail {
    pub system: Option<SystemInfo>,
    pub motherboard: Option<Motherboard>,
    #[serde(rename = "biosReleaseDate")]
    pub bios_release_date: Option<String>,
    #[serde(rename = "memoryModules")]
    pub memory_modules: Vec<MemoryModule>,
    pub gpus: Vec<Gpu>,
    pub disks: Vec<Disk>,
    #[serde(rename = "systemEstProductionYear")]
    pub system_est_production_year: Option<i32>,
    /// Human-readable notes explaining how each estimate was derived and its confidence.
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubScores {
    pub cpu: f64,
    pub memory: f64,
    pub disk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawMetrics {
    #[serde(rename = "cpuSingleOps")]
    pub cpu_single_ops: f64,
    #[serde(rename = "cpuMultiOps")]
    pub cpu_multi_ops: f64,
    #[serde(rename = "memBandwidthGBs")]
    pub mem_bandwidth_gbs: f64,
    #[serde(rename = "diskSeqMBs")]
    pub disk_seq_mbs: f64,
    #[serde(rename = "diskRandIOPS")]
    pub disk_rand_iops: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    pub schema: String,
    pub hardware: Hardware,
    pub detail: HwDetail,
    pub subscores: SubScores,
    #[serde(rename = "totalScore")]
    pub total_score: f64,
    /// "hdd" or "ssd" — which baseline was used to score the disk subsystem.
    #[serde(rename = "diskTier")]
    pub disk_tier: String,
    pub raw: RawMetrics,
    pub timestamp: u64,
}
