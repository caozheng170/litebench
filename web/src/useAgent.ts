import { useCallback, useEffect, useRef, useState } from "react";
import { AGENT_BASE_URL } from "./data/baseline";
import { isBenchResult, type AgentProgress, type BenchResult } from "./types";

export type AgentConnState = "searching" | "running" | "done" | "error";

interface AgentHook {
  state: AgentConnState;
  progress: AgentProgress | null;
  result: BenchResult | null;
  error: string | null;
  /** Manually re-attempt the connection from scratch. */
  retry: () => void;
}

const POLL_MS = 1200;

/**
 * Continuously poll the locally-running native agent. While the agent is not
 * running this stays in "searching" (so the page auto-connects the moment the
 * user launches the helper). Once the agent reports `done`, the full result is
 * fetched and exposed.
 */
export function useAgent(baseUrl: string = AGENT_BASE_URL, enabled: boolean = true): AgentHook {
  const [state, setState] = useState<AgentConnState>("searching");
  const [progress, setProgress] = useState<AgentProgress | null>(null);
  const [result, setResult] = useState<BenchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [nonce, setNonce] = useState(0);
  const stopped = useRef(false);

  const retry = useCallback(() => {
    setState("searching");
    setProgress(null);
    setResult(null);
    setError(null);
    setNonce((n) => n + 1);
  }, []);

  useEffect(() => {
    if (!enabled) return;
    stopped.current = false;
    let timer: number | undefined;

    const tick = async () => {
      try {
        const res = await fetch(`${baseUrl}/status`, { cache: "no-store" });
        if (!res.ok && res.status !== 202) throw new Error(`status ${res.status}`);
        const p = (await res.json()) as AgentProgress;
        if (stopped.current) return;
        setProgress(p);

        if (p.done) {
          const rr = await fetch(`${baseUrl}/result`, { cache: "no-store" });
          if (rr.ok) {
            const data = await rr.json();
            if (isBenchResult(data)) {
              if (stopped.current) return;
              setResult(data);
              setState("done");
              return; // stop polling
            }
          }
          setState("error");
          setError("助手返回的结果格式不正确。");
          return;
        }
        setState("running");
      } catch {
        // Agent not reachable yet — keep searching so it auto-connects later.
        if (!stopped.current) setState("searching");
      }
      if (!stopped.current) timer = window.setTimeout(tick, POLL_MS);
    };

    tick();
    return () => {
      stopped.current = true;
      if (timer) window.clearTimeout(timer);
    };
  }, [baseUrl, enabled, nonce]);

  return { state, progress, result, error, retry };
}
