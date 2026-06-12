import type { BenchResult } from "../types";
import { diskTierLabel, percentileOf } from "../data/baseline";

interface Props {
  result: BenchResult;
}

function Stat({ label, value, unit }: { label: string; value: string; unit?: string }) {
  return (
    <div className="stat">
      <span className="stat-label">{label}</span>
      <span className="stat-value">
        {value}
        {unit && <span className="stat-unit"> {unit}</span>}
      </span>
    </div>
  );
}

export function ScoreCard({ result }: Props) {
  const { hardware, subscores, totalScore, raw } = result;
  const pct = percentileOf(totalScore);

  return (
    <div className="card score-card">
      <div className="total">
        <div className="total-num">{Math.round(totalScore)}</div>
        <div className="total-label">综合性能评分</div>
        <div className="total-pct">超过约 {pct}% 的参考机型</div>
      </div>

      <div className="sub-scores">
        <div className="sub cpu">
          <span>CPU</span>
          <b>{Math.round(subscores.cpu)}</b>
        </div>
        <div className="sub mem">
          <span>内存</span>
          <b>{Math.round(subscores.memory)}</b>
        </div>
        <div className="sub disk">
          <span>存储</span>
          <b>{Math.round(subscores.disk)}</b>
          {result.diskTier && (
            <span className="sub-hint">{diskTierLabel(result.diskTier)}</span>
          )}
        </div>
      </div>

      <div className="hw">
        <h3>检测到的硬件</h3>
        <div className="hw-grid">
          <Stat label="处理器" value={hardware.cpuBrand} />
          <Stat label="核心 / 线程" value={`${hardware.cpuCores} / ${hardware.cpuThreads}`} />
          {hardware.cpuMaxClockMHz > 0 && (
            <Stat label="主频" value={(hardware.cpuMaxClockMHz / 1000).toFixed(2)} unit="GHz" />
          )}
          <Stat label="内存容量" value={String(hardware.memTotalGB)} unit="GB" />
          <Stat label="显卡" value={hardware.gpuName} />
          <Stat label="操作系统" value={hardware.os} />
        </div>
      </div>

      <div className="hw">
        <h3>原始测试数据</h3>
        <div className="hw-grid">
          <Stat label="单核" value={raw.cpuSingleOps.toLocaleString()} unit="ops/s" />
          <Stat label="多核" value={raw.cpuMultiOps.toLocaleString()} unit="ops/s" />
          <Stat label="内存带宽" value={raw.memBandwidthGBs.toFixed(1)} unit="GB/s" />
          <Stat label="磁盘顺序" value={raw.diskSeqMBs.toFixed(0)} unit="MB/s" />
          <Stat label="磁盘随机" value={raw.diskRandIOPS.toFixed(0)} unit="IOPS" />
        </div>
      </div>
    </div>
  );
}
