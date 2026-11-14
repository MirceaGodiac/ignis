// T9: how a ConditionsTable cell reads in the matrix. The source's own text and flags stay visible:
// a gap that quotes the table ("21; 26.5; 34") shows that text, never a guessed value. A flag that
// changes how the value reads (approximate, burner-stream O2, composition off) is on the cell; a
// provenance note (test sheet agrees, header reconciled) is in the cell's detail. No flag is dropped.

/** The text a gap's reason quotes from the source table, if any. */
export function quotedText(reason: string): string | null {
  const m = reason.match(/"([^"]+)"|“([^”]+)”/);
  return m ? (m[1] ?? m[2]).trim() || null : null;
}

export interface FlagTag {
  label: string;
  title: string;
  /** Shown on the cell (changes how the value reads); otherwise only in the detail. */
  inline: boolean;
  /** The value is not the test atmosphere; it must not be read as one. */
  notAtmosphere: boolean;
}

/** Flags from the R9a producers (ignis-data families and conditions); see their `flags` calls. */
export function flagTag(f: string): FlagTag {
  if (f === "approximate") return { label: "~", title: "Approximate: the table writes this value with ~", inline: true, notAtmosphere: false };
  if (f === "approximate_per_header") return { label: "~", title: "Approximate: the column header says the values are approximate", inline: true, notAtmosphere: false };
  if (f === "oxidizer_stream:burner") return { label: "burner stream", title: "O₂ of the burner stream (inverse flame), not the ambient atmosphere", inline: true, notAtmosphere: true };
  const sum = f.match(/^composition_sum=(.+)$/);
  if (sum) return { label: `Σ ${sum[1]}`, title: `The gas composition of this run sums to ${sum[1]}, not 1`, inline: true, notAtmosphere: false };
  if (f === "testsheet_agrees") return { label: "test sheet agrees", title: "The run's own test sheet gives the same value as the summary table", inline: false, notAtmosphere: false };
  const hdr = f.match(/^original_header:(.*)$/);
  if (hdr) return { label: "header reconciled", title: `The table's column header was written "${hdr[1]}"; it was matched to this condition`, inline: false, notAtmosphere: false };
  if (f === "volume_percent_as_mole_fraction") return { label: "vol % as mole fraction", title: "The table gives volume percent; it is read as a mole fraction (ideal gas)", inline: false, notAtmosphere: false };
  return { label: f.replace(/_/g, " "), title: f, inline: false, notAtmosphere: false };
}

export const flagTags = (flags: readonly string[] | null | undefined): FlagTag[] => (flags ?? []).map(flagTag);
