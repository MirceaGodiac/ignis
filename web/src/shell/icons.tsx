// One authored icon set: 16px grid, 1.5 stroke, round joins. Geometry only.
import type { SVGProps } from "react";

type P = SVGProps<SVGSVGElement>;
const base = (p: P) => ({ width: 16, height: 16, viewBox: "0 0 16 16", fill: "none", stroke: "currentColor", strokeWidth: 1.5, strokeLinecap: "round" as const, strokeLinejoin: "round" as const, "aria-hidden": true, ...p });

export const IconMap = (p: P) => (
  <svg {...base(p)}>
    <path d="M2.5 13.5V2.5M2.5 13.5h11" />
    <circle cx="6" cy="9.5" r="1" />
    <circle cx="9" cy="6.5" r="1" />
    <rect x="9.5" y="9" width="3" height="3" rx="0.5" strokeDasharray="1.5 1.2" />
  </svg>
);
export const IconMatrix = (p: P) => (
  <svg {...base(p)}>
    <rect x="2.5" y="2.5" width="11" height="11" rx="1" />
    <path d="M2.5 6.5h11M6.5 2.5v11" />
    <path d="M9 9.5h2.5M9 11.5h1.5" strokeDasharray="1.5 1.2" />
  </svg>
);
export const IconRuns = (p: P) => (
  <svg {...base(p)}>
    <path d="M8 14.25c-2.6 0-4.5-1.75-4.5-4.1 0-1.7.95-2.85 1.9-3.9.85-.95 1.55-2.05 1.5-3.5 2.1.95 3.55 2.85 3.65 4.85.5-.3.9-.85 1.05-1.55 1.05 1 1.9 2.45 1.9 4.15 0 2.35-1.9 4.05-4.5 4.05z" />
    <path d="M8 14.25c-1.05 0-1.85-.75-1.85-1.75 0-.95.7-1.6 1.85-2.75 1.15 1.15 1.85 1.8 1.85 2.75 0 1-.8 1.75-1.85 1.75z" />
  </svg>
);
export const IconCompare = (p: P) => (
  <svg {...base(p)}>
    <path d="M2.5 2.5v11h11" />
    <path d="M4.5 10.5c1.5 0 2-4.5 4-4.5s2.25 2.5 4.25 2.5" />
    <path d="M4.5 12c1.75 0 2.5-2.25 4.25-2.25s2.5 1 4 1" strokeDasharray="1.6 1.6" />
  </svg>
);
export const IconLab = (p: P) => (
  <svg {...base(p)}>
    <path d="M9.33 1.33v4a1.33 1.33 0 0 0 .16.64l3.67 6.72A1.33 1.33 0 0 1 12 14.67H4a1.33 1.33 0 0 1-1.17-1.97l3.67-6.72a1.33 1.33 0 0 0 .17-.65v-4" />
    <path d="M4.3 10h7.4M5.67 1.33h4.66" />
  </svg>
);
export const IconSources = (p: P) => (
  <svg {...base(p)}>
    <path d="M9.25 2.25H5c-.83 0-1.5.67-1.5 1.5v8.5c0 .83.67 1.5 1.5 1.5h6c.83 0 1.5-.67 1.5-1.5V5.5z" />
    <path d="M9.25 2.25V4.5c0 .55.45 1 1 1h2.25" />
    <path d="M5.75 8.25h4.5M5.75 10.75h3" />
  </svg>
);
export const IconModels = (p: P) => (
  <svg {...base(p)}>
    <path d="M7.35 2.6a1.3 1.3 0 0 1 1.3 0l3.9 2.2c.4.23.65.66.65 1.13v4.14c0 .47-.25.9-.65 1.13l-3.9 2.2a1.3 1.3 0 0 1-1.3 0l-3.9-2.2a1.3 1.3 0 0 1-.65-1.13V5.93c0-.47.25-.9.65-1.13z" />
    <path d="M3 5.1 8 7.9l5-2.8M8 7.9v5.7" />
  </svg>
);
export const IconThread = (p: P) => (
  <svg {...base(p)}>
    <path d="M3 4.5c0-.97.78-1.75 1.75-1.75h6.5c.97 0 1.75.78 1.75 1.75v4.75c0 .97-.78 1.75-1.75 1.75H7.5l-2.6 2.1c-.33.27-.82.03-.82-.4V11H4.75C3.78 11 3 10.22 3 9.25z" />
  </svg>
);
export const IconPlus = (p: P) => (
  <svg {...base(p)}>
    <path d="M8 3.25v9.5M3.25 8h9.5" />
  </svg>
);
export const IconSearch = (p: P) => (
  <svg {...base(p)}>
    <circle cx="7.25" cy="7.25" r="4.5" />
    <path d="M10.6 10.6 13.5 13.5" />
  </svg>
);
export const IconChevron = (p: P) => (
  <svg {...base(p)}>
    <path d="M6 3.5L10.5 8 6 12.5" />
  </svg>
);
export const IconBack = (p: P) => (
  <svg {...base(p)}>
    <path d="M10 3.5L5.5 8l4.5 4.5" />
  </svg>
);
export const IconClose = (p: P) => (
  <svg {...base(p)}>
    <path d="M4 4l8 8M12 4l-8 8" />
  </svg>
);
export const IconExternal = (p: P) => (
  <svg {...base(p)}>
    <path d="M9 3h4v4M13 3L7.5 8.5M11.5 9.5V13H3V4.5h3.5" />
  </svg>
);
export const IconMenu = (p: P) => (
  <svg {...base(p)}>
    <path d="M2.5 4.5h11M2.5 8h11M2.5 11.5h11" />
  </svg>
);
export const IconHome = (p: P) => (
  <svg {...base(p)}>
    <path d="M2 6.67a1.33 1.33 0 0 1 .47-1.02l4.67-4a1.33 1.33 0 0 1 1.72 0l4.67 4A1.33 1.33 0 0 1 14 6.67v6a1.33 1.33 0 0 1-1.33 1.33H3.33A1.33 1.33 0 0 1 2 12.67z" />
    <path d="M10 14V8.67a.67.67 0 0 0-.67-.67H6.67a.67.67 0 0 0-.67.67V14" />
  </svg>
);
export const IconEvidence = (p: P) => (
  <svg {...base(p)}>
    <path d="M5.5 4.25H4.75C3.78 4.25 3 5.03 3 6v1.75c0 .97.78 1.75 1.75 1.75h.75c0 1.4-.7 2.4-1.9 3" />
    <path d="M11.5 4.25h-.75C9.78 4.25 9 5.03 9 6v1.75c0 .97.78 1.75 1.75 1.75h.75c0 1.4-.7 2.4-1.9 3" />
  </svg>
);
export const IconRoute = (p: P) => (
  <svg {...base(p)}>
    <circle cx="4.25" cy="4" r="1.5" />
    <circle cx="11.75" cy="12" r="1.5" />
    <path d="M5.75 4h4a2.25 2.25 0 0 1 0 4.5h-3.5a2.25 2.25 0 0 0 0 4.5h4" />
  </svg>
);
export const IconUpDown = (p: P) => (
  <svg {...base(p)}>
    <path d="M5 6l3-3 3 3M5 10l3 3 3-3" />
  </svg>
);
export const IconInfo = (p: P) => (
  <svg {...base(p)}>
    <circle cx="8" cy="8" r="5.75" />
    <path d="M8 7.25v3.5M8 5.15v.1" />
  </svg>
);
export const IconArrow = (p: P) => (
  <svg {...base(p)}>
    <path d="M3 8h10M9 4l4 4-4 4" />
  </svg>
);

