import { useCallback, useRef, useState } from "react";
import { isBenchResult, type BenchResult } from "../types";

interface Props {
  onLoaded: (result: BenchResult) => void;
  onSample: () => void;
}

export function UploadResult({ onLoaded, onSample }: Props) {
  const [dragOver, setDragOver] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleFile = useCallback(
    (file: File) => {
      setError(null);
      const reader = new FileReader();
      reader.onload = () => {
        try {
          const parsed = JSON.parse(String(reader.result));
          if (!isBenchResult(parsed)) {
            setError("文件格式不正确：不是有效的 bench-result JSON。");
            return;
          }
          onLoaded(parsed);
        } catch {
          setError("无法解析 JSON 文件。");
        }
      };
      reader.onerror = () => setError("读取文件失败。");
      reader.readAsText(file);
    },
    [onLoaded]
  );

  return (
    <div
      className={`dropzone${dragOver ? " over" : ""}`}
      onDragOver={(e) => {
        e.preventDefault();
        setDragOver(true);
      }}
      onDragLeave={() => setDragOver(false)}
      onDrop={(e) => {
        e.preventDefault();
        setDragOver(false);
        const f = e.dataTransfer.files?.[0];
        if (f) handleFile(f);
      }}
    >
      <div className="dz-icon">📊</div>
      <h2>拖入 result.json</h2>
      <p>运行本地跑分助手后会生成 result.json，把它拖到这里查看评分与对比图表。</p>

      <div className="dz-actions">
        <button className="btn primary" onClick={() => inputRef.current?.click()}>
          选择文件
        </button>
        <button className="btn ghost" onClick={onSample}>
          加载示例数据
        </button>
      </div>

      <input
        ref={inputRef}
        type="file"
        accept="application/json,.json"
        style={{ display: "none" }}
        onChange={(e) => {
          const f = e.target.files?.[0];
          if (f) handleFile(f);
        }}
      />

      {error && <div className="dz-error">{error}</div>}
    </div>
  );
}
