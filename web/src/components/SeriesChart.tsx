// T8 `series_comparison`: observed, computed and predicted series on one time axis. Observed values
// are points, computed lines are dashed, predictions are lines over their shaded band; a 2px ink bar
// marks where the forecast starts. Values are shown in the unit that reads at their magnitude (SI
// metres of a droplet read as millimetres). Hovering reads every series at that time and sets the
// answer's linked time; a time set elsewhere (the video) draws a cursor line here.
import { useEffect, useMemo, useRef, useState } from "react";
import { useScheme } from "../shell/theme";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { bandName, seriesName, type Series } from "../research/viewSpec";
import { displayUnit, fmtNum, scaled } from "../research/chartRows";
import { useLinkedTime } from "../shell/linked";
import { IconInfo } from "../shell/icons";

const css = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
/** Tick labels in the shown unit: plain digits ("3.4"), exponent form only at extreme magnitudes. */
const tick = (v: number) => (v === 0 ? "0" : Math.abs(v) < 1e-3 || Math.abs(v) >= 1e5 ? v.toExponential(1) : String(+v.toPrecision(3)));
const ROLE_COLOR = { observed: "--observed", computed: "--fg-muted", predicted: "--model" } as const;

function align(series: Series[], factor: number): uPlot.AlignedData {
  const x = [...new Set(series.flatMap((s) => s.times))].sort((a, b) => a - b);
  const idx = new Map(x.map((t, i) => [t, i]));
  const cols: (number | null)[][] = [];
  for (const s of series) {
    const col = (vals: (number | null)[]) => {
      const c = new Array<number | null>(x.length).fill(null);
      s.times.forEach((t, j) => (c[idx.get(t)!] = vals[j] == null ? null : scaled(vals[j]!, factor)));
      return c;
    };
    cols.push(col(s.values));
    if (s.band) cols.push(col(s.band.lower), col(s.band.upper));
  }
  return [x, ...cols] as uPlot.AlignedData;
}

/** "Droplet diameter (mm)", or the unit alone when no series names its quantity. */
function yTitle(series: Series[], unit: string) {
  const q = [...new Set(series.map((s) => s.quantity).filter(Boolean))];
  const u = unit === "1" ? "" : unit;
  return q.length === 1 ? (u ? `${q[0]} (${u})` : q[0]!) : u;
}

/** Where a forecast starts: the first time a predicted series has a value, when something earlier is drawn. */
function cutoffOf(series: Series[]): number | null {
  const pred = series.filter((s) => s.ref.role === "predicted").flatMap((s) => s.times.filter((_, j) => s.values[j] != null));
  if (!pred.length) return null;
  const start = Math.min(...pred);
  const first = Math.min(...series.flatMap((s) => s.times));
  return start > first ? start : null;
}

interface Readout {
  t: number;
  values: (string | null)[];
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
  const scheme = useScheme();
  const [read, setRead] = useState<Readout | null>(null);

  const shown = useMemo(() => displayUnit(series[0]?.unit ?? "", series.flatMap((s) => [...s.values, ...(s.band ? [...s.band.lower, ...s.band.upper] : [])])), [series]);
  const names = useMemo(() => series.map((s) => seriesName(s.ref, s.levels)), [series]);
  const cutoff = useMemo(() => cutoffOf(series), [series]);
  const unitText = shown.unit === "1" ? "" : shown.unit;