export const IconSun = (p: P) => (
  <svg {...base(p)}>
    <circle cx="8" cy="8" r="2.75" />
    <path d="M8 1.75v1.5M8 12.75v1.5M1.75 8h1.5M12.75 8h1.5M3.6 3.6l1.05 1.05M11.35 11.35l1.05 1.05M3.6 12.4l1.05-1.05M11.35 4.65l1.05-1.05" />
  </svg>
);
export const IconMoon = (p: P) => (
  <svg {...base(p)}>
    <path d="M13 9.6A5.25 5.25 0 0 1 6.4 3a5.25 5.25 0 1 0 6.6 6.6z" />
  </svg>
);
export const IconSystem = (p: P) => (
  <svg {...base(p)}>
    <rect x="2" y="3" width="12" height="8" rx="1.25" />
    <path d="M6 13.5h4M8 11v2.5" />
  </svg>
);

export const IconConditions = (p: P) => (
  <svg {...base(p)}>
    <path d="M2.5 4.5h6M11.5 4.5h2M2.5 8h2M7.5 8h6M2.5 11.5h7.5M13 11.5h.5" />
    <circle cx="10" cy="4.5" r="1.5" />
    <circle cx="6" cy="8" r="1.5" />
    <circle cx="11.5" cy="11.5" r="1.5" />
  </svg>
);

