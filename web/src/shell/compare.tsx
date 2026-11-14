// The run selection shared by the map, the run index, the side panel and Compare. Cleared when the
// data source changes, so a selection never mixes sources.
import { createContext, useContext, useEffect, useState, type ReactNode } from "react";
import { useData } from "./data";

interface Sel {
  ids: string[];
  toggle: (id: string) => void;
  has: (id: string) => boolean;
  clear: () => void;
  set: (ids: string[]) => void;
}
const Ctx = createContext<Sel | null>(null);
export const MAX_COMPARE = 8;

export function CompareProvider({ children }: { children: ReactNode }) {
  const { source } = useData();
  const [ids, setIds] = useState<string[]>([]);
  useEffect(() => setIds([]), [source]);
  const toggle = (id: string) => setIds((s) => (s.includes(id) ? s.filter((x) => x !== id) : s.length >= MAX_COMPARE ? s : [...s, id]));
  return <Ctx.Provider value={{ ids, toggle, has: (id) => ids.includes(id), clear: () => setIds([]), set: setIds }}>{children}</Ctx.Provider>;
}

export function useCompare(): Sel {
  const v = useContext(Ctx);
  if (!v) throw new Error("useCompare outside CompareProvider");
  return v;
}
