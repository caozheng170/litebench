import type { BenchResult } from "../types";

/**
 * Reference machines recalibrated for v0.2+ scoring:
 * - Unbuffered disk IO (real device speed, not OS cache)
 * - HDD / SSD scored on separate baselines
 *
 * Totals are hand-tuned to match typical honest scores on each tier.
 */
export interface ReferenceMachine {
  label: string;
  tier: "entry" | "mainstream" | "highend";
  total: number;
  cpu: number;
  memory: number;
  disk: number;
  diskTier: "hdd" | "ssd";
}

export const REFERENCE_MACHINES: ReferenceMachine[] = [
  {
    label: "入门级笔记本 (2019)",
    tier: "entry",
    total: 520,
    cpu: 450,
    memory: 480,
    disk: 580,
    diskTier: "hdd",
  },
  {
    label: "主流办公机 (2021)",
    tier: "mainstream",
    total: 620,
    cpu: 600,
    memory: 660,
    disk: 700,
    diskTier: "ssd",
  },
  {
    label: "游戏台式机 (2022)",
    tier: "highend",
    total: 820,
    cpu: 780,
    memory: 820,
    disk: 880,
    diskTier: "ssd",
  },
  {
    label: "高端工作站 (2024)",
    tier: "highend",
    total: 1100,
    cpu: 1050,
    memory: 1100,
    disk: 1050,
    diskTier: "ssd",
  },
];

/** A sample result so the UI is demonstrable without running the native agent. */
export const SAMPLE_RESULT: BenchResult = {
  schema: "bench-result/v2",
  hardware: {
    cpuBrand: "AMD Ryzen 7 5800X 8-Core Processor",
    cpuCores: 8,
    cpuThreads: 16,
    cpuMaxClockMHz: 3800,
    memTotalGB: 32,
    gpuName: "NVIDIA GeForce RTX 3070",
    os: "Windows 11",
  },
  detail: {
    system: {
      manufacturer: "ASUSTeK Computer Inc.",
      model: "System Product Name",
      family: "Desktop",
    },
    motherboard: {
      manufacturer: "ASUSTeK COMPUTER INC.",
      product: "ROG STRIX B550-F GAMING",
      version: "Rev X.0x",
      serial: "MB-2107-XXXXX",
    },
    biosReleaseDate: "2021-08-12",
    memoryModules: [
      {
        slot: "DIMM_A2",
        manufacturer: "G.Skill",
        partNumber: "F4-3600C16-16GVKC",
        capacityGB: 16,
        speedMHz: 3600,
        serial: "00000000",
        estProductionYear: null,
      },
      {
        slot: "DIMM_B2",
        manufacturer: "G.Skill",
        partNumber: "F4-3600C16-16GVKC",
        capacityGB: 16,
        speedMHz: 3600,
        serial: "00000000",
        estProductionYear: null,
      },
    ],
    gpus: [{ name: "NVIDIA GeForce RTX 3070", vramGB: 8, driverVersion: "31.0.15.3179" }],
    disks: [
      {
        model: "Samsung SSD 980 PRO 1TB",
        serial: "S5GXNX0R000000",
        sizeGB: 931.5,
        mediaType: "SSD",
        interfaceType: "SCSI",
        estProductionYear: null,
      },
    ],
    systemEstProductionYear: 2021,
    notes: [
      "整机生产年份约为 2021 年（依据 BIOS 发布日期 2021-08-12，为估算值）。",
      "内存/磁盘的精确生产周年存于 SPD/SMART，用户态无法直接读取；上方展示的是真实批次标识（厂商/料号/序列号）。",
    ],
  },
  subscores: { cpu: 920, memory: 950, disk: 980 },
  totalScore: 940,
  diskTier: "ssd",
  raw: {
    cpuSingleOps: 110000,
    cpuMultiOps: 1100000,
    memBandwidthGBs: 17.1,
    diskSeqMBs: 1450,
    diskRandIOPS: 11000,
  },
  timestamp: 1718000000,
};

export const AGENT_BASE_URL = import.meta.env.DEV ? "http://127.0.0.1:38291" : "";

const DISK_TIER_LABEL: Record<string, string> = {
  hdd: "机械硬盘 (HDD)",
  ssd: "固态硬盘 (SSD)",
};

export function diskTierLabel(tier: string | undefined): string {
  return DISK_TIER_LABEL[tier ?? ""] ?? "未知";
}

/**
 * Interpolate percentile across the sorted reference totals so a score sitting
 * between two reference machines gets a sensible rank (not a coarse bucket).
 */
export function percentileOf(total: number): number {
  const sorted = [...REFERENCE_MACHINES].sort((a, b) => a.total - b.total);
  const n = sorted.length;

  if (total <= sorted[0].total) return 10;
  if (total >= sorted[n - 1].total) return 90;

  for (let i = 0; i < n - 1; i++) {
    const lo = sorted[i];
    const hi = sorted[i + 1];
    if (total >= lo.total && total <= hi.total) {
      const frac = (total - lo.total) / (hi.total - lo.total);
      const pctLo = ((i + 1) / n) * 100;
      const pctHi = ((i + 2) / n) * 100;
      return Math.round(pctLo + frac * (pctHi - pctLo));
    }
  }
  return 50;
}
