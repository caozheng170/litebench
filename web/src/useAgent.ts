import { useCallback, useEffect, useRef, useState } from "react";
import { AGENT_BASE_URL } from "./data/baseline";
import { isBenchResult, type AgentProgress, type BenchResult } from "./types";

export type AgentConnState = "searching" | "running" | "done" | "offline" | "error";

interface AgentHook {
  state: AgentConnState;
  progress: AgentProgress | null;
  result: BenchResult | null;
  error: string | null;
  retry: () => void;
}

const POLL_MS = 1200;
/** After this many consecutive failures, assume the agent process was closed. */
const OFFLINE_AFTER_FAILURES = 3;
const POLL_MS_OFFLINE = 2500;

/** Ask a running agent to start a fresh benchmark (CMD window still open). */
export async function requestAgentRerun(
  baseUrl: string = AGENT_BASE_URL
): Promise<"ok" | "busy" | "failed"> {
  try {
    const res = await fetch(`${baseUrl}/rerun`, { method: "POST", cache: "no-store" });
    if (res.status === 202) return "ok";
    if (res.status === 409) return "busy";
    return "failed";
  } catch {
    return "failed";
  }
}

/**
 * Poll the locally-running native agent. Switches to `offline` when the helper
 * is not reachable (e.g. the user closed the CMD window) instead of waiting forever.
 */
export function useAgent(baseUrl: string = AGENT_BASE_URL, enabled: boolean = true): AgentHook {
  const [state, setState] = useState<AgentConnState>("searching");
  const [progress, setProgress] = useState<AgentProgress | null>(null);
  const [result, setResult] = useState<BenchResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [nonce, setNonce] = useState(0);
  const stopped = useRef(false);
  const failStreak = useRef(0);

  const retry = useCallback(() => {
    failStreak.current = 0;
    setState("searching");
    setProgress(null);
    setResult(null);
    setError(null);
    setNonce((n) => n + 1);
  }, []);

  useEffect(() => {
    if (!enabled) return;
    stopped.current = false;
    failStreak.current = 0;
    let timer: number | undefined;

    const tick = async () => {
      try {
        const res = await fetch(`${baseUrl}/status`, { cache: "no-store" });
        if (!res.ok && res.status !== 202) throw new Error(`status ${res.status}`);
        const p = (await res.json()) as AgentProgress;
        if (stopped.current) return;

        failStreak.current = 0;
        setProgress(p);

        if (p.done) {
          const rr = await fetch(`${baseUrl}/result`, { cache: "no-store" });
          if (rr.ok) {
            const data = await rr.json();
            if (isBenchResult(data)) {
              if (stopped.current) return;
              setResult(data);
              setState("done");
              return;
            }
          }
          setState("error");
          setError("助手返回的结果格式不正确。");
          return;
        }
        setState("running");
      } catch {
        if (stopped.current) return;
        failStreak.current += 1;
        if (failStreak.current >= OFFLINE_AFTER_FAILURES) {
          setState("offline");
        } else {
          setState("searching");
        }
      }

      if (!stopped.current) {
        const delay =
          failStreak.current >= OFFLINE_AFTER_FAILURES ? POLL_MS_OFFLINE : POLL_MS;
        timer = window.setTimeout(tick, delay);
      }
    };

    tick();
    return () => {
      stopped.current = true;
      if (timer) window.clearTimeout(timer);
    };
  }, [baseUrl, enabled, nonce]);

  return { state, progress, result, error, retry };
}
