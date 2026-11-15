// T8 `chart`: a bar, line or scatter chart over rows of a cited artifact, drawn as SVG from the
// rows the view names (research/chartRows.ts). Categorical x is drawn as bars (or dots at category
// positions); numeric x as lines or points, linear or log. Series are the view's y fields times its
// groups, each with a categorical colour from the validated chart palette and its own legend entry
// (click to hide). Hover shows the row's label and every value with its unit.
import { useEffect, useMemo, useRef, useState, type MouseEvent } from "react";
import { axisTitle, shownUnit, fmtNum, fullNum, logTicks, niceTicks, rowCount, type ChartData, type ChartPoint, type ChartView } from "../research/chartRows";

const H = 300;
const CH = 6.4; // approximate width of one 11.5px label character
const color = (s: number) => `var(--cat-${(s % 5) + 1})`;
const DASH = [undefined, "6 3", "2 3", "8 3 2 3"];

interface Scale {
  pos: (v: number) => number;
  ticks: number[];
  lo: number;
  hi: number;
}

function linearOrLog(vals: number[], log: boolean, from: number, to: number, count: number, zero: boolean): Scale {
  let lo = Math.min(...vals);
  let hi = Math.max(...vals);
  if (log) {
    const t = logTicks(lo, hi) ?? { domain: [1, 10] as [number, number], ticks: [1, 10] };
    const [a, b] = t.domain.map(Math.log10);
    return { pos: (v) => from + ((Math.log10(v) - a) / (b - a)) * (to - from), ticks: t.ticks, lo: t.domain[0], hi: t.domain[1] };
  }
  if (zero) {
    lo = Math.min(0, lo);
    hi = Math.max(0, hi);
  }
  const ticks = niceTicks(lo, hi, count);
  lo = ticks[0];
  hi = ticks[ticks.length - 1];
  return { pos: (v) => from + ((v - lo) / (hi - lo || 1)) * (to - from), ticks, lo, hi };
}

function useWidth() {
  const ref = useRef<HTMLDivElement>(null);
  const [w, setW] = useState(640);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setW(Math.max(280, Math.round(el.clientWidth))));
    ro.observe(el);
    setW(Math.max(280, Math.round(el.clientWidth || 640)));
    return () => ro.disconnect();
  }, []);
  return [ref, w] as const;
}

interface Hover {
  p: ChartPoint;
  k: number;
  x: number;
  y: number;
}

