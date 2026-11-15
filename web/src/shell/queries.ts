// Queries more than one view reads, keyed by data source so a switch never shows stale rows.
import { useEffect } from "react";
import { infiniteQueryOptions, queryOptions, useQuery, useQueryClient } from "@tanstack/react-query";
import type { Mode, ResearchClient } from "../research/client";
import { useData } from "./data";

export function useEpisodes() {
  const { client, researchMode, replay, ready } = useData();
  return useQuery({ queryKey: [researchMode, replay, "episodes"], queryFn: () => client.listEpisodes(), retry: false, enabled: ready });
}

export function useReleases() {
  const { client, researchMode, replay, caps } = useData();
  return useQuery({
    queryKey: [researchMode, replay, "models"],
    queryFn: () => client.listModels(),
    retry: false,
    enabled: caps.data?.models.available !== false,
  });
}

/**
 * Starter searches. Chosen on the staging corpus (15 Nov 2026) because their first hits are
 * passages of substance, not reference lists; the first one loads when Sources opens empty.
 */
export const STARTERS = ["oxygen concentration flammability limit", "soot formation in microgravity flames", "flame size of gas jet flames in microgravity"];

/** Passages per page of results; larger than the server default so a broad query shows more at once. */
const SEARCH_PAGE = 25;

/** A search over the pinned corpus snapshot. The snapshot does not move under a session, so a result
 * stays fresh for half an hour, and the first starter can be fetched before Sources is opened. */
export function searchOptions(client: ResearchClient, researchMode: Mode, replay: number, query: string) {
  return infiniteQueryOptions({
    queryKey: [researchMode, replay, "search", query],
    initialPageParam: null as string | null,
    queryFn: ({ pageParam }) => client.search({ schema_version: "evidence.v1", query, mode: "hybrid", page_size: SEARCH_PAGE, ...(pageParam ? { cursor: pageParam } : {}) }),
    getNextPageParam: (last) => last.next_cursor ?? undefined,
    staleTime: 30 * 60_000,
    gcTime: 60 * 60_000,
    retry: false,
  });
}

/** How a source's original is reached. A signed URL is short-lived: never served past its expiry. */
export function accessOptions(client: ResearchClient, researchMode: Mode, sourceId: string) {
  return queryOptions({
    queryKey: [researchMode, "access", sourceId],
    queryFn: () => client.sourceAccess(sourceId),
    retry: false,
    staleTime: (q) => {
      const a = q.state.data;
      return a?.expires_at ? Math.max(0, Date.parse(a.expires_at) - Date.now() - 30_000) : 5 * 60_000;
    },
  });
}

/** Runs `fn` once the browser is idle (or shortly, where idle callbacks do not exist). */
export function whenIdle(fn: () => void, timeout = 2000): () => void {
  if (typeof requestIdleCallback === "function") {
    const id = requestIdleCallback(fn, { timeout });
    return () => cancelIdleCallback(id);
  }
  const id = setTimeout(fn, 200);
  return () => clearTimeout(id);
}

/** Fetches the first starter search once the app is idle, so Sources opens on its results. */
export function usePrefetchStarterSearch() {
  const qc = useQueryClient();
  const { client, researchMode, replay, caps } = useData();
  const ready = !!caps.data && caps.data.knowledge.available !== false;
  useEffect(() => {
    if (!ready) return;
    return whenIdle(() => void qc.prefetchInfiniteQuery(searchOptions(client, researchMode, replay, STARTERS[0]!)));
  }, [ready, qc, client, researchMode, replay]);
}
