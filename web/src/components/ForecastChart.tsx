// Observed points vs model quantile bands vs revealed continuation, in one uPlot instance.
// The uPlot instance is created once; data changes call setData and the cutoff line is drawn from a
// ref in a draw hook, so moving the cutoff never re-renders React per pixel.
// Interactions: hovering reads the values at that time (observed point, median, outer band); with
// `onCutoff`, pressing on the plot and dragging moves the cutoff (the parent decides what it costs).
import { useEffect, useRef } from "react";
import { useScheme } from "../shell/theme";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";

export interface ChartPoint {
  t: number;
  v: number;
}

export interface Bands {
  times: number[];
  levels: number[];
  /** [level][time] in display units */
  q: number[][];
}

interface Props {
  observed: ChartPoint[];
  revealed: ChartPoint[] | null;
  bands: Bands | null;
  cutoff: number;
  xMax: number;
  yLabel: string;
  height?: number;
  /** Unit shown in the hover readout. */
  unit?: string;
  /** Makes the cutoff draggable on the plot. */
  onCutoff?: (t: number) => void;
}

const css = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";
const fmt = (x: number) => (Math.abs(x) >= 100 ? x.toFixed(0) : Math.abs(x) >= 10 ? x.toFixed(1) : x.toPrecision(3));

function merge(props: Props): { data: uPlot.AlignedData; nQ: number } {
  const xs = new Set<number>();
  props.observed.forEach((p) => xs.add(p.t));
  props.revealed?.forEach((p) => xs.add(p.t));
  props.bands?.times.forEach((t) => xs.add(t));
  const x = [...xs].sort((a, b) => a - b);
  const idx = new Map(x.map((t, i) => [t, i]));
  const col = () => new Array<number | null>(x.length).fill(null);
  const obs = col();
  props.observed.forEach((p) => (obs[idx.get(p.t)!] = p.v));
  const rev = col();
  props.revealed?.forEach((p) => (rev[idx.get(p.t)!] = p.v));
  const nQ = props.bands?.levels.length ?? 0;
  const qs: (number | null)[][] = [];
  for (let l = 0; l < nQ; l++) {
    const c = col();
    props.bands!.times.forEach((t, j) => (c[idx.get(t)!] = props.bands!.q[l][j]));
    qs.push(c);
  }
  return { data: [x, obs, rev, ...qs] as uPlot.AlignedData, nQ };
}

/** Linear interpolation of one quantile row at time t (null outside the forecast). */
function at(times: number[], row: number[], t: number): number | null {
  if (!times.length || t < times[0] || t > times[times.length - 1]) return null;
  let j = 1;
  while (j < times.length && times[j] < t) j++;
  if (j >= times.length) return row[times.length - 1];
  const t0 = times[j - 1];
  const t1 = times[j];
  return t1 === t0 ? row[j] : row[j - 1] + ((row[j] - row[j - 1]) * (t - t0)) / (t1 - t0);
}

function nearest(points: ChartPoint[], t: number, tol: number): ChartPoint | null {
  let best: ChartPoint | null = null;
  for (const p of points) if (Math.abs(p.t - t) <= tol && (!best || Math.abs(p.t - t) < Math.abs(best.t - t))) best = p;
  return best;
}

