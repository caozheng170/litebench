import { useState } from "react";
import type { BenchResult } from "./types";
import { SAMPLE_RESULT, REFERENCE_MACHINES } from "./data/baseline";
import { requestAgentRerun } from "./useAgent";
import { AgentConnect } from "./components/AgentConnect";
import { UploadResult } from "./components/UploadResult";
import { ScoreCard } from "./components/ScoreCard";
import { HardwareDetail } from "./components/HardwareDetail";
import { RadarChart } from "./components/RadarChart";
import { CompareBar } from "./components/CompareBar";
import { NamePrompt } from "./components/NamePrompt";
import { ExportPdf } from "./components/ExportPdf";

const NAME_KEY = "litebench.userName";

function formatDateTime(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}`;
}

export default function App() {
  const [userName, setUserName] = useState<string>(
    () => localStorage.getItem(NAME_KEY) ?? ""
  );
  const [result, setResult] = useState<BenchResult | null>(null);
  const [showFallback, setShowFallback] = useState(false);
  const [runKey, setRunKey] = useState(0);

  const mainstream = REFERENCE_MACHINES.find((m) => m.tier === "mainstream")!;

  const handleName = (name: string) => {
    localStorage.setItem(NAME_KEY, name);
    setUserName(name);
  };

  const handleRerun = async () => {
    const status = await requestAgentRerun();
    if (status === "busy") {
      window.alert("检测正在进行中，请稍候。");
      return;
    }
    if (status === "failed") {
      window.alert("无法启动新一轮检测。请重新双击运行bench程序");
      return;
    }
    setResult(null);
    setRunKey((k) => k + 1);
  };

  const reportTime = result ? new Date(result.timestamp * 1000) : new Date();

  return (
    <div className="app">
      {!userName && <NamePrompt onSubmit={handleName} />}

      <header className="topbar no-print">
        <div className="brand">
          <span className="logo">⚡</span>
          <span>LiteBench</span>
        </div>
        <span className="tagline">轻量电脑评测 · 自动检测 CPU / 内存 / 存储</span>
        {result && (
          <button
            className="btn ghost sm"
            onClick={handleRerun}
            title="在助手窗口仍打开时，从头重新跑分"
          >
            新一轮检测
          </button>
        )}
      </header>

      <main className="content">
        {!result ? (
          <div className="intro">
            <h1>打开即测，几分钟看清你的电脑</h1>
            <p className="sub">
              运行轻量本地助手，自动检测精准硬件型号、批次与生产年份，跑分评估 CPU、内存、存储，并与网络参考机型对比。
            </p>

            <AgentConnect key={runKey} onResult={setResult} />

            <div className="fallback">
              {!showFallback ? (
                <button className="link-btn" onClick={() => setShowFallback(true)}>
                  没有助手？手动导入 result.json 或查看示例 →
                </button>
              ) : (
                <UploadResult onLoaded={setResult} onSample={() => setResult(SAMPLE_RESULT)} />
              )}
            </div>
          </div>
        ) : (
          <div className="dashboard">
            <div className="card report-bar no-print">
              <div>
                <div className="report-title">
                  {userName} - {formatDateTime(reportTime)}
                </div>
                <div className="muted report-sub">LiteBench 评测报告</div>
              </div>
              <ExportPdf userName={userName} reportTime={reportTime} />
            </div>

            <div className="report-body print-area">
              <div className="report-head-print">
                <span className="report-title">
                  {userName} - {formatDateTime(reportTime)}
                </span>
                <span className="muted">LiteBench 评测报告</span>
              </div>

              <ScoreCard result={result} />

              <HardwareDetail detail={result.detail} />

              <div className="charts">
                <div className="card">
                  <h3>分项性能雷达图</h3>
                  <RadarChart
                    scores={result.subscores}
                    reference={{
                      cpu: mainstream.cpu,
                      memory: mainstream.memory,
                      disk: mainstream.disk,
                      label: mainstream.label,
                    }}
                  />
                </div>

                <div className="card">
                  <h3>综合评分 · 网络对比</h3>
                  <CompareBar myTotal={result.totalScore} />
                </div>
              </div>
            </div>
          </div>
        )}
      </main>

      <footer className="footer no-print">
        精准型号 / 批次 / 生产年份由本地助手读取（CIM/WMI、SMBIOS）。内存与磁盘的精确生产周年存于 SPD/SMART，用户态无法直接读取，故年份为估算值。
      </footer>
    </div>
  );
}
