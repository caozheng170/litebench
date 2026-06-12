import { useEffect, useState } from "react";
import ReactECharts from "echarts-for-react";
import type { SubScores } from "../types";

interface Props {
  scores: SubScores;
  reference?: { cpu: number; memory: number; disk: number; label: string };
}

function usePrintMode(): boolean {
  const [printing, setPrinting] = useState(false);
  useEffect(() => {
    const on = () => setPrinting(true);
    const off = () => setPrinting(false);
    window.addEventListener("beforeprint", on);
    window.addEventListener("afterprint", off);
    return () => {
      window.removeEventListener("beforeprint", on);
      window.removeEventListener("afterprint", off);
    };
  }, []);
  return printing;
}

export function RadarChart({ scores, reference }: Props) {
  const printing = usePrintMode();
  const ink = printing ? "#475569" : "#cbd5e1";
  const split = printing ? "rgba(71,85,105,0.25)" : "rgba(148,163,184,0.2)";
  const max = Math.max(
    scores.cpu,
    scores.memory,
    scores.disk,
    reference?.cpu ?? 0,
    reference?.memory ?? 0,
    reference?.disk ?? 0,
    1000
  );

  const series = [
    {
      value: [scores.cpu, scores.memory, scores.disk],
      name: "本机",
      areaStyle: { opacity: 0.25 },
      lineStyle: { width: 2 },
    },
  ];
  if (reference) {
    series.push({
      value: [reference.cpu, reference.memory, reference.disk],
      name: reference.label,
      areaStyle: { opacity: 0.1 },
      lineStyle: { width: 2 },
    });
  }

  const option = {
    color: ["#6366f1", "#94a3b8"],
    tooltip: {},
    legend: { bottom: 0, textStyle: { color: ink } },
    radar: {
      indicator: [
        { name: "CPU", max },
        { name: "内存", max },
        { name: "存储", max },
      ],
      axisName: { color: ink },
      splitLine: { lineStyle: { color: split } },
      splitArea: { areaStyle: { color: ["rgba(99,102,241,0.03)", "transparent"] } },
      axisLine: { lineStyle: { color: split } },
    },
    series: [{ type: "radar", data: series }],
  };

  return <ReactECharts option={option} style={{ height: 320 }} />;
}
