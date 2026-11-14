// Display formatting for measured values. 6 significant digits keep recorded values such as
// 101325 Pa intact while trimming float noise; unit "1" (dimensionless) is not printed.
export const fmtNum = (v: number) => String(+v.toPrecision(6));

export const fmtMeasured = (v: number | null | undefined, unit?: string | null) =>
  v == null ? "—" : `${fmtNum(v)} ${unit && unit !== "1" ? unit : ""}`.trim();

// Gravity is stored as a ratio to standard gravity (unit_si "1").
export const fmtGravity = (v: number | null | undefined) => fmtMeasured(v, "g₀");