export function ForecastChart(props: Props) {
  const host = useRef<HTMLDivElement>(null);
  const tip = useRef<HTMLDivElement>(null);
  const plot = useRef<uPlot | null>(null);
  const shape = useRef<string>("");
  const cutoff = useRef(props.cutoff);
  cutoff.current = props.cutoff;
  const latest = useRef(props);
  latest.current = props;
  const dragging = useRef(false);

  // The colours are read from CSS tokens at build time, so a scheme change rebuilds the plot.
  const scheme = useScheme();

  // (Re)build only when the series structure changes (number of quantile levels) or the scheme.
  useEffect(() => {
    const el = host.current!;
    const { data, nQ } = merge(props);
    const key = `${nQ}|${props.yLabel}|${scheme}`;
    if (plot.current && shape.current === key) {
      plot.current.setData(data, false);
      plot.current.setScale("x", { min: 0, max: Math.max(props.xMax, props.cutoff) });
      return;
    }
    plot.current?.destroy();
    shape.current = key;
    const observedColor = css("--observed");
    const revealedColor = css("--revealed");
    const modelColor = css("--model");
    const series: uPlot.Series[] = [
      { label: "t [s]" },
      { label: "Observed (admitted at cutoff)", stroke: observedColor, width: 0, points: { show: true, size: 6, fill: observedColor, stroke: observedColor }, paths: () => null },
      { label: "Revealed continuation", stroke: revealedColor, width: 0, points: { show: true, size: 9, fill: "transparent", stroke: revealedColor, width: 2 }, paths: () => null },
    ];
    const bands: uPlot.Band[] = [];
    const levels = props.bands?.levels ?? [];
    levels.forEach((lv, l) => {
      const median = Math.abs(lv - 0.5) < 1e-9;
      series.push({
        label: `Model q${Math.round(lv * 100)}`,
        stroke: median ? modelColor : "transparent",
        width: median ? 2.5 : 0,
        dash: median ? [7, 5] : undefined,
        spanGaps: true,
        points: { show: false },
      });
      if (l < nQ / 2 && nQ - 1 - l !== l) {
        bands.push({ series: [3 + nQ - 1 - l, 3 + l], fill: modelColor + (l === 0 ? "26" : "40") });
      }
    });
    const opts: uPlot.Options = {
      width: el.clientWidth || 600,
      height: props.height ?? 360,
      series,
      bands,
      legend: { show: false },
      cursor: { drag: { x: false, y: false, setScale: false }, points: { show: false }, y: false },
      scales: { x: { time: false, min: 0, max: Math.max(props.xMax, props.cutoff) } },
      axes: [
        { label: "time since run origin [s]", stroke: css("--fg-muted"), grid: { stroke: css("--grid") } },
        { label: props.yLabel, stroke: css("--fg-muted"), grid: { stroke: css("--grid") }, size: 60 },
      ],
      hooks: {
        drawClear: [
          (u) => {
            // The region after the cutoff is what the model forecasts: a light wash behind the series.
            const x = u.valToPos(cutoff.current, "x", true);
            const ctx = u.ctx;
            ctx.save();
            ctx.fillStyle = modelColor + "0d";
            ctx.fillRect(x, u.bbox.top, u.bbox.left + u.bbox.width - x, u.bbox.height);
            ctx.restore();
          },
        ],
        draw: [
          (u) => {
            const dpr = devicePixelRatio;
            const x = u.valToPos(cutoff.current, "x", true);
            const ctx = u.ctx;
            ctx.save();
            ctx.strokeStyle = css("--cutoff");
            ctx.lineWidth = 2 * dpr;
            ctx.setLineDash([]);
            ctx.beginPath();
            ctx.moveTo(x, u.bbox.top);
            ctx.lineTo(x, u.bbox.top + u.bbox.height);
            ctx.stroke();
            // Handle: a small grip in the middle of the line says it can be dragged.
            if (latest.current.onCutoff) {
              const y = u.bbox.top + u.bbox.height / 2;
              ctx.fillStyle = css("--sheet");
              ctx.strokeStyle = css("--cutoff");
              ctx.lineWidth = 1.5 * dpr;
              ctx.beginPath();
              ctx.roundRect(x - 6 * dpr, y - 14 * dpr, 12 * dpr, 28 * dpr, 4 * dpr);
              ctx.fill();
              ctx.stroke();
              ctx.beginPath();
              for (const dy of [-5, 0, 5]) {
                ctx.moveTo(x - 3 * dpr, y + dy * dpr);
                ctx.lineTo(x + 3 * dpr, y + dy * dpr);
              }
              ctx.stroke();
            }
            ctx.font = `600 ${11 * dpr}px system-ui`;
            ctx.textBaseline = "top";
            const top = u.bbox.top + 6 * dpr;
            if (x - u.bbox.left > 110 * dpr) {
              ctx.fillStyle = css("--fg-muted");
              ctx.textAlign = "right";
              ctx.fillText("← given to the model", x - 8 * dpr, u.bbox.top + u.bbox.height - 18 * dpr);
            }
            ctx.textAlign = "left";
            ctx.fillStyle = modelColor;
            ctx.fillText(`→ forecast after ${fmt(cutoff.current)} s`, x + 8 * dpr, top);
            ctx.restore();
          },
        ],
        setCursor: [
          (u) => {
            const t = tip.current;
            if (!t) return;
            const left = u.cursor.left ?? -1;
            if (left < 0 || dragging.current) {
              t.style.display = "none";
              return;
            }
            const p = latest.current;
            const time = u.posToVal(left, "x");
            const tol = Math.abs(u.posToVal(left + 8, "x") - time);
            const unit = p.unit ? ` ${p.unit}` : "";
            const rows: string[] = [`<b>t = ${fmt(time)} s</b>`];
            const o = nearest(p.observed, time, tol);
            if (o) rows.push(`<span class="ft-obs">●</span> observed ${fmt(o.v)}${unit}`);
            const r = p.revealed ? nearest(p.revealed, time, tol) : null;
            if (r) rows.push(`<span class="ft-rev">○</span> revealed ${fmt(r.v)}${unit}`);
            if (p.bands && p.bands.levels.length) {
              const lv = p.bands.levels;
              const mi = lv.findIndex((x) => Math.abs(x - 0.5) < 1e-9);
              const med = mi >= 0 ? at(p.bands.times, p.bands.q[mi], time) : null;
              const lo = at(p.bands.times, p.bands.q[0], time);
              const hi = at(p.bands.times, p.bands.q[lv.length - 1], time);
              if (med != null) rows.push(`<span class="ft-mod">- -</span> model median ${fmt(med)}${unit}`);
              if (lo != null && hi != null)
                rows.push(`<span class="ft-band">▮</span> ${Math.round((lv[lv.length - 1] - lv[0]) * 100)}% range ${fmt(lo)}–${fmt(hi)}${unit}`);
            }
            if (time > cutoff.current && !o && !(p.bands && p.bands.levels.length)) rows.push(`<span class="muted">after the cutoff: hidden from the model</span>`);
            t.innerHTML = rows.join("<br>");
            t.style.display = "block";
            const w = t.offsetWidth;
            const ox = u.over.offsetLeft;
            const x = left + ox;
            t.style.left = `${left + w + 16 > u.over.clientWidth ? x - w - 12 : x + 12}px`;
            t.style.top = `${Math.max(0, (u.cursor.top ?? 0) + u.over.offsetTop - 10)}px`;
          },
        ],
        ready: [
          (u) => {
            // Press anywhere on the plot (or on the line) and drag to move the cutoff.
            const over = u.over;
            const timeAt = (e: PointerEvent) => {
              const r = over.getBoundingClientRect();
              const v = u.posToVal(Math.max(0, Math.min(e.clientX - r.left, r.width)), "x");
              return Math.max(0, Math.min(v, u.scales.x.max ?? v));
            };
            const nearLine = (e: PointerEvent) => {
              const r = over.getBoundingClientRect();
              return Math.abs(e.clientX - r.left - u.valToPos(cutoff.current, "x")) < 10;
            };
            over.addEventListener("pointermove", (e) => {
              const cb = latest.current.onCutoff;
              if (!cb) return;
              over.style.cursor = dragging.current || nearLine(e) ? "ew-resize" : "crosshair";
              if (dragging.current) cb(timeAt(e));
            });
            over.addEventListener("pointerdown", (e) => {
              const cb = latest.current.onCutoff;
              if (!cb || e.button !== 0) return;
              dragging.current = true;
              over.setPointerCapture(e.pointerId);
              if (tip.current) tip.current.style.display = "none";
              cb(timeAt(e));
            });
            const end = (e: PointerEvent) => {
              if (!dragging.current) return;
              dragging.current = false;
              try {
                over.releasePointerCapture(e.pointerId);
              } catch {
                /* already released */
              }
            };
            over.addEventListener("pointerup", end);
            over.addEventListener("pointercancel", end);
          },
        ],
      },
    };
    plot.current = new uPlot(opts, data, el);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.observed, props.revealed, props.bands, props.yLabel, props.xMax, scheme]);

  // Cutoff moves immediately: redraw only.
  useEffect(() => {
    const u = plot.current;
    if (!u) return;
    const max = Math.max(props.xMax, props.cutoff);
    if (u.scales.x.max !== max) u.setScale("x", { min: 0, max });
    else u.redraw(false);
  }, [props.cutoff, props.xMax]);

  useEffect(() => {
    const el = host.current!;
    const ro = new ResizeObserver(() => plot.current?.setSize({ width: el.clientWidth, height: props.height ?? 360 }));
    ro.observe(el);
    return () => {
      ro.disconnect();
      plot.current?.destroy();
      plot.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="chart-wrap">
      <div ref={host} className="chart" aria-hidden="true" />
      <div ref={tip} className="chart-tip" role="presentation" />
    </div>
  );
}
