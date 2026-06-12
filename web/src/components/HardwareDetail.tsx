import type { HwDetail } from "../types";

interface Props {
  detail: HwDetail;
}

function yearBadge(year: number | null): string {
  return year ? `约 ${year} 年` : "未知";
}

export function HardwareDetail({ detail }: Props) {
  const hasAny =
    detail.system ||
    detail.motherboard ||
    detail.memoryModules.length > 0 ||
    detail.disks.length > 0 ||
    detail.gpus.length > 0;

  if (!hasAny) {
    return (
      <div className="card">
        <h3>精准硬件信息</h3>
        <p className="muted">
          未获取到精准硬件信息（示例数据或非 Windows 环境）。运行本地助手后这里会显示型号、批次与生产年份。
        </p>
      </div>
    );
  }

  return (
    <div className="card hw-detail-print">
      <div className="detail-head">
        <h3>精准硬件信息与批次</h3>
        {detail.systemEstProductionYear && (
          <span className="year-pill">整机约 {detail.systemEstProductionYear} 年</span>
        )}
      </div>

      {detail.system && (detail.system.manufacturer || detail.system.model) && (
        <div className="detail-block">
          <h4>整机型号</h4>
          <div className="system-model">
            {detail.system.manufacturer} {detail.system.model}
          </div>
          {detail.system.family && <div className="muted">{detail.system.family}</div>}
        </div>
      )}

      {detail.motherboard && (
        <div className="detail-block">
          <h4>主板</h4>
          <div className="kv">
            <span>厂商</span>
            <b>{detail.motherboard.manufacturer || "—"}</b>
          </div>
          <div className="kv">
            <span>型号</span>
            <b>{detail.motherboard.product || "—"}</b>
          </div>
          <div className="kv">
            <span>序列号(批次)</span>
            <b>{detail.motherboard.serial || "—"}</b>
          </div>
          {detail.biosReleaseDate && (
            <div className="kv">
              <span>BIOS 日期</span>
              <b>{detail.biosReleaseDate}</b>
            </div>
          )}
        </div>
      )}

      {detail.memoryModules.length > 0 && (
        <div className="detail-block">
          <h4>内存条（{detail.memoryModules.length} 根）</h4>
          <table className="detail-table">
            <thead>
              <tr>
                <th>插槽</th>
                <th>厂商</th>
                <th>料号</th>
                <th>容量</th>
                <th>频率</th>
                <th>生产年份</th>
              </tr>
            </thead>
            <tbody>
              {detail.memoryModules.map((m, i) => (
                <tr key={i}>
                  <td>{m.slot || "—"}</td>
                  <td>{m.manufacturer || "—"}</td>
                  <td className="mono">{m.partNumber || "—"}</td>
                  <td>{m.capacityGB} GB</td>
                  <td>{m.speedMHz > 0 ? `${m.speedMHz} MHz` : "—"}</td>
                  <td>{yearBadge(m.estProductionYear)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {detail.disks.length > 0 && (
        <div className="detail-block">
          <h4>存储设备（{detail.disks.length}）</h4>
          <table className="detail-table">
            <thead>
              <tr>
                <th>型号</th>
                <th>类型</th>
                <th>容量</th>
                <th>序列号(批次)</th>
                <th>生产年份</th>
              </tr>
            </thead>
            <tbody>
              {detail.disks.map((d, i) => (
                <tr key={i}>
                  <td>{d.model || "—"}</td>
                  <td>{d.mediaType || d.interfaceType || "—"}</td>
                  <td>{d.sizeGB} GB</td>
                  <td className="mono">{d.serial || "—"}</td>
                  <td>{yearBadge(d.estProductionYear)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {detail.gpus.length > 0 && (
        <div className="detail-block">
          <h4>显卡</h4>
          {detail.gpus.map((g, i) => (
            <div className="kv" key={i}>
              <span>{g.name}</span>
              <b>
                {g.vramGB > 0 ? `${g.vramGB} GB` : ""} {g.driverVersion && `· 驱动 ${g.driverVersion}`}
              </b>
            </div>
          ))}
        </div>
      )}

      {detail.notes.length > 0 && (
        <div className="detail-notes">
          {detail.notes.map((n, i) => (
            <p key={i}>· {n}</p>
          ))}
        </div>
      )}
    </div>
  );
}
