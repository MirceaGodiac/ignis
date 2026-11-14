// The atmosphere map: every run placed by ambient oxygen mole fraction and pressure, against the
// crew atmospheres named in the Space Apps 2026 "Flame in Freefall" brief. Points come from the
// conditions table (T9, research/mapPoints.ts): recorded SI values only; runs without both values
// are counted by the caller, never placed at a guessed point.
import { useMemo, useRef, useState, type KeyboardEvent, type MouseEvent } from "react";
import type { MapPoint } from "../research/mapPoints";
import { FamilyMark } from "../shell/icons";

export const PSI = 6.894757; // kPa per psi
/** Reference atmospheres (Flame in Freefall brief, "Spacesuits and oxygen concentration"). */
export const HABITATS = {
  earth: { o2: 0.21, kpa: 101.325, label: "Earth sea level · ISS", sub: "21 % O₂ · 14.7 psi" },
  suit: { o2: 1.0, kpa: 4.3 * PSI, label: "Spacesuit", sub: "100 % O₂ · 4.3 psi" },
  moon: { o2Min: 0.3, kpaMax: 10.2 * PSI, kpaMin: 4.3 * PSI, label: "Moon base and rovers (planned)", sub: "≥ 30 % O₂ · ≤ 10.2 psi" },
};

/** Marker shape per investigation family (FLEX circle, ACME/CFI-G triangle, BASS square, others diamond). */
const SHAPE: Record<string, "circle" | "triangle" | "square"> = { "PSI-69": "circle", "PSI-68": "circle", "PSI-159": "triangle", "PSI-25": "square", "PSI-26": "square" };

/** Distance to the planned Moon-base envelope in axis-normalised units (0 = inside). UI geometry, not a model. */
export function moonDistance(o2: number, kpa: number, kpaSpan: number): number {
  const m = HABITATS.moon;
  const dx = o2 < m.o2Min ? (m.o2Min - o2) / 0.9 : 0;
  const dy = kpa > m.kpaMax ? (kpa - m.kpaMax) / kpaSpan : kpa < m.kpaMin ? (m.kpaMin - kpa) / kpaSpan : 0;
  return Math.hypot(dx, dy);
}

const W = 760;
const H = 430;
const M = { l: 58, r: 58, t: 18, b: 46 };

