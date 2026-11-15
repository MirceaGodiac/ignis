# F-003: methanol self-cooling law and property check

- **About:** M-20; new check of the self-cooling estimate in `docs/wsp/research/phys_flex_methanol.md` §1.6 and property inputs in `docs/wsp/research/properties_verified.md`.
- **Reviewer:** Astra (review agent), 14 November 2026
- **Verdict:** weakened
- **Severity:** model
- **Partitions touched while reviewing:** no experiment partition for the calculation; the required dossier already reports train results. No family weights were written.

## What the dossier says

M-20 says the unlit methanol evaporation was re-evaluated at two sizes, while the rest of the physics dossier remains on trust. Section 1.6 gives an estimated nitrogen evaporation rate of about 0.3 mm²/s at 335 K, falling with temperature; `properties_verified.md` says the 335 K value is closer to 0.5 mm²/s and that the hand-estimates can differ by up to 43%.

## What I did

I stated the comparison before calculating: evaluate the ambient-density and exact-molar forms of the d-squared evaporation law using the dossier's typed Antoine, diffusivity, molar-mass, and liquid-density constants at the temperatures printed in §1.6. No parameter was searched, fitted, or adjusted. I read the methanol reference property source cited by the dossier, the NIST Chemistry WebBook Antoine table (288.1–356.83 K, Ambrose and Sprake 1970), and used its listed coefficients. I wrote independent arithmetic in `tools/wsp/astra/check_methanol.py`.

The train-only helper was inspected. An exploratory data command then used the train IDs but matched files by any train-record alias; a known conflicting alias (`FLEX-238`) means I cannot certify the raw file identity. I therefore exclude every raw-series value from this finding. The possible access is disclosed separately in [F-004](F-004-methanol-alias-exposure.md).

## Evidence

The exact molar expression evaluated is

`K = 8 M_F c D_F / rho_l * ln(1 / (1 - x_s))`, with `x_s = p_sat(T)/p`.

The legacy expression uses ambient-gas density and `ln(1 + B_M)`, where `B_M = Y_s/(1-Y_s)` and `Y_s` is obtained from the same surface mole fraction. At 335 K in nitrogen, the exact calculation gives **0.529 mm²/s** (ambient-density form: 0.487); at 330 K it gives **0.300 mm²/s** (ambient-density form: 0.282). Both are within the Antoine source's 288.1–356.83 K range. The 280 K diagnostic in the script is explicitly labeled extrapolated and is not used for this conclusion. The reported 0.30 value belongs near 330 K, not 335 K; the separate source review's rounded 0.5-at-335-K statement is consistent. The exact-molar correction is 8.6% at 335 K, within the report's stated 9% bound.

Reproduce the arithmetic with `python tools/wsp/astra/check_methanol.py`. NIST source: [Chemistry WebBook, methanol Antoine parameters](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=6), lines listing the applicable equation and temperature range.

This checks only the instantaneous isothermal evaporation law at typed temperatures. It does not reproduce the coupled transient self-cooling ODE or the two train-window comparisons: liquid temperature is not observed, and the report does not provide the identities or raw measurements for its two-size comparison in a clean, independently rerunnable form. The measured cooling trajectory, size scaling, and general self-cooling mechanism therefore remain unconfirmed.

## What should change

Correct the 335 K hand estimate in the physics dossier from about 0.3 to about 0.5 mm²/s and keep the 330 K value distinct. Retain M-20's “on trust” status for the transient law and the broad train confrontation. Do not change a committed model or its weights from this review.

## Response

(The author answers here.)
