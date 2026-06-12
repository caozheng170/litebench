import { useState, type RefObject } from "react";
import html2canvas from "html2canvas";
import { jsPDF } from "jspdf";

interface Props {
  targetRef: RefObject<HTMLElement | null>;
  userName: string;
}

/** "张三-2026-06-12_08-23" — filename-safe (no colons). */
export function reportBaseName(userName: string, d: Date = new Date()): string {
  const p = (n: number) => String(n).padStart(2, "0");
  const date = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
  const time = `${p(d.getHours())}-${p(d.getMinutes())}`;
  return `${userName}-${date}_${time}`;
}

async function exportToPdf(el: HTMLElement, baseName: string) {
  const canvas = await html2canvas(el, {
    scale: 2,
    backgroundColor: "#0b1020",
    useCORS: true,
    logging: false,
  });

  const pdf = new jsPDF({ orientation: "portrait", unit: "mm", format: "a4" });
  const pageW = 210;
  const pageH = 297;
  const imgW = pageW;
  const imgH = (canvas.height * imgW) / canvas.width;
  const img = canvas.toDataURL("image/jpeg", 0.92);

  // Tile the tall capture across as many A4 pages as needed.
  let offset = 0;
  pdf.addImage(img, "JPEG", 0, 0, imgW, imgH);
  let remaining = imgH - pageH;
  while (remaining > 0) {
    offset -= pageH;
    pdf.addPage();
    pdf.addImage(img, "JPEG", 0, offset, imgW, imgH);
    remaining -= pageH;
  }

  pdf.setProperties({ title: baseName });
  pdf.save(`${baseName}.pdf`);
}

export function ExportPdf({ targetRef, userName }: Props) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState(false);

  const onClick = async () => {
    const el = targetRef.current;
    if (!el || busy) return;
    setBusy(true);
    setError(false);
    try {
      await exportToPdf(el, reportBaseName(userName));
    } catch {
      setError(true);
    } finally {
      setBusy(false);
    }
  };

  return (
    <button className="btn primary" onClick={onClick} disabled={busy}>
      {busy ? "导出中…" : error ? "导出失败，重试" : "导出 PDF"}
    </button>
  );
}
