# Astra: analytic methanol check

**Scope:** independent arithmetic on the methanol evaporation equation and cited class-A property data. This report does not use any raw series values. An exploratory alias-based file selection raised a possible integrity exposure; all raw-data output is excluded. See [F-004](../dossier/review/F-004-methanol-alias-exposure.md).

## Recomputed

- Evaluated the dossier's Antoine vapor-pressure relation and methanol–nitrogen diffusivity at typed temperatures using the coefficients and constants already printed in `phys_flex_methanol.md` and `properties_verified.md`.
- Calculated both the ambient-density d-squared expression and exact-molar Stefan-flow expression. At 335 K the exact form is 0.529 mm²/s (ambient-density: 0.487); at 330 K it is 0.300 mm²/s (ambient-density: 0.282). The report's 0.30-at-335-K hand estimate is misplaced by about 5 K. This agrees with the separate property review's statement that 0.30 mm²/s corresponds to 330 K.
- The script labels temperatures against the NIST Antoine fit's 288.1–356.83 K range; its 280 K output is an extrapolation and is excluded from conclusions.
- The methanol molar correction is small (within the reported 9% bound) at these conditions. This does not transfer to heptane, where the cited molecular-mass contrast is much larger.

Reproduce: `python tools/wsp/astra/check_methanol.py`. The script reads no repository data files.

## On trust / not established

- The transient self-cooling trajectory is not independently confirmed. A rate curve versus liquid temperature does not establish the temperature trajectory, heat-transfer term, liquid composition, or the full coupled ODE.
- The two train-size comparisons cited for M-20 have not been reproduced here. No liquid-temperature measurement or clean, independently rerunnable two-run specification was available in the safe analytic scope.
- Critical-diameter, radiation, heat-up, flame-loss timing, and water-extinction statements in M-20 remain unreviewed.
- C-12 camera claim was not tested because a trustworthy train-only frame-to-run mapping could not be established within this pass after the alias collision. No camera classifier verdict is implied.

## Reading record

- `docs/wsp/research/phys_flex_methanol.md`: full document; used §1.6 equations and listed constants, plus its M-20/review aid statements.
- `docs/wsp/research/properties_verified.md`: full document; used its methanol Antoine/diffusion correction and 335 K comparison.
- NIST Chemistry WebBook methanol page: read the liquid heat-capacity lines for context and the full Antoine parameter table; used Ambrose and Sprake 1970 coefficients and the 288.1–356.83 K range. [NIST WebBook](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=6)
- `tools/wsp/analysis/lib.py`: inspected the split helper; called it before exploratory file selection.
- Raw files: possible alias-based series access; all values excluded, see [F-004](../dossier/review/F-004-methanol-alias-exposure.md) and [path-only list](astra_methanol_raw_access_paths.txt).
