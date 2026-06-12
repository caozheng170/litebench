import { useState } from "react";

interface Props {
  onSubmit: (name: string) => void;
}

/** Blocking modal shown on first open: the report needs the user's name. */
export function NamePrompt({ onSubmit }: Props) {
  const [value, setValue] = useState("");
  const trimmed = value.trim();

  const submit = () => {
    if (trimmed) onSubmit(trimmed);
  };

  return (
    <div className="modal-overlay">
      <div className="modal-card">
        <div className="modal-icon">👤</div>
        <h2>请输入您的姓名</h2>
        <p className="muted">姓名将用于评测报告的标题与导出的 PDF 文件名。</p>
        <input
          className="name-input"
          type="text"
          autoFocus
          maxLength={30}
          placeholder="例如：张三"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
        />
        <button className="btn primary wide" disabled={!trimmed} onClick={submit}>
          开始评测
        </button>
      </div>
    </div>
  );
}
