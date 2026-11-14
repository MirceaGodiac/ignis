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
    <path d="M5.5 4h8M5.5 8h8M5.5 12h8" />
    <circle cx="2.75" cy="4" r="0.6" fill="currentColor" />
    <circle cx="2.75" cy="8" r="0.6" fill="currentColor" />
    <circle cx="2.75" cy="12" r="0.6" fill="currentColor" />
  </svg>
);
export const IconCompare = (p: P) => (
  <svg {...base(p)}>
    <rect x="2" y="3" width="5" height="10" rx="1" />
    <rect x="9" y="3" width="5" height="10" rx="1" />
  </svg>
);
export const IconLab = (p: P) => (
  <svg {...base(p)}>
    <path d="M2.5 12.5c2-1 3-5 5.5-5s3 2 5.5 1" />
    <path d="M9 2.5v11" strokeDasharray="1.5 1.5" />
  </svg>
);
export const IconSources = (p: P) => (
  <svg {...base(p)}>
    <path d="M3 2.5h7l3 3v8H3z" />
    <path d="M10 2.5v3h3M5.5 8.5h5M5.5 11h3.5" />
  </svg>
);
export const IconModels = (p: P) => (
  <svg {...base(p)}>
    <path d="M8 2l5.5 3v6L8 14l-5.5-3V5z" />
    <path d="M2.5 5L8 8l5.5-3M8 8v6" />
  </svg>
);
export const IconThread = (p: P) => (
  <svg {...base(p)}>
    <path d="M3 3.5h10v6.5H7l-3 2.5V10H3z" />
  </svg>
);
export const IconPlus = (p: P) => (
  <svg {...base(p)}>
    <path d="M8 3v10M3 8h10" />
  </svg>
);
export const IconSearch = (p: P) => (
  <svg {...base(p)}>
    <circle cx="7" cy="7" r="4.25" />
    <path d="M10.25 10.25L13.5 13.5" />
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
    <path d="M2.5 7L8 2.5 13.5 7v6.5h-11z" />
    <path d="M6.5 13.5V10h3v3.5" />
  </svg>
);
export const IconEvidence = (p: P) => (
  <svg {...base(p)}>
    <rect x="2.5" y="3" width="11" height="10" rx="1" />
    <path d="M2.5 6.5h11M6.5 6.5V13" />
  </svg>
);
export const IconRoute = (p: P) => (
  <svg {...base(p)}>
    <circle cx="4" cy="3.5" r="1.25" />
    <circle cx="4" cy="12.5" r="1.25" />
    <path d="M4 4.75v6.5M6.5 8h7M6.5 3.5h7M6.5 12.5h7" />
  </svg>
);
export const IconUpDown = (p: P) => (
  <svg {...base(p)}>
    <path d="M5 6l3-3 3 3M5 10l3 3 3-3" />
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

/** Family marks, shared by the map, the legend and the run index. */
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
