import type { BenchResult } from "../types";

/**
 * Built-in reference machines used for the "network comparison" charts in the MVP.
 * Once the backend + database land, this static sample set is replaced by real,
 * percentile-based comparisons aggregated by hardware model.
 */
export interface ReferenceMachine {
  label: string;
  tier: "entry" | "mainstream" | "highend";
  total: number;
  cpu: number;
  memory: number;
  disk: number;
}

export const REFERENCE_MACHINES: ReferenceMachine[] = [
  { label: "入门级笔记本 (2019)", tier: "entry", total: 620, cpu: 580, memory: 700, disk: 520 },
  { label: "主流办公机 (2021)", tier: "mainstream", total: 1000, cpu: 1000, memory: 1000, disk: 1000 },
  { label: "游戏台式机 (2022)", tier: "highend", total: 1850, cpu: 1900, memory: 1600, disk: 1750 },
  { label: "高端工作站 (2024)", tier: "highend", total: 2900, cpu: 3100, memory: 2400, disk: 2600 },
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
  subscores: { cpu: 1820, memory: 1240, disk: 1560 },
  totalScore: 1626,
  raw: {
    cpuSingleOps: 138000,
    cpuMultiOps: 1380000,
    memBandwidthGBs: 22.3,
    diskSeqMBs: 2400,
    diskRandIOPS: 18600,
  },
  timestamp: 1718000000,
};

/**
 * Where to reach the native agent.
 * - Production: the exe serves this very page, so use a same-origin (relative)
 *   base — no CORS, no mixed-content issues.
 * - Dev (vite on :5173): talk to the agent on its fixed port.
 */
export const AGENT_BASE_URL = import.meta.env.DEV ? "http://127.0.0.1:38291" : "";

/**
 * Compute the approximate percentile of a total score against the reference set
 * (linear interpolation over the sorted reference totals).
 */
export function percentileOf(total: number): number {
  const totals = REFERENCE_MACHINES.map((m) => m.total).sort((a, b) => a - b);
  let below = 0;
  for (const t of totals) if (t < total) below++;
  return Math.round((below / totals.length) * 100);
}
