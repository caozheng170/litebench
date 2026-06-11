use crate::types::{Disk, Gpu, HwDetail, MemoryModule, Motherboard, SystemInfo};

/// Authoritative values pulled from CIM/WMI that should override the generic
/// `sysinfo` summary so the report matches what Windows "System" page shows.
#[derive(Debug, Clone, Default)]
pub struct DmiOverrides {
    pub total_mem_gb: Option<f64>,
    pub cpu_name: Option<String>,
    pub cpu_max_clock_mhz: Option<u64>,
    pub cpu_cores: Option<usize>,
    pub cpu_threads: Option<usize>,
    pub gpu_name: Option<String>,
    pub os: Option<String>,
}

/// Collect precise hardware identity + batch info and infer an approximate
/// production year.
///
/// Honest scope note: precise per-module manufacture week/year lives in the
/// memory SPD EEPROM and is NOT exposed to user-space without a kernel driver.
/// So we surface the real *batch* identifiers (manufacturer / part number /
/// serial) and infer the *system* production year from the BIOS release date,
/// which is a reliable proxy.
#[cfg(windows)]
pub fn collect() -> (HwDetail, DmiOverrides) {
    match query_windows() {
        Some(x) => x,
        None => (
            HwDetail {
                notes: vec!["精准硬件检测失败：无法读取 CIM/WMI 数据。".to_string()],
                ..Default::default()
            },
            DmiOverrides::default(),
        ),
    }
}

#[cfg(not(windows))]
pub fn collect() -> (HwDetail, DmiOverrides) {
    (
        HwDetail {
            notes: vec!["精准硬件检测目前仅在 Windows 上实现（其它平台待补充 SMBIOS/DMI 解析）。".to_string()],
            ..Default::default()
        },
        DmiOverrides::default(),
    )
}

#[cfg(windows)]
const PS_SCRIPT: &str = r#"
$ErrorActionPreference='SilentlyContinue'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$bios = Get-CimInstance Win32_BIOS
$bb   = Get-CimInstance Win32_BaseBoard
$cs   = Get-CimInstance Win32_ComputerSystem
$csp  = Get-CimInstance Win32_ComputerSystemProduct
$cpu  = Get-CimInstance Win32_Processor | Select-Object -First 1
$os   = Get-CimInstance Win32_OperatingSystem
$mem  = Get-CimInstance Win32_PhysicalMemory
$gpu  = Get-CimInstance Win32_VideoController
$disk = Get-CimInstance Win32_DiskDrive
$biosDate = $null; $sysYear = $null
if ($bios.ReleaseDate) { $biosDate = $bios.ReleaseDate.ToString('yyyy-MM-dd'); $sysYear = $bios.ReleaseDate.Year }
$o = [ordered]@{
  biosReleaseDate = $biosDate
  systemYear = $sysYear
  os = ((([string]$os.Caption) + ' ' + ([string]$os.Version)).Trim())
  system = @{ manufacturer=[string]$cs.Manufacturer; model=[string]$cs.Model; family=[string]$csp.Name; totalMemory=[string]$cs.TotalPhysicalMemory }
  cpu = @{ name=[string]$cpu.Name; maxClock=[int]$cpu.MaxClockSpeed; cores=[int]$cpu.NumberOfCores; threads=[int]$cpu.NumberOfLogicalProcessors }
  motherboard = @{ manufacturer=[string]$bb.Manufacturer; product=[string]$bb.Product; version=[string]$bb.Version; serial=[string]$bb.SerialNumber }
  memory = @($mem | ForEach-Object { @{ slot=[string]$_.DeviceLocator; manufacturer=[string]$_.Manufacturer; partNumber=[string]$_.PartNumber; capacity=[string]$_.Capacity; speed=[int]$(if ($_.Speed) {$_.Speed} elseif ($_.ConfiguredClockSpeed) {$_.ConfiguredClockSpeed} else {0}); serial=[string]$_.SerialNumber } })
  gpus = @($gpu | ForEach-Object { @{ name=[string]$_.Name; vram=[string]$_.AdapterRAM; driver=[string]$_.DriverVersion } })
  disks = @($disk | ForEach-Object { @{ model=[string]$_.Model; serial=[string]$_.SerialNumber; size=[string]$_.Size; media=[string]$_.MediaType; iface=[string]$_.InterfaceType } })
}
$o | ConvertTo-Json -Depth 5 -Compress
"#;