export function AnswerChart({ view, data }: { view: ChartView; data: ChartData }) {
  const [box, W] = useWidth();
  const [hidden, setHidden] = useState<Set<number>>(new Set());
  const [hover, setHover] = useState<Hover | null>(null);
  const ny = view.y.length;
  const series = useMemo(
    () =>
      data.groups.flatMap((g, gi) =>
        view.y.map((a, k) => ({ id: gi * ny + k, group: g, k, name: g != null && ny > 1 ? `${g} · ${a.title}` : g ?? a.title })),
      ),
    [data.groups, view.y, ny],
  );
  const sid = (p: ChartPoint, k: number) => data.groups.indexOf(p.group) * ny + k;
  const visible = (p: ChartPoint, k: number) => p.y[k] != null && !hidden.has(sid(p, k));

  const yUnit = [...new Set(view.y.map((a) => shownUnit(a.unit)).filter(Boolean))];
  const yTitle = ny === 1 ? axisTitle(view.y[0]) : `${view.y.map((a) => a.title).join(", ")}${yUnit.length ? ` (${yUnit.join(", ")})` : ""}`;
  const ylog = view.y.some((a) => a.log);
  const bars = data.categorical && view.mark === "bar";

  const yVals = data.points.flatMap((p) => p.y.flatMap((v, k) => (v != null && visible(p, k) ? [v] : [])));
  const allY = yVals.length ? yVals : data.points.flatMap((p) => p.y.filter((v): v is number => v != null));
  const yTickText = (v: number) => fmtNum(v);
  // Left margin fits the widest tick label; bottom fits category labels, rotated when they crowd.
  const yProbe = linearOrLog(allY.length ? allY : [0, 1], ylog, 0, 1, 5, bars);
  const left = 22 + Math.max(...yProbe.ticks.map((t) => yTickText(t).length)) * CH + 10;
  const right = 16;
  const plotW = W - left - right;
  const cats = data.categories;
  const step = cats.length ? plotW / cats.length : plotW;
  const longest = Math.max(0, ...cats.map((c) => Math.min(c.length, 20)));
  const rotate = data.categorical && longest * CH > step - 8;
  const bottom = 40 + (rotate ? Math.min(20, longest) * CH * 0.6 : 0);
  const top = 12;
  const plotH = H - top - bottom;
  const ys = linearOrLog(allY.length ? allY : [0, 1], ylog, top + plotH, top, 5, bars);
  const xs = data.categorical ? null : linearOrLog(data.points.map((p) => p.x as number), !!view.x.log, left, left + plotW, Math.max(3, Math.floor(plotW / 90)), false);
  const catX = (c: string) => left + (cats.indexOf(c) + 0.5) * step;

  // Bar or dot slots per category: one per visible (point, y) value, in series order.
  let slots: Map<string, { p: ChartPoint; k: number }[]> | null = null;
  if (data.categorical) {
    slots = new Map();
    for (const p of data.points)
      for (let k = 0; k < ny; k++) if (visible(p, k)) slots.set(p.x as string, [...(slots.get(p.x as string) ?? []), { p, k }]);
    for (const l of slots.values()) l.sort((a, b) => sid(a.p, a.k) - sid(b.p, b.k));
  }
  const maxSlots = slots ? Math.max(1, ...[...slots.values()].map((l) => l.length)) : 1;
  const barW = Math.max(2, Math.min(34, (step * 0.78) / maxSlots));
  const base = ylog ? ys.pos(ys.lo) : ys.pos(Math.max(ys.lo, Math.min(0, ys.hi)));

  const xyOf = (p: ChartPoint, k: number): [number, number] => {
    const y = ys.pos(p.y[k]!);
    if (!data.categorical) return [xs!.pos(p.x as number), y];
    const l = slots?.get(p.x as string) ?? [];
    const j = l.findIndex((s) => s.p === p && s.k === k);
    return [catX(p.x as string) + (j - (l.length - 1) / 2) * barW, y];
  };

  const lines =
    view.mark !== "line" || data.categorical
      ? []
      : series
          .filter((s) => !hidden.has(s.id))
          .map((s) => {
            const pts = data.points.filter((p) => p.group === s.group && p.y[s.k] != null).sort((a, b) => (a.x as number) - (b.x as number));
            return { s, d: pts.map((p, i) => `${i ? "L" : "M"}${xs!.pos(p.x as number).toFixed(1)},${ys.pos(p.y[s.k]!).toFixed(1)}`).join("") };
          });

  const onMove = (e: MouseEvent<SVGSVGElement>) => {
    if (bars) return;
    const r = e.currentTarget.getBoundingClientRect();
    const mx = e.clientX - r.left;
    const my = e.clientY - r.top;
    let best: Hover | null = null;
    let bd = 28 * 28;
    for (const p of data.points)
      for (let k = 0; k < ny; k++) {
        if (!visible(p, k)) continue;
        const [x, y] = xyOf(p, k);
        const d = (x - mx) ** 2 + (y - my) ** 2;
        if (d < bd) {
          bd = d;
          best = { p, k, x, y };
        }
      }
    setHover(best);
  };

  const nPts = data.points.length;
  const showDots = view.mark === "scatter" || data.categorical || nPts <= 80;
  const label = `${view.mark} chart of ${view.y.map((a) => a.title).join(", ")} by ${view.x.title}, ${rowCount(nPts, data.noun)}`;
  // Points drawn on the same pixel (same x and y): the dots are translucent so a stack reads darker,
  // and the tooltip counts them.
  const here = (x: number, y: number) => `${Math.round(x)}:${Math.round(y)}`;
  const stacks = new Map<string, number>();
  if (!bars && showDots)
    for (const p of data.points)
      for (let k = 0; k < ny; k++) {
        if (!visible(p, k)) continue;
        const [x, y] = xyOf(p, k);
        stacks.set(here(x, y), (stacks.get(here(x, y)) ?? 0) + 1);
      }
  const stacked = hover && !bars ? stacks.get(here(hover.x, hover.y)) ?? 1 : 1;

  return (
    <div className="chart-box" ref={box}>
      {series.length > 1 && (
        <div className="chart-legend" role="group" aria-label="Series">
          {series.map((s) => (
            <button key={s.id} className={`chart-key${hidden.has(s.id) ? " off" : ""}`} aria-pressed={!hidden.has(s.id)} onClick={() => setHidden((h) => { const n = new Set(h); if (n.has(s.id)) n.delete(s.id); else n.add(s.id); return n; })}>
              <span className="chart-sw" style={{ background: color(s.id) }} aria-hidden="true" />
              {s.name}
            </button>
          ))}
        </div>
      )}
      <svg className="answer-chart" width={W} height={H} viewBox={`0 0 ${W} ${H}`} role="img" aria-label={label} onMouseMove={onMove} onMouseLeave={() => setHover(null)}>
        {/* horizontal grid and y ticks */}
        {ys.ticks.map((t) => (
          <g key={`y${t}`}>
            <line className="ch-grid" x1={left} x2={left + plotW} y1={ys.pos(t)} y2={ys.pos(t)} />
            <text className="ch-tick" x={left - 8} y={ys.pos(t)} dy="0.32em" textAnchor="end">
              {yTickText(t)}
            </text>
          </g>
        ))}
        {/* x axis */}
        <line className="ch-axis" x1={left} x2={left + plotW} y1={top + plotH} y2={top + plotH} />
        {xs &&
          xs.ticks.map((t) => (
            <g key={`x${t}`}>
              {view.mark === "scatter" && <line className="ch-grid" x1={xs.pos(t)} x2={xs.pos(t)} y1={top} y2={top + plotH} />}
              <line className="ch-axis" x1={xs.pos(t)} x2={xs.pos(t)} y1={top + plotH} y2={top + plotH + 4} />
              <text className="ch-tick" x={xs.pos(t)} y={top + plotH + 16} textAnchor="middle">
                {fmtNum(t)}
              </text>
            </g>
          ))}
        {data.categorical &&
          cats.map((c) => {
            const x = catX(c);
            const t = c.length > 20 ? `${c.slice(0, 19)}…` : c;
            return rotate ? (
              <text key={c} className="ch-tick" transform={`translate(${x},${top + plotH + 12}) rotate(-35)`} textAnchor="end">
                <title>{c}</title>
                {t}
              </text>
            ) : (
              <text key={c} className="ch-tick" x={x} y={top + plotH + 16} textAnchor="middle">
                <title>{c}</title>
                {t}
              </text>
            );
          })}
        {/* axis titles */}
        <text className="ch-title" x={left + plotW / 2} y={H - 6} textAnchor="middle">
          {axisTitle(view.x)}
          {view.x.log ? " · log" : ""}
        </text>
        <text className="ch-title" transform={`translate(14,${top + plotH / 2}) rotate(-90)`} textAnchor="middle">
          {yTitle}
          {ylog ? " · log" : ""}
        </text>

        {/* marks */}
        {lines.map(({ s, d }) => (
          <path key={s.id} className="ch-line" d={d} style={{ stroke: color(s.id) }} strokeDasharray={DASH[Math.floor(s.id / 5) % DASH.length]} />
        ))}
        {bars && slots
          ? [...slots.values()].flatMap((l) =>
              l.map(({ p, k }) => {
                const [x, y] = xyOf(p, k);
                const s = sid(p, k);
                const on = hover?.p === p && hover.k === k;
                return (
                  <rect
                    key={`${p.row}:${k}`}
                    className={`ch-bar${hover && !on ? " dim" : ""}`}
                    x={x - barW / 2 + 0.5}
                    width={Math.max(1, barW - 1)}
                    y={Math.min(y, base)}
                    height={Math.max(1, Math.abs(base - y))}
                    rx={Math.min(3, barW / 4)}
                    style={{ fill: color(s) }}
                    onMouseEnter={() => setHover({ p, k, x, y: Math.min(y, base) })}
                    onMouseLeave={() => setHover(null)}
                  />
                );
              }),
            )
          : showDots &&
            data.points.flatMap((p) =>
              view.y.map((_, k) => {
                if (!visible(p, k)) return null;
                const [x, y] = xyOf(p, k);
                const s = sid(p, k);
                return <circle key={`${p.row}:${k}`} className={`ch-dot${view.mark === "scatter" ? " ch-dot-scatter" : ""}`} cx={x} cy={y} r={view.mark === "line" ? 2.6 : 3.6} style={{ fill: color(s), stroke: color(s) }} />;
              }),
            )}
        {hover && !bars && <circle className="ch-hover" cx={hover.x} cy={hover.y} r={6} style={{ stroke: color(sid(hover.p, hover.k)) }} />}
      </svg>
      {hover && <ChartTip view={view} h={hover} W={W} color={color(sid(hover.p, hover.k))} categorical={data.categorical} stacked={stacked > 1 ? rowCount(stacked, data.noun) : null} />}
    </div>
  );
}

