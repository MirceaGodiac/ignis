// The atmosphere map: test points (runs grouped by research/testPoints) by ambient oxygen and pressure,
// against the crew atmospheres. Zoom with the wheel or the buttons, drag to pan, double-click to zoom
// in; a point's size counts its runs, its colour names the investigation and its shape the hardware.
// Only recorded values are placed; the caller counts what cannot be.
import { useEffect, useMemo, useRef, useState, type PointerEvent as RPointerEvent } from "react";
import { invColor, invName, invShape, type Shape } from "../data/investigations";
import type { Habitat, HabitatId } from "../research/habitats";
import { HABITAT_LIST, PSI_PA } from "../research/habitats";
import type { TestPoint } from "../research/testPoints";

const M = { l: 56, r: 56, t: 16, b: 44 };
const PSI_KPA = PSI_PA / 1000;

export interface Domain {
  x: [number, number]; // O2 mole fraction
  y: [number, number]; // kPa
}
/** The habitat region: O2 10–103 %, 0–150 kPa (the crew atmospheres and most tests). */
export const HOME: Domain = { x: [0.1, 1.03], y: [0, 150] };

function niceTicks(lo: number, hi: number, n = 6): number[] {
  const span = hi - lo;
  const raw = span / n;
  const mag = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => span / s <= n) ?? 10 * mag;
  const out: number[] = [];
  for (let v = Math.ceil(lo / step) * step; v <= hi + 1e-9; v += step) out.push(+v.toFixed(10));
  return out;
}

function mark(shape: Shape, x: number, y: number, r: number) {
  switch (shape) {
    case "circle":
      return <circle cx={x} cy={y} r={r} />;
    case "square":
      return <rect x={x - r * 0.88} y={y - r * 0.88} width={r * 1.76} height={r * 1.76} rx={1.5} />;
    case "triangle":
      return <path d={`M${x} ${y - r * 1.12} L${x + r * 1.05} ${y + r * 0.72} L${x - r * 1.05} ${y + r * 0.72} Z`} />;
    default:
      return <path d={`M${x} ${y - r * 1.15} L${x + r * 1.15} ${y} L${x} ${y + r * 1.15} L${x - r * 1.15} ${y} Z`} />;
  }
}