  useEffect(() => {
    const el = host.current!;
    const uSeries: uPlot.Series[] = [{ label: "Time" }];
    const bands: uPlot.Band[] = [];
    // Where each series' value sits in the aligned data, and its band edges when it has one.
    const cols: { v: number; lo: number | null; hi: number | null }[] = [];
    series.forEach((s, i) => {
      const color = css(ROLE_COLOR[s.ref.role]);
      const label = names[i];
      const v = uSeries.length;
      if (s.ref.role === "observed") uSeries.push({ label, stroke: color, width: 0, points: { show: true, size: 6, fill: color, stroke: color }, paths: () => null });
      else uSeries.push({ label, stroke: color, width: 2, dash: s.ref.role === "computed" ? [5, 4] : undefined, spanGaps: true, points: { show: false } });
      if (s.band) {
        // The band is a fill between its edges; the edges themselves are not drawn as lines.
        uSeries.push({ label: `${label} lower`, stroke: "transparent", width: 0, spanGaps: true, points: { show: false } });
        uSeries.push({ label: `${label} upper`, stroke: "transparent", width: 0, spanGaps: true, points: { show: false } });
        bands.push({ series: [v + 2, v + 1], fill: color + "29" });
        cols.push({ v, lo: v + 1, hi: v + 2 });
      } else cols.push({ v, lo: null, hi: null });
    });
    const muted = css("--fg-muted");
    const grid = css("--grid");
    let raf = 0;
    const p = new uPlot(
      {
        width: el.clientWidth || 600,
        height,
        series: uSeries,
        bands,
        legend: { show: false },
        cursor: { y: false, points: { size: 7 } },
        scales: { x: { time: false } },
        axes: [
          { label: "Time (s)", labelSize: 20, size: 30, gap: 4, labelFont: `500 12px ${css("--font-ui")}`, font: `12px ${css("--font-ui")}`, stroke: muted, grid: { stroke: grid }, ticks: { show: false } },
          { label: yTitle(series, shown.unit), labelSize: 22, labelFont: `500 12px ${css("--font-ui")}`, font: `12px ${css("--font-ui")}`, stroke: muted, grid: { stroke: grid }, ticks: { show: false }, size: 52, values: (_u, ticks) => ticks.map(tick) },
        ],
        hooks: {
          setCursor: [
            (u) => {
              const i = u.cursor.idx;
              cancelAnimationFrame(raf);
              if (i == null) {
                raf = requestAnimationFrame(() => setRead(null));
                return;
              }
              const t = u.data[0][i];
              const at = (c: number | null) => (c == null ? null : (u.data[c][i] as number | null | undefined) ?? null);
              const values = cols.map(({ v, lo, hi }) => {
                const x = at(v);
                if (x == null) return null;
                const a = at(lo);
                const b = at(hi);
                return a != null && b != null ? `${fmtNum(x)} (${fmtNum(a)}–${fmtNum(b)})` : fmtNum(x);
              });
              raf = requestAnimationFrame(() => {
                setRef.current({ origin, t, by: id });
                setRead({ t, values });
              });
            },
          ],
          draw: [
            (u) => {
              const ctx = u.ctx;
              const dpr = devicePixelRatio;
              const top = u.bbox.top;
              const bottom = u.bbox.top + u.bbox.height;
              if (cutoff != null) {
                const x = Math.round(u.valToPos(cutoff, "x", true));
                ctx.save();
                ctx.fillStyle = css("--cutoff");
                ctx.fillRect(x - dpr, top, 2 * dpr, bottom - top);
                ctx.font = `500 ${11 * dpr}px ${css("--font-ui")}`;
                ctx.fillStyle = muted;
                ctx.textBaseline = "top";
                ctx.textAlign = "left";
                ctx.fillText("Forecast", x + 6 * dpr, top + 4 * dpr);
                ctx.restore();
              }
              const t = linked.current;
              if (t == null) return;
              const x = u.valToPos(t, "x", true);
              ctx.save();
              ctx.strokeStyle = css("--revealed");
              ctx.lineWidth = 1.5 * dpr;
              ctx.beginPath();
              ctx.moveTo(x, top);
              ctx.lineTo(x, bottom);
              ctx.stroke();
              ctx.restore();
            },
          ],
        },
      },
      align(series, shown.factor),
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
  }, [series, height, origin, id, scheme, shown, names, cutoff]);

  // A time set by another view redraws the cursor line without rebuilding the plot.
  useEffect(() => {
    plot.current?.redraw(false);
  }, [time]);

  const originText = origin.replace(/_/g, " ");
  return (
    <div className="series">
      <div ref={host} className="chart" onMouseLeave={() => setRead(null)} />
      <div className="series-foot">
        <ul className="series-legend" aria-label="Series">
          {series.map((s, i) => (
            <li key={i}>
              <span className={`swatch swatch-${s.ref.role}`} aria-hidden="true" />
              <span>{names[i]}</span>
              {read?.values[i] != null && (
                <b className="series-val">
                  {read.values[i]}
                  {unitText && <span className="unit"> {unitText}</span>}
                </b>
              )}
            </li>
          ))}
          {series.map((s, i) =>
            s.band ? (
              <li key={`band-${i}`} className="series-band">
                <span className={`swatch swatch-band swatch-band-${s.ref.role}`} aria-hidden="true" />
                <span>{bandName(s.ref, s.levels)}</span>
              </li>
            ) : null,
          )}
          {cutoff != null && (
            <li>
              <span className="swatch swatch-cutoff" aria-hidden="true" />
              <span>Forecast starts</span>
            </li>
          )}
        </ul>
        {read ? (
          <span className="series-t num">t = {fmtNum(read.t)} s</span>
        ) : (
          originText && (
            <span className="series-origin" title={`Time is counted in seconds since ${originText}.`}>
              <IconInfo aria-hidden="true" />
              <span className="sr-only">Time is counted in seconds since {originText}.</span>
              <span aria-hidden="true">Time origin</span>
            </span>
          )
        )}
      </div>
    </div>
  );
}
