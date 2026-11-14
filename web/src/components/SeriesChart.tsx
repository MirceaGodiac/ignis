// T8 `series_comparison`: observed, computed and predicted series on one time axis. Observed values
// are points, computed lines are dashed, predictions are lines with their band. Hovering sets the
// answer's linked time; a time set elsewhere (the video) draws a cursor line here.
import { useEffect, useRef, useState } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import type { Series } from "../research/viewSpec";
import { useLinkedTime } from "../shell/linked";

const css = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
/** Tick labels that keep their digits at any scale (SI values of a droplet are around 1e-3 m). */
const tick = (v: number) => (v === 0 ? "0" : Math.abs(v) < 1e-2 || Math.abs(v) >= 1e5 ? v.toExponential(2) : String(+v.toPrecision(3)));
const ROLE_COLOR = { observed: "--observed", computed: "--fg-muted", predicted: "--model" } as const;

function align(series: Series[]): uPlot.AlignedData {
  const x = [...new Set(series.flatMap((s) => s.times))].sort((a, b) => a - b);
  const idx = new Map(x.map((t, i) => [t, i]));
  const cols: (number | null)[][] = [];
  for (const s of series) {
    const col = (vals: (number | null)[]) => {
      const c = new Array<number | null>(x.length).fill(null);
      s.times.forEach((t, j) => (c[idx.get(t)!] = vals[j]));
      return c;
    };
    cols.push(col(s.values));
    if (s.band) cols.push(col(s.band.lower), col(s.band.upper));
  }
  return [x, ...cols] as uPlot.AlignedData;
}

export function SeriesChart({ series, id, height = 260 }: { series: Series[]; id: string; height?: number }) {
  const host = useRef<HTMLDivElement>(null);
  const plot = useRef<uPlot | null>(null);
  const { time, setTime } = useLinkedTime();
  const origin = series[0]?.timeOrigin ?? "";
  const linked = useRef<number | null>(null);
  linked.current = time && time.origin === origin && time.by !== id ? time.t : null;
  const setRef = useRef(setTime);
  setRef.current = setTime;
  const [scheme, setScheme] = useState(() => (window.matchMedia?.("(prefers-color-scheme: dark)").matches ? "dark" : "light"));
  useEffect(() => {
    const mq = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!mq) return;
    const on = () => setScheme(mq.matches ? "dark" : "light");
    mq.addEventListener("change", on);
    return () => mq.removeEventListener("change", on);
  }, []);

  useEffect(() => {
    const el = host.current!;
    const uSeries: uPlot.Series[] = [{ label: "t [s]" }];
    const bands: uPlot.Band[] = [];
    for (const s of series) {
      const color = css(ROLE_COLOR[s.ref.role]);
      const label = s.ref.label ?? `${s.ref.role} · ${s.ref.artifact_id}`;
      if (s.ref.role === "observed") uSeries.push({ label, stroke: color, width: 0, points: { show: true, size: 6, fill: color, stroke: color }, paths: () => null });
      else uSeries.push({ label, stroke: color, width: 2, dash: s.ref.role === "computed" ? [5, 4] : undefined, spanGaps: true, points: { show: false } });
      if (s.band) {
        const at = uSeries.length; // lower, upper follow the series
        uSeries.push({ label: `${label} lower`, stroke: color, width: 1, dash: [2, 3], spanGaps: true, points: { show: false } });
        uSeries.push({ label: `${label} upper`, stroke: color, width: 1, dash: [2, 3], spanGaps: true, points: { show: false } });
        bands.push({ series: [at + 1, at], fill: color + "26" });
      }
    }
    let raf = 0;
    const p = new uPlot(
      {
        width: el.clientWidth || 600,
        height,
        series: uSeries,
        bands,
        legend: { show: false },
        scales: { x: { time: false } },
        axes: [
          { label: `seconds since ${origin.replace(/_/g, " ")}`, stroke: css("--fg-muted"), grid: { stroke: css("--grid") } },
          { label: series[0]?.unit ?? "", stroke: css("--fg-muted"), grid: { stroke: css("--grid") }, size: 72, values: (_u, ticks) => ticks.map(tick) },
        ],
        hooks: {
          setCursor: [
            (u) => {
              const i = u.cursor.idx;
              if (i == null) return;
              const t = u.data[0][i];
              cancelAnimationFrame(raf);
              raf = requestAnimationFrame(() => setRef.current({ origin, t, by: id }));
            },
          ],
          draw: [
            (u) => {
              const t = linked.current;
              if (t == null) return;
              const x = u.valToPos(t, "x", true);
              const ctx = u.ctx;
              ctx.save();
              ctx.strokeStyle = css("--revealed");
              ctx.lineWidth = 1.5 * devicePixelRatio;
              ctx.beginPath();
              ctx.moveTo(x, u.bbox.top);
              ctx.lineTo(x, u.bbox.top + u.bbox.height);
              ctx.stroke();
              ctx.restore();
            },
          ],
        },
      },
      align(series),
      el,
    );
    plot.current = p;
    const ro = new ResizeObserver(() => p.setSize({ width: el.clientWidth || 600, height }));
    ro.observe(el);
    return () => {
      cancelAnimationFrame(raf);
      ro.disconnect();
      p.destroy();
      plot.current = null;
    };
  }, [series, height, origin, id, scheme]);

  // A time set by another view redraws the cursor line without rebuilding the plot.
  useEffect(() => {
    plot.current?.redraw(false);
  }, [time]);

  return (
    <div>
      <div ref={host} className="chart" />
      <div className="legend-inline small">
        {series.map((s, i) => (
          <span key={i}>
            <span className={`swatch swatch-${s.ref.role}`} aria-hidden="true" /> {s.ref.label ?? s.ref.artifact_id}
            {!(s.ref.label ?? "").toLowerCase().includes(s.ref.role) && <span className="dim"> ({s.ref.role})</span>}
          </span>
        ))}
      </div>
    </div>
  );
}