#[cfg(windows)]
fn query_windows() -> Option<(HwDetail, DmiOverrides)> {
    use std::process::Command;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", PS_SCRIPT])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;

    let mut notes: Vec<String> = Vec::new();
    let mut ov = DmiOverrides::default();

    let bios_release_date = str_field(&v, "biosReleaseDate");
    let system_year = v.get("systemYear").and_then(|x| x.as_i64()).map(|y| y as i32);
    ov.os = str_field(&v, "os");

    // --- System (marketing model + total RAM) ---
    let system = v.get("system").map(|s| SystemInfo {
        manufacturer: str_field(s, "manufacturer").unwrap_or_default(),
        model: str_field(s, "model").unwrap_or_default(),
        family: str_field(s, "family").unwrap_or_default(),
    });
    let total_physical_gb = v.get("system").and_then(|s| {
        str_field(s, "totalMemory")
            .and_then(|t| t.parse::<f64>().ok())
            .map(|b| (b / 1024.0 / 1024.0 / 1024.0 * 10.0).round() / 10.0)
    });

    // --- CPU authoritative values ---
    if let Some(c) = v.get("cpu") {
        ov.cpu_name = str_field(c, "name");
        ov.cpu_max_clock_mhz = c.get("maxClock").and_then(|x| x.as_i64()).map(|x| x as u64);
        ov.cpu_cores = c.get("cores").and_then(|x| x.as_i64()).map(|x| x as usize);
        ov.cpu_threads = c.get("threads").and_then(|x| x.as_i64()).map(|x| x as usize);
    }

    let motherboard = v.get("motherboard").map(|m| Motherboard {
        manufacturer: str_field(m, "manufacturer").unwrap_or_default(),
        product: str_field(m, "product").unwrap_or_default(),
        version: str_field(m, "version").unwrap_or_default(),
        serial: str_field(m, "serial").unwrap_or_default(),
    });

    let memory_modules: Vec<MemoryModule> = as_array(v.get("memory"))
        .iter()
        .map(|m| MemoryModule {
            slot: str_field(m, "slot").unwrap_or_default(),
            manufacturer: str_field(m, "manufacturer").unwrap_or_default(),
            part_number: str_field(m, "partNumber").unwrap_or_default(),
            capacity_gb: parse_bytes_gb(&str_field(m, "capacity").unwrap_or_default()),
            speed_mhz: str_field(m, "speed")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0),
            serial: str_field(m, "serial").unwrap_or_default(),
            est_production_year: None,
        })
        .collect();

    // Installed RAM (headline) = sum of physical modules; fall back to the
    // OS-reported usable total.
    let modules_sum_gb: f64 = memory_modules.iter().map(|m| m.capacity_gb).sum();
    ov.total_mem_gb = if modules_sum_gb > 0.0 {
        Some((modules_sum_gb * 10.0).round() / 10.0)
    } else {
        total_physical_gb
    };

    let gpus: Vec<Gpu> = as_array(v.get("gpus"))
        .iter()
        .map(|g| Gpu {
            name: str_field(g, "name").unwrap_or_default(),
            vram_gb: parse_bytes_gb(&str_field(g, "vram").unwrap_or_default()),
            driver_version: str_field(g, "driver").unwrap_or_default(),
        })
        .collect();
    ov.gpu_name = pick_primary_gpu(&gpus);

    let disks: Vec<Disk> = as_array(v.get("disks"))
        .iter()
        .map(|d| Disk {
            model: str_field(d, "model").unwrap_or_default(),
            serial: str_field(d, "serial").unwrap_or_default(),
            size_gb: parse_bytes_gb(&str_field(d, "size").unwrap_or_default()),
            media_type: str_field(d, "media").unwrap_or_default(),
            interface_type: str_field(d, "iface").unwrap_or_default(),
            est_production_year: None,
        })
        .collect();

    if let Some(year) = system_year {
        notes.push(format!(
            "整机生产年份约为 {} 年（依据 BIOS 发布日期 {}，为估算值）。",
            year,
            bios_release_date.clone().unwrap_or_else(|| "未知".into())
        ));
    }
    notes.push(
        "内存/磁盘的精确生产周年存于 SPD/SMART，用户态无法直接读取；上方展示的是真实批次标识（厂商/料号/序列号）。"
            .to_string(),
    );

    let detail = HwDetail {
        system,
        motherboard,
        bios_release_date,
        memory_modules,
        gpus,
        disks,
        system_est_production_year: system_year,
        notes,
    };
    Some((detail, ov))
}

/// Choose the GPU a user actually cares about: skip virtual / remote-desktop
/// display adapters and prefer a discrete card, then integrated.
#[cfg(windows)]
fn pick_primary_gpu(gpus: &[Gpu]) -> Option<String> {
    const VIRTUAL: &[&str] = &[
        "virtual", "idd", "oray", "todesk", "parsec", "sunshine", "spacedesk", "duet",
        "remote", "meeting", "basic display", "basic render", "mirror", "rdp",
    ];
    let is_virtual = |n: &str| {
        let l = n.to_lowercase();
        VIRTUAL.iter().any(|k| l.contains(k))
    };
    let real: Vec<&str> = gpus
        .iter()
        .map(|g| g.name.as_str())
        .filter(|n| !n.is_empty() && !is_virtual(n))
        .collect();

    let discrete = real.iter().find(|n| {
        let l = n.to_lowercase();
        l.contains("nvidia") || l.contains("geforce") || l.contains("radeon") || l.contains("amd")
    });
    discrete
        .copied()
        .or_else(|| real.first().copied())
        .or_else(|| gpus.iter().map(|g| g.name.as_str()).find(|n| !n.is_empty()))
        .map(|s| s.to_string())
}

#[cfg(windows)]
fn str_field(v: &serde_json::Value, key: &str) -> Option<String> {
    let s = v.get(key)?.as_str()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// PowerShell `ConvertTo-Json` collapses a single-element collection into an
/// object and an empty one into null. Normalize all three shapes into a Vec.
#[cfg(windows)]
fn as_array(v: Option<&serde_json::Value>) -> Vec<serde_json::Value> {
    match v {
        Some(serde_json::Value::Array(a)) => a.clone(),
        Some(serde_json::Value::Object(_)) => vec![v.unwrap().clone()],
        _ => Vec::new(),
    }
}

#[cfg(windows)]
fn parse_bytes_gb(s: &str) -> f64 {
    s.trim()
        .parse::<f64>()
        .map(|b| (b / 1024.0 / 1024.0 / 1024.0 * 10.0).round() / 10.0)
        .unwrap_or(0.0)
}