export function AtmosMap({
  points: placed,
  selected,
  filter,
  onSelect,
}: {
  points: MapPoint[];
  selected: string | null;
  /** Investigation to keep in full colour; the others are dimmed. */
  filter: string | null;
  onSelect: (id: string) => void;
}) {
  const yMax = useMemo(() => {
    const top = Math.max(110, ...placed.map((p) => p.kpa * 1.08));
    return Math.ceil(top / 20) * 20;
  }, [placed]);
  const x = (o2: number) => M.l + ((o2 - 0.1) / 0.93) * (W - M.l - M.r); // domain padded to 103 % so the 100 % marker is not clipped
  const y = (kpa: number) => H - M.b - (kpa / yMax) * (H - M.t - M.b);
  const [hover, setHover] = useState<{ p: MapPoint; cx: number; cy: number } | null>(null);
  const svg = useRef<SVGSVGElement>(null);

  // Runs at the same conditions are fanned out on a small ring, so each stays clickable.
  const marks = useMemo(() => {
    const groups = new Map<string, MapPoint[]>();
    for (const p of placed) {
      const k = `${p.o2.toFixed(3)}|${p.kpa.toFixed(1)}`;
      groups.set(k, [...(groups.get(k) ?? []), p]);
    }
    const out: { p: MapPoint; mx: number; my: number }[] = [];
    for (const g of groups.values()) {
      g.forEach((p, i) => {
        const n = g.length;
        const r = n === 1 ? 0 : 7 + Math.floor(i / 8) * 7;
        const a = (i % 8) * ((2 * Math.PI) / Math.min(n, 8)) - Math.PI / 2;
        out.push({ p, mx: x(p.o2) + r * Math.cos(a), my: y(p.kpa) + r * Math.sin(a) });
      });
    }
    return out;
  }, [placed, yMax]); // eslint-disable-line react-hooks/exhaustive-deps

  const xTicks = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
  const yTicks = Array.from({ length: yMax / 20 + 1 }, (_, i) => i * 20);
  const psiTicks = Array.from({ length: Math.floor(yMax / PSI / 5) + 1 }, (_, i) => i * 5);
  const m = HABITATS.moon;

  return (
    <div style={{ position: "relative" }}>
      <svg ref={svg} className="atmos" viewBox={`0 0 ${W} ${H}`} role="group" aria-label="Runs by ambient oxygen fraction and pressure, with crew atmospheres">
        {yTicks.map((t) => (
          <g key={`y${t}`}>
            <line className="gridline" x1={M.l} x2={W - M.r} y1={y(t)} y2={y(t)} />
            <text x={M.l - 8} y={y(t) + 4} textAnchor="end">
              {t}
            </text>
          </g>
        ))}
        {psiTicks.map((t) => (
          <text key={`p${t}`} x={W - M.r + 8} y={y(t * PSI) + 4}>
            {t}
          </text>
        ))}
        {xTicks.map((t) => (
          <g key={`x${t}`}>
            <line className="gridline" x1={x(t)} x2={x(t)} y1={M.t} y2={H - M.b} />
            <text x={x(t)} y={H - M.b + 16} textAnchor="middle">
              {Math.round(t * 100)}
            </text>
          </g>
        ))}
        <rect className="frame" x={M.l} y={M.t} width={W - M.l - M.r} height={H - M.t - M.b} />
        <text className="axis-title" x={(M.l + W - M.r) / 2} y={H - 6} textAnchor="middle">
          Ambient oxygen, % by volume
        </text>
        <text className="axis-title" transform={`translate(16 ${(M.t + H - M.b) / 2}) rotate(-90)`} textAnchor="middle">
          Pressure, kPa
        </text>
        <text className="axis-title" transform={`translate(${W - 12} ${(M.t + H - M.b) / 2}) rotate(90)`} textAnchor="middle">
          psi
        </text>

        {/* planned Moon base / rover envelope */}
        <rect className="moon" x={x(m.o2Min)} y={y(m.kpaMax)} width={x(1) - x(m.o2Min)} height={y(m.kpaMin) - y(m.kpaMax)} />
        {/* above the box at the right, where no run sits (runs cluster at 15–40 % O₂): inside the box it would
            collide with the spacesuit label once the pressure axis is tall (the real table reaches ~520 kPa) */}
        <text className="moon-label" x={x(1) - 8} y={y(m.kpaMax) - 22} textAnchor="end">
          {m.label}
        </text>
        <text className="moon-label" x={x(1) - 8} y={y(m.kpaMax) - 8} textAnchor="end" style={{ fontWeight: 400 }}>
          {m.sub}
        </text>

        {marks.map(({ p, mx, my }) => {
          const dim = filter && p.investigation !== filter;
          const cls = `run${dim ? " dim" : ""}${selected === p.id ? " sel" : ""}${hover?.p.id === p.id ? " on" : ""}`;
          const label = `${p.id}, ${p.investigation}, oxygen ${(p.o2 * 100).toFixed(0)} %, ${p.kpa.toFixed(1)} kPa${p.approximate ? ", approximate" : ""}`;
          const common = {
            className: cls,
            tabIndex: dim ? -1 : 0,
            role: "button",
            "aria-label": label,
            "aria-pressed": selected === p.id,
            onClick: () => onSelect(p.id),
            onKeyDown: (e: KeyboardEvent) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                onSelect(p.id);
              }
            },
            onMouseEnter: (e: MouseEvent) => setHover({ p, cx: e.clientX, cy: e.clientY }),
            onMouseMove: (e: MouseEvent) => setHover({ p, cx: e.clientX, cy: e.clientY }),
            onMouseLeave: () => setHover(null),
          };
          const s = 5;
          switch (SHAPE[p.investigation]) {
            case "circle":
              return <circle key={p.id} cx={mx} cy={my} r={s} {...common} />;
            case "triangle":
              return <path key={p.id} d={`M${mx} ${my - s - 1}L${mx + s + 1} ${my + s}L${mx - s - 1} ${my + s}Z`} {...common} />;
            case "square":
              return <rect key={p.id} x={mx - s} y={my - s} width={2 * s} height={2 * s} {...common} />;
            default:
              return <path key={p.id} d={`M${mx} ${my - s - 1}L${mx + s + 1} ${my}L${mx} ${my + s + 1}L${mx - s - 1} ${my}Z`} {...common} />;
          }
        })}

        {/* reference atmospheres, drawn last so they stay legible */}
        <g className="ref">
          <circle className="ring" cx={x(HABITATS.earth.o2)} cy={y(HABITATS.earth.kpa)} r={8} />
          <path d={`M${x(HABITATS.earth.o2) - 4} ${y(HABITATS.earth.kpa)}h8M${x(HABITATS.earth.o2)} ${y(HABITATS.earth.kpa) - 4}v8`} stroke="var(--ink)" strokeWidth={1.5} />
          <text className="lbl" x={x(HABITATS.earth.o2) + 13} y={y(HABITATS.earth.kpa) - 32}>
            {HABITATS.earth.label}
          </text>
          <text className="sub" x={x(HABITATS.earth.o2) + 13} y={y(HABITATS.earth.kpa) - 19}>
            {HABITATS.earth.sub}
          </text>
        </g>
        <g className="ref">
          <circle className="ring" cx={x(HABITATS.suit.o2)} cy={y(HABITATS.suit.kpa)} r={8} />
          <path d={`M${x(1) - 4} ${y(HABITATS.suit.kpa)}h8M${x(1)} ${y(HABITATS.suit.kpa) - 4}v8`} stroke="var(--ink)" strokeWidth={1.5} />
          <text className="lbl" x={x(1) - 13} y={y(HABITATS.suit.kpa) - 12} textAnchor="end">
            {HABITATS.suit.label}
          </text>
          <text className="sub" x={x(1) - 13} y={y(HABITATS.suit.kpa) + 2} textAnchor="end">
            {HABITATS.suit.sub}
          </text>
        </g>
      </svg>
      {hover && (
        <div className="tooltip" style={{ left: hover.cx + 14, top: hover.cy + 14 }} role="tooltip">
          <b>{hover.p.id}</b>
          <br />
          <span className="muted">
            {hover.p.investigation} · {hover.p.fuel ?? "fuel not stated"}
          </span>
          <br />
          {hover.p.approximate ? "~" : ""}
          {(hover.p.o2 * 100).toFixed(0)} % O₂ · {hover.p.kpa.toFixed(1)} kPa ({(hover.p.kpa / PSI).toFixed(1)} psi)
        </div>
      )}
    </div>
  );
}

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
