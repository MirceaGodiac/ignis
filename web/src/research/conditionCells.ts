// T9: how a ConditionsTable cell reads in the matrix. The source's own text and flags stay visible:
// a gap that quotes the table ("21; 26.5; 34") shows that text, never a guessed value, and a flag that
// changes the meaning (approximate, burner-stream O2) is on the cell, not only in the detail.

/** The text a gap's reason quotes from the source table, if any. */
export function quotedText(reason: string): string | null {
  const m = reason.match(/"([^"]+)"|“([^”]+)”/);
  return m ? (m[1] ?? m[2]).trim() || null : null;
}

export interface FlagTag {
  label: string;
  title: string;
  /** The value is not the test atmosphere; it must not be read as one. */
  notAtmosphere: boolean;
}

const KNOWN: Record<string, Omit<FlagTag, "notAtmosphere"> & { notAtmosphere?: boolean }> = {
  approximate: { label: "~", title: "Approximate: the table writes this value with ~" },
  "oxidizer_stream:burner": { label: "burner stream", title: "O₂ of the burner stream (inverse flame), not the ambient atmosphere", notAtmosphere: true },
};

/** Flags as short cell tags; an unknown flag is shown as its own name, never dropped. */
export function flagTags(flags: readonly string[] | null | undefined): FlagTag[] {
  return (flags ?? []).map((f) => {
    const k = KNOWN[f];
    return k ? { label: k.label, title: k.title, notAtmosphere: !!k.notAtmosphere } : { label: f.replace(/_/g, " "), title: f, notAtmosphere: false };
  });
}
