// The family filter of the run index: one segment per family, with its mark and count.
import { FamilyMark } from "../shell/icons";

export function FamilyLegend({ counts, active, onPick }: { counts: Record<string, number>; active: string | null; onPick: (f: string | null) => void }) {
  return (
    <div className="seg" role="group" aria-label="Filter by family" style={{ flexWrap: "wrap" }}>
      <button aria-pressed={active === null} onClick={() => onPick(null)}>
        All <span className="count">{Object.values(counts).reduce((a, b) => a + b, 0)}</span>
      </button>
      {Object.entries(counts).map(([f, n]) => (
        <button key={f} aria-pressed={active === f} onClick={() => onPick(active === f ? null : f)}>
          <FamilyMark family={f} size={9} /> {f.replace("_", " ").replace("acme ", "ACME ").replace("flex", "FLEX").replace("bass", "BASS")} <span className="count">{n}</span>
        </button>
      ))}
    </div>
  );
}
