import ReactECharts from "echarts-for-react";
import { REFERENCE_MACHINES } from "../data/baseline";

interface Props {
  myTotal: number;
}

export function CompareBar({ myTotal }: Props) {
  const entries = [
    ...REFERENCE_MACHINES.map((m) => ({ label: m.label, value: m.total, me: false })),
    { label: "★ 本机", value: myTotal, me: true },
  ].sort((a, b) => a.value - b.value);

  const option = {
    grid: { left: 140, right: 40, top: 20, bottom: 20 },
    tooltip: { trigger: "axis", axisPointer: { type: "shadow" } },
    xAxis: {
      type: "value",
      axisLabel: { color: "#94a3b8" },
      splitLine: { lineStyle: { color: "rgba(148,163,184,0.15)" } },
    },
    yAxis: {
      type: "category",
      data: entries.map((e) => e.label),
      axisLabel: { color: "#cbd5e1" },
    },
    series: [
      {
        type: "bar",
        data: entries.map((e) => ({
          value: e.value,
          itemStyle: { color: e.me ? "#6366f1" : "#475569", borderRadius: [0, 4, 4, 0] },
        })),
        label: { show: true, position: "right", color: "#e2e8f0", formatter: "{c}" },
        barWidth: "55%",
      },
    ],
  };

  return <ReactECharts option={option} style={{ height: 280 }} />;
}
