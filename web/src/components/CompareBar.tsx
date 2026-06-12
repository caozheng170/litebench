import { useEffect, useState } from "react";
import ReactECharts from "echarts-for-react";
import { REFERENCE_MACHINES } from "../data/baseline";

interface Props {
  myTotal: number;
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

export function CompareBar({ myTotal }: Props) {
  const printing = usePrintMode();
  const axis = printing ? "#475569" : "#94a3b8";
  const label = printing ? "#334155" : "#cbd5e1";
  const valueLabel = printing ? "#1e293b" : "#e2e8f0";
  const gridLine = printing ? "rgba(71,85,105,0.2)" : "rgba(148,163,184,0.15)";
  const entries = [
    ...REFERENCE_MACHINES.map((m) => ({ label: m.label, value: m.total, me: false })),
    { label: "★ 本机", value: myTotal, me: true },
  ].sort((a, b) => a.value - b.value);

  const option = {
    grid: { left: 140, right: 40, top: 20, bottom: 20 },
    tooltip: { trigger: "axis", axisPointer: { type: "shadow" } },
    xAxis: {
      type: "value",
      axisLabel: { color: axis },
      splitLine: { lineStyle: { color: gridLine } },
    },
    yAxis: {
      type: "category",
      data: entries.map((e) => e.label),
      axisLabel: { color: label },
    },
    series: [
      {
        type: "bar",
        data: entries.map((e) => ({
          value: e.value,
          itemStyle: { color: e.me ? "#6366f1" : printing ? "#94a3b8" : "#475569", borderRadius: [0, 4, 4, 0] },
        })),
        label: { show: true, position: "right", color: valueLabel, formatter: "{c}" },
        barWidth: "55%",
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 280 }} />;
}
