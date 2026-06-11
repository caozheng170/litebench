import { useEffect } from "react";
import { useAgent } from "../useAgent";
import type { BenchResult } from "../types";

interface Props {
  onResult: (r: BenchResult) => void;
}

export function AgentConnect({ onResult }: Props) {
  const { state, progress, result, retry } = useAgent();

  useEffect(() => {
    if (state === "done" && result) onResult(result);
  }, [state, result, onResult]);

  if (state === "running" || (state === "done" && !result)) {
    const pct = Math.round((progress?.progress ?? 0) * 100);
    return (
      <div className="agent running">
        <div className="agent-icon spin">⏳</div>
        <h2>正在检测本机…</h2>
        <p className="agent-label">{progress?.label ?? "测试中…"}</p>
        <div className="progress">
          <div className="progress-bar" style={{ width: `${pct}%` }} />
        </div>
        <div className="progress-pct">{pct}%</div>
      </div>
    );
  }

  return (
    <div className="agent searching">
      <div className="agent-icon">🔌</div>
      <h2>等待本地助手…</h2>
      <p>
        为获得<b>精准的型号、批次与生产年份</b>，需要运行一个轻量本地助手（浏览器无法直接读取硬件）。
        页面会自动连接，无需手动上传。
      </p>
      <ol className="steps">
        <li>
          下载并运行 <code>bench-agent</code>（无需安装）。
        </li>
        <li>
          助手会自动跑分并在本机开放 <code>127.0.0.1:38291</code>。
        </li>
        <li>本页面检测到后会自动显示结果。</li>
      </ol>
      <div className="agent-status">
        <span className="dot searching" /> 正在监听本地助手…
        <button className="btn ghost sm" onClick={retry}>
          重试
        </button>
      </div>
    </div>
  );
}
