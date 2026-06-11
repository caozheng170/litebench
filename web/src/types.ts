export interface Hardware {
  cpuBrand: string;
  cpuCores: number;
  cpuThreads: number;
  cpuMaxClockMHz: number;
  memTotalGB: number;
  gpuName: string;
  os: string;
}

export interface SystemInfo {
  manufacturer: string;
  model: string;
  family: string;
}

export interface Motherboard {
  manufacturer: string;
  product: string;
  version: string;
  serial: string;
}

export interface MemoryModule {
  slot: string;
  manufacturer: string;
  partNumber: string;
  capacityGB: number;
  speedMHz: number;
  serial: string;
  estProductionYear: number | null;
}

export interface Gpu {
  name: string;
  vramGB: number;
  driverVersion: string;
}

export interface Disk {
  model: string;
  serial: string;
  sizeGB: number;
  mediaType: string;
  interfaceType: string;
  estProductionYear: number | null;
}

export interface HwDetail {
  system: SystemInfo | null;
  motherboard: Motherboard | null;
  biosReleaseDate: string | null;
  memoryModules: MemoryModule[];
  gpus: Gpu[];
  disks: Disk[];
  systemEstProductionYear: number | null;
  notes: string[];
}

export interface SubScores {
  cpu: number;
  memory: number;
  disk: number;
}

export interface RawMetrics {
  cpuSingleOps: number;
  cpuMultiOps: number;
  memBandwidthGBs: number;
  diskSeqMBs: number;
  diskRandIOPS: number;
}

export interface BenchResult {
  schema: string;
  hardware: Hardware;
  detail: HwDetail;
  subscores: SubScores;
  totalScore: number;
  raw: RawMetrics;
  timestamp: number;
}

export interface AgentProgress {
  phase: string;
  label: string;
  progress: number;
  done: boolean;
}

export function isBenchResult(x: unknown): x is BenchResult {
  if (!x || typeof x !== "object") return false;
  const o = x as Record<string, unknown>;
  return (
    typeof o.schema === "string" &&
    o.schema.startsWith("bench-result/") &&
    typeof o.hardware === "object" &&
    typeof o.subscores === "object" &&
    typeof o.totalScore === "number"
  );
}
