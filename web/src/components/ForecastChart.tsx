// Observed points vs model quantile bands vs revealed continuation, in one uPlot instance.
// The uPlot instance is created once; data changes call setData and the cutoff line is drawn from a
// ref in a draw hook, so moving the cutoff never re-renders React per pixel.
import { useEffect, useRef } from "react";
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
}

const css = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim() || "#888";

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

export function ForecastChart(props: Props) {
  const host = useRef<HTMLDivElement>(null);
  const plot = useRef<uPlot | null>(null);
  const shape = useRef<string>("");
  const cutoff = useRef(props.cutoff);
  cutoff.current = props.cutoff;

  // (Re)build only when the series structure changes (number of quantile levels).
  useEffect(() => {
    const el = host.current!;
    const { data, nQ } = merge(props);
    const key = `${nQ}|${props.yLabel}`;
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
      { label: "Observed (admitted at cutoff)", stroke: observedColor, width: 0, points: { show: true, size: 7, fill: observedColor, stroke: observedColor }, paths: () => null },
      { label: "Revealed continuation", stroke: revealedColor, width: 0, points: { show: true, size: 9, fill: "transparent", stroke: revealedColor, width: 2 }, paths: () => null },
    ];
    const bands: uPlot.Band[] = [];
    const levels = props.bands?.levels ?? [];
    levels.forEach((lv, l) => {
      const median = Math.abs(lv - 0.5) < 1e-9;
      series.push({
        label: `Model q${Math.round(lv * 100)}`,
        stroke: modelColor,
        width: median ? 2 : 1,
        dash: median ? [6, 4] : [2, 3],
        spanGaps: true,
        points: { show: false },
      });
      if (l < nQ / 2 && nQ - 1 - l !== l) {
        bands.push({ series: [3 + nQ - 1 - l, 3 + l], fill: modelColor + (l === 0 ? "22" : "33") });
      }
    });
    const opts: uPlot.Options = {
      width: el.clientWidth || 600,
      height: props.height ?? 320,
      series,
      bands,
      legend: { show: false },
      scales: { x: { time: false, min: 0, max: Math.max(props.xMax, props.cutoff) } },
      axes: [
        { label: "time since run origin [s]", stroke: css("--fg-muted"), grid: { stroke: css("--grid") } },
        { label: props.yLabel, stroke: css("--fg-muted"), grid: { stroke: css("--grid") }, size: 60 },
      ],
      hooks: {
        draw: [
          (u) => {
            const x = u.valToPos(cutoff.current, "x", true);
            const ctx = u.ctx;
            ctx.save();
            ctx.strokeStyle = css("--cutoff");
            ctx.lineWidth = 2 * devicePixelRatio;
            ctx.setLineDash([]);
            ctx.beginPath();
            ctx.moveTo(x, u.bbox.top);
            ctx.lineTo(x, u.bbox.top + u.bbox.height);
            ctx.stroke();
            ctx.fillStyle = css("--cutoff");
            ctx.font = `${12 * devicePixelRatio}px system-ui`;
            ctx.fillText("cutoff", x + 4 * devicePixelRatio, u.bbox.top + 14 * devicePixelRatio);
            ctx.restore();
          },
        ],
      },
    };
    plot.current = new uPlot(opts, data, el);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.observed, props.revealed, props.bands, props.yLabel, props.xMax]);

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
    const ro = new ResizeObserver(() => plot.current?.setSize({ width: el.clientWidth, height: props.height ?? 320 }));
    ro.observe(el);
    return () => {
      ro.disconnect();
      plot.current?.destroy();
      plot.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return <div ref={host} className="chart" aria-hidden="true" />;
}
