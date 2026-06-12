import { useEffect } from "react";
import { useAgent } from "../useAgent";
import type { BenchResult } from "../types";

interface Props {
  onResult: (r: BenchResult) => void;
}

export function AgentConnect({ onResult }: Props) {
  const { state, progress, result, error, retry } = useAgent();

  useEffect(() => {
    if (state === "done" && result) onResult(result);
  }, [state, result, onResult]);

  if (state === "error") {
    return (
      <div className="agent offline">
        <div className="agent-icon">⚠️</div>
        <h2>连接异常</h2>
        <p className="muted">{error}</p>
        <button className="btn primary" onClick={retry}>
          重试
        </button>
      </div>
    );
  }

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

  if (state === "offline") {
    return (
      <div className="agent offline">
        <div className="agent-icon">⛔</div>
        <h2>本地助手未运行</h2>
        <p>
          检测助手已关闭（例如您关掉了黑色命令行窗口）。<b>仅刷新网页无法重新检测</b>
          ，需要再次启动助手程序。
        </p>
        <ol className="steps">
          <li>
            重新<b>双击运行</b> <code>bench-agent.exe</code>。
          </li>
          <li>助手会自动打开浏览器并开始新一轮检测。</li>
          <li>若浏览器未自动打开，请手动访问 <code>127.0.0.1:38291</code>。</li>
        </ol>
        <div className="agent-status">
          <span className="dot offline" /> 等待助手重新启动…
          <button className="btn ghost sm" onClick={retry}>
            重试连接
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="agent searching">
      <div className="agent-icon">🔌</div>
      <h2>等待本地助手…</h2>
      <p>
        为获得<b>精准的型号、批次与生产年份</b>，需要运行轻量本地助手（浏览器无法直接读取硬件）。
        页面会自动连接，无需手动上传。
      </p>
      <ol className="steps">
        <li>
          双击运行 <code>bench-agent.exe</code>（无需安装）。
        </li>
        <li>助手会自动跑分并在本机开放 <code>127.0.0.1:38291</code>。</li>
        <li>本页面检测到后会自动显示结果。</li>
      </ol>
      <div className="agent-status">
        <span className="dot searching" /> 正在连接本地助手…
        <button className="btn ghost sm" onClick={retry}>
          重试
        </button>
      </div>
    </div>
  );
}
