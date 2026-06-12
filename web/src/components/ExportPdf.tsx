import { useCallback, useState } from "react";

interface Props {
  userName: string;
  reportTime?: Date;
}

/** "张三-2026-06-12_08-23" — used as default PDF filename in the print dialog. */
export function reportBaseName(userName: string, d: Date = new Date()): string {
  const p = (n: number) => String(n).padStart(2, "0");
  const date = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  const time = `${p(d.getHours())}-${p(d.getMinutes())}`;
  return `${userName}-${date}_${time}`;
}

/** Trigger the browser print dialog (Save as PDF) — same path as Ctrl+P, layout is reliable. */
export function ExportPdf({ userName, reportTime = new Date() }: Props) {
  const [busy, setBusy] = useState(false);

  const onClick = useCallback(() => {
    if (busy) return;
    setBusy(true);

    const prevTitle = document.title;
    document.title = reportBaseName(userName, reportTime);

    window.dispatchEvent(new Event("beforeprint"));
    window.dispatchEvent(new Event("resize"));

    const restore = () => {
      document.title = prevTitle;
      setBusy(false);
      window.dispatchEvent(new Event("afterprint"));
      window.removeEventListener("afterprint", restore);
    };
    window.addEventListener("afterprint", restore);

    // Two frames so ECharts can repaint with print colors before preview opens.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        window.print();
      });
    });
  }, [busy, userName, reportTime]);

  return (
    <button
      className="btn primary no-print"
      onClick={onClick}
      disabled={busy}
      title="打开打印窗口，目标打印机选「另存为 PDF」"
    >
      {busy ? "准备打印…" : "导出 PDF"}
    </button>
  );
}