function ChartTip({ view, h, W, color: c, categorical, stacked }: { view: ChartView; h: Hover; W: number; color: string; categorical: boolean; stacked: string | null }) {
  const { p } = h;
  const title = p.label ?? (categorical ? String(p.x) : null);
  const tipW = 220;
  const left = h.x + 14 + tipW > W ? Math.max(0, h.x - 14 - tipW) : h.x + 14;
  const val = (v: number | string, unit?: string | null) => (
    <span title={typeof v === "number" ? fullNum(v) : undefined}>
      {typeof v === "number" ? fmtNum(v) : v}
      {shownUnit(unit) ? <span className="unit"> {unit}</span> : null}
    </span>
  );
  return (
    <div className="chart-tip" style={{ left, top: Math.max(0, h.y - 12), width: tipW }} role="status">
      {title && <b className="chart-tip-t">{title}</b>}
      {p.group != null && (
        <span className="chart-tip-g">
          <span className="chart-sw" style={{ background: c }} aria-hidden="true" />
          {p.group}
        </span>
      )}
      <dl>
        {!(categorical && !p.label) && (
          <>
            <dt>{view.x.title}</dt>
            <dd>{val(p.x, view.x.unit)}</dd>
          </>
        )}
        {view.y.map((a, k) => (
          <div key={k} className={k === h.k ? "on" : undefined} style={{ display: "contents" }}>
            <dt>{a.title}</dt>
            <dd>{p.y[k] != null ? val(p.y[k]!, a.unit) : <span className="dim">—</span>}</dd>
          </div>
        ))}
      </dl>
      {stacked && <span className="chart-tip-n">{stacked} here</span>}
    </div>
  );
}