export const IconCopy = (p: P) => (
  <svg {...base(p)}>
    <rect x="5.5" y="5.5" width="8" height="8" rx="1.75" />
    <path d="M10.5 5.5V4.25c0-.97-.78-1.75-1.75-1.75h-4.5c-.97 0-1.75.78-1.75 1.75v4.5c0 .97.78 1.75 1.75 1.75H5.5" />
  </svg>
);
export const IconMore = (p: P) => (
  <svg {...base(p)}>
    <circle cx="3.75" cy="8" r="0.6" fill="currentColor" />
    <circle cx="8" cy="8" r="0.6" fill="currentColor" />
    <circle cx="12.25" cy="8" r="0.6" fill="currentColor" />
  </svg>
);
export const IconCheck = (p: P) => (
  <svg {...base(p)}>
    <path d="M3.25 8.5 6.5 11.5l6.25-7" />
  </svg>
);

/** Family marks, shared by the map, the legend and the run index. */
export const IconTable = (p: P) => (
  <svg {...base(p)}>
    <rect x="2.5" y="3" width="11" height="10" rx="1.5" />
    <path d="M2.5 6.5h11M2.5 9.75h11M6.5 6.5V13" />
  </svg>
);
export const IconVideo = (p: P) => (
  <svg {...base(p)}>
    <rect x="2" y="4" width="8.5" height="8" rx="1.5" />
    <path d="m10.5 7 3.5-2v6l-3.5-2" />
  </svg>
);
export function FamilyMark({ family, size = 10 }: { family: string; size?: number }) {
  const s = size;
  const h = s / 2;
  const fill = "currentColor";
  switch (family) {
    case "flex":
      return (
        <svg width={s} height={s} viewBox={`0 0 ${s} ${s}`} aria-hidden="true">
          <circle cx={h} cy={h} r={h - 0.5} fill={fill} />
        </svg>
      );
    case "acme_normal":
      return (
        <svg width={s} height={s} viewBox={`0 0 ${s} ${s}`} aria-hidden="true">
          <path d={`M${h} 0.5 L${s - 0.5} ${s - 0.5} L0.5 ${s - 0.5}Z`} fill={fill} />
        </svg>
      );
    case "acme_inverse":
      return (
        <svg width={s} height={s} viewBox={`0 0 ${s} ${s}`} aria-hidden="true">
          <path d={`M0.5 0.5 L${s - 0.5} 0.5 L${h} ${s - 0.5}Z`} fill={fill} />
        </svg>
      );
    case "bass":
      return (
        <svg width={s} height={s} viewBox={`0 0 ${s} ${s}`} aria-hidden="true">
          <rect x="1" y="1" width={s - 2} height={s - 2} fill={fill} />
        </svg>
      );
    default:
      return (
        <svg width={s} height={s} viewBox={`0 0 ${s} ${s}`} aria-hidden="true">
          <path d={`M${h} 0.5 L${s - 0.5} ${h} L${h} ${s - 0.5} L0.5 ${h}Z`} fill={fill} />
        </svg>
      );
  }
}

export const FAMILY_LABEL: Record<string, string> = {
  flex: "FLEX droplets",
  acme_normal: "ACME normal flames",
  acme_inverse: "ACME inverse flames",
  bass: "BASS solids",
  other: "Other",
};
