import { useEffect, useRef, useState } from "react";
import { RefusalError } from "./api";

export interface Latest<T> {
  /** Result only if it belongs to the *current* inputs; stale results are never exposed. */
  data: T | undefined;
  error: Error | undefined;
  loading: boolean;
  version: number;
}

/**
 * Debounced, cancellable request bound to an input key. Every input change bumps a version and
 * aborts the in-flight request; a response is accepted only if its version is still current.
 */
export function useLatest<T>(key: string | null, run: (signal: AbortSignal) => Promise<T>, delayMs = 250): Latest<T> {
  const version = useRef(0);
  const [state, setState] = useState<{ key: string | null; v: number; data?: T; error?: Error; loading: boolean }>({ key: null, v: 0, loading: false });
  useEffect(() => {
    const v = ++version.current;
    if (key === null) {
      setState({ key, v, loading: false });
      return;
    }
    setState({ key, v, loading: true });
    const ctl = new AbortController();
    const h = setTimeout(() => {
      run(ctl.signal).then(
        (data) => v === version.current && setState({ key, v, data, loading: false }),
        (error: Error) => {
          if (error.name === "AbortError" || v !== version.current) return;
          setState({ key, v, error, loading: false });
        },
      );
    }, delayMs);
    return () => {
      clearTimeout(h);
      ctl.abort();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);
  const current = state.key === key && state.v === version.current;
  return { data: current ? state.data : undefined, error: current ? state.error : undefined, loading: !current || state.loading, version: version.current };
}

export const sleep = (ms: number, signal: AbortSignal) =>
  new Promise<void>((resolve, reject) => {
    const h = setTimeout(resolve, ms);
    signal.addEventListener("abort", () => {
      clearTimeout(h);
      reject(new DOMException("aborted", "AbortError"));
    });
  });

export const asRefusal = (e: unknown) => (e instanceof RefusalError ? e.refusal : e ? { kind: "failed" as const, code: "client", reason: String(e) } : null);
