// Queries more than one view reads, keyed by data source so a switch never shows stale rows.
import { useQuery } from "@tanstack/react-query";
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