export function ConditionsMap({
  points,
  hidden,
  habitats,
  selected,
  onSelect,
  domain,
  setDomain,
  maxKpa,
}: {
  points: TestPoint[];
  /** Investigations switched off. */
  hidden: Set<string>;
  habitats: Set<HabitatId>;
  selected: string | null;
  onSelect: (key: string | null) => void;
  domain: Domain;
  setDomain: (d: Domain | ((d: Domain) => Domain)) => void;
  maxKpa: number;
}) {
  const svg = useRef<SVGSVGElement>(null);
  // Drawn at the container's real pixel size, so labels keep their size when a side panel narrows it.
  const wrap = useRef<HTMLDivElement>(null);
  const [W, setW] = useState(820);
  useEffect(() => {
    const ro = new ResizeObserver(([e]) => setW(Math.max(380, Math.round(e.contentRect.width))));
    ro.observe(wrap.current!);
    return () => ro.disconnect();
  }, []);
  const H = Math.round(Math.min(560, Math.max(320, W * 0.58)));
  const PW = W - M.l - M.r;
  const PH = H - M.t - M.b;
  const [hover, setHover] = useState<{ p: TestPoint; x: number; y: number } | null>(null);
  const drag = useRef<{ px: number; py: number; d: Domain; moved: boolean } | null>(null);

  const { x: [x0, x1], y: [y0, y1] } = domain;
  const sx = (o2: number) => M.l + ((o2 - x0) / (x1 - x0)) * PW;
  const sy = (k: number) => M.t + PH - ((k - y0) / (y1 - y0)) * PH;
  const shown = useMemo(() => points.filter((p) => !hidden.has(p.investigation)), [points, hidden]);
  // Big points first, so small ones stay on top and clickable.
  const drawn = useMemo(() => [...shown].sort((a, b) => b.runs.length - a.runs.length), [shown]);

  const limits = (d: Domain): Domain => {
    const clampSpan = (lo: number, hi: number, min: number, max: number, floor: number, ceil: number): [number, number] => {
      let span = Math.min(Math.max(hi - lo, min), max);
      let a = (lo + hi) / 2 - span / 2;
      a = Math.max(floor, Math.min(a, ceil - span));
      return [a, a + span];
    };
    return { x: clampSpan(d.x[0], d.x[1], 0.01, 1.1, 0, 1.1), y: clampSpan(d.y[0], d.y[1], 2, maxKpa * 1.1 + 10, 0, maxKpa * 1.1 + 10) };
  };
  const toData = (clientX: number, clientY: number) => {
    const ctm = svg.current!.getScreenCTM()!.inverse();
    const pt = new DOMPoint(clientX, clientY).matrixTransform(ctm);
    return { o2: x0 + ((pt.x - M.l) / PW) * (x1 - x0), k: y0 + ((M.t + PH - pt.y) / PH) * (y1 - y0), inPlot: pt.x >= M.l && pt.x <= M.l + PW && pt.y >= M.t && pt.y <= M.t + PH };
  };
  const zoomAt = (o2: number, k: number, f: number) =>
    setDomain((d) => limits({ x: [o2 + (d.x[0] - o2) * f, o2 + (d.x[1] - o2) * f], y: [k + (d.y[0] - k) * f, k + (d.y[1] - k) * f] }));

  // A wheel over the plot zooms around the cursor; the page does not scroll under it.
  useEffect(() => {
    const el = svg.current!;
    const on = (e: WheelEvent) => {
      const p = toData(e.clientX, e.clientY);
      if (!p.inPlot) return;
      e.preventDefault();
      zoomAt(p.o2, p.k, Math.exp(e.deltaY * 0.0015));
    };
    el.addEventListener("wheel", on, { passive: false });
    return () => el.removeEventListener("wheel", on);
  });

  const down = (e: RPointerEvent<SVGSVGElement>) => {
    if (e.button !== 0) return;
    drag.current = { px: e.clientX, py: e.clientY, d: domain, moved: false };
    (e.target as Element).setPointerCapture?.(e.pointerId);
  };
  const move = (e: RPointerEvent<SVGSVGElement>) => {
    const g = drag.current;
    if (!g) return;
    const ctm = svg.current!.getScreenCTM()!;
    const dx = (e.clientX - g.px) / ctm.a;
    const dy = (e.clientY - g.py) / ctm.d;
    if (Math.abs(dx) + Math.abs(dy) > 3) g.moved = true;
    if (!g.moved) return;
    setHover(null);
    const ox = (dx / PW) * (g.d.x[1] - g.d.x[0]);
    const oy = (dy / PH) * (g.d.y[1] - g.d.y[0]);
    setDomain(limits({ x: [g.d.x[0] - ox, g.d.x[1] - ox], y: [g.d.y[0] + oy, g.d.y[1] + oy] }));
  };
  const up = () => {
    const g = drag.current;
    drag.current = null;
    if (g && !g.moved) onSelect(null);
  };

  const xt = niceTicks(x0 * 100, x1 * 100);
  const yt = niceTicks(y0, y1);
  const pt = niceTicks(y0 / PSI_KPA, y1 / PSI_KPA);
  const r = (n: number) => Math.min(13, 3.2 + 2.1 * Math.sqrt(n));
  const out = shown.filter((p) => p.o2 < x0 || p.o2 > x1 || p.kpa < y0 || p.kpa > y1).length;
  const hab = (id: HabitatId): Habitat => HABITAT_LIST.find((h) => h.id === id)!;

  return (
    <div className="cmap" ref={wrap}>
      <svg
        ref={svg}
        className="cmap-svg"
        width={W}
        height={H}
        viewBox={`0 0 ${W} ${H}`}
        role="img"
        aria-label="Test points by ambient oxygen and pressure, with the crew atmospheres"
        onPointerDown={down}
        onPointerMove={move}
        onPointerUp={up}
        onPointerLeave={() => setHover(null)}
        onDoubleClick={(e) => {
          if ((e.target as Element).closest(".cmap-pt")) return;
          const p = toData(e.clientX, e.clientY);
          if (p.inPlot) zoomAt(p.o2, p.k, 0.5);
        }}
      >
        <defs>
          <clipPath id="cmap-clip">
            <rect x={M.l} y={M.t} width={PW} height={PH} />
          </clipPath>
        </defs>
        <rect className="cmap-plot" x={M.l} y={M.t} width={PW} height={PH} />
        {xt.map((v) => (
          <g key={`x${v}`}>
            <line className="cmap-grid" x1={sx(v / 100)} x2={sx(v / 100)} y1={M.t} y2={M.t + PH} />
            <text className="cmap-tick" x={sx(v / 100)} y={M.t + PH + 18} textAnchor="middle">
              {v}
            </text>
          </g>
        ))}
        {yt.map((v) => (
          <g key={`y${v}`}>
            <line className="cmap-grid" x1={M.l} x2={M.l + PW} y1={sy(v)} y2={sy(v)} />
            <text className="cmap-tick" x={M.l - 8} y={sy(v) + 4} textAnchor="end">
              {v}
            </text>
          </g>
        ))}
        {pt.map((v) => (
          <text key={`p${v}`} className="cmap-tick" x={M.l + PW + 8} y={sy(v * PSI_KPA) + 4}>
            {v}
          </text>
        ))}
        <text className="cmap-axis" x={M.l + PW / 2} y={H - 6} textAnchor="middle">
          Ambient oxygen, % by volume
        </text>
        <text className="cmap-axis" transform={`translate(14 ${M.t + PH / 2}) rotate(-90)`} textAnchor="middle">
          Pressure, kPa
        </text>
        <text className="cmap-axis" transform={`translate(${W - 10} ${M.t + PH / 2}) rotate(90)`} textAnchor="middle">
          psi
        </text>

        <g clipPath="url(#cmap-clip)">
          {habitats.has("moon") &&
            (() => {
              const h = hab("moon");
              const x = sx(h.o2[0]);
              const y = sy(h.pa[1] / 1000);
              return (
                <g className="cmap-hab">
                  <rect className="cmap-region" x={x} y={y} width={Math.max(0, sx(h.o2[1]) - x)} height={Math.max(0, sy(h.pa[0] / 1000) - y)} />
                  <text className="cmap-hab-label" x={x + 8} y={y + 16}>
                    Moon base and rovers
                  </text>
                </g>
              );
            })()}
          {drawn.map((p) => {
            const on = p.key === selected;
            return (
              <g
                key={p.key}
                className={`cmap-pt${on ? " sel" : ""}`}
                style={{ fill: invColor(p.investigation) }}
                onPointerEnter={(e) => !drag.current && setHover({ p, x: e.clientX, y: e.clientY })}
                onPointerLeave={() => setHover(null)}
                onPointerUp={(e) => {
                  if (drag.current && !drag.current.moved) {
                    e.stopPropagation();
                    drag.current = null;
                    onSelect(on ? null : p.key);
                  }
                }}
              >
                {mark(invShape(p.investigation), sx(p.o2), sy(p.kpa), r(p.runs.length))}
              </g>
            );
          })}
          {(["iss", "suit"] as HabitatId[])
            .filter((id) => habitats.has(id))
            .map((id) => {
              const h = hab(id);
              const x = sx(h.o2[0]);
              const y = sy(h.pa[0] / 1000);
              const left = x > M.l + PW - 140;
              return (
                <g key={id} className="cmap-hab">
                  <circle className="cmap-ref" cx={x} cy={y} r={7} />
                  <path className="cmap-ref-x" d={`M${x - 4} ${y}h8M${x} ${y - 4}v8`} />
                  <text className="cmap-hab-label" x={left ? x - 12 : x + 12} y={y - 10} textAnchor={left ? "end" : "start"}>
                    {id === "iss" ? "ISS · Earth" : "Spacesuit"}
                  </text>
                </g>
              );
            })}
        </g>
      </svg>
      <div className="cmap-tools" role="group" aria-label="Zoom">
        <button className="btn btn-icon btn-s" aria-label="Zoom in" title="Zoom in" onClick={() => zoomAt((x0 + x1) / 2, (y0 + y1) / 2, 0.6)}>
          +
        </button>
        <button className="btn btn-icon btn-s" aria-label="Zoom out" title="Zoom out" onClick={() => zoomAt((x0 + x1) / 2, (y0 + y1) / 2, 1 / 0.6)}>
          −
        </button>
        <button className="btn btn-s" onClick={() => setDomain(HOME)} title="Crew atmospheres and most tests">
          Habitats
        </button>
        <button className="btn btn-s" onClick={() => setDomain(limits({ x: [0.1, 1.03], y: [0, maxKpa * 1.06] }))} title="Every test point">
          All
        </button>
      </div>
      {out > 0 && (
        <button className="cmap-out" onClick={() => setDomain(limits({ x: [0.1, 1.03], y: [0, maxKpa * 1.06] }))}>
          {out} test point{out === 1 ? "" : "s"} outside this view
        </button>
      )}
      {hover && (
        <div className="tooltip" style={{ left: hover.x + 14, top: hover.y + 14 }}>
          <b>{invName(hover.p.investigation)}</b> · {hover.p.fuel ?? "fuel not stated"}
          <br />
          {(hover.p.o2 * 100).toFixed(1)} % O₂ · {hover.p.kpa.toFixed(1)} kPa ({(hover.p.kpa / PSI_KPA).toFixed(1)} psi)
          {hover.p.approximate ? " · approximate" : ""}
          <br />
          <span className="muted">
            {hover.p.runs.length} run{hover.p.runs.length === 1 ? "" : "s"}
          </span>
        </div>
      )}
    </div>
  );
}

export function ShapeSwatch({ id, size = 12 }: { id: string; size?: number }) {
  const h = size / 2;
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} aria-hidden="true" style={{ fill: invColor(id), flex: "none" }}>
      {mark(invShape(id), h, h + (invShape(id) === "triangle" ? 1 : 0), h * 0.78)}
    </svg>
  );
}
