# Astra: property choices and their limits

14 November 2026. **Data-informed R1 and joint candidates, not zero-shot.** The main
author typed the new literal blocks before any of their train scores existed. The main
thread's runtime metadata was checked separately: `gpt-6-astra`, effort `max`. The two
independent research analyses explicitly used `gpt-6-astra`, high effort.

This is a selected-property check, not a certification of all 374 entries in Fable's
property report. External reference correlations are empirical science; their original
fits are disclosed. No IGNIS coefficient is estimated from them by a script. SI
conversions and rounded transcriptions are written by the author in `addition.wsp`.

## Liquid and gas properties used

| Block | Source and independent comparison | Decision / remaining uncertainty |
|---|---|---|
| Fuel identity, Antoine and latent heat | Main author reopened [NIST methanol](https://webbook.nist.gov/cgi/cbook.cgi?ID=C67561&Mask=4) and [NIST heptane](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=4), reading the molecular mass, Antoine and vaporization-enthalpy equations, coefficients and temperature intervals. Compared with the independently collected `astra_physical_properties_and_transport.md` and Fable property report | Source-native coefficients retained. Methanol requires separate `alpha=-0.31`, `beta=0.4241`; these are not interchangeable. Underlying original experiments were not reanalyzed |
| Methanol liquid density/Cp | The collector queried NIST saturated-liquid 300 K values; main author compared with the 300 K density table in [Xiang et al., viscosity correlation](https://www.nist.gov/system/files/documents/srd/jpcrd3520061597.pdf), and the whole one-page [Methanol Institute property sheet](https://methanol.org/uploads/publications/methanol-physical-properties.pdf), especially 25 C density/Cp/latent heat | Typed 784.5 kg/m3 and 2546 J/kg/K. Nearby-temperature agreement supports transcription, not constant-property accuracy up to 330 K; different compilations can share underlying measurements |
| Heptane liquid density/Cp | NIST fluid 300 K row was checked by the collector against the earlier report. Main author read the [NIST condensed-phase Cp table](https://webbook.nist.gov/cgi/cbook.cgi?ID=C142825&Mask=2): several 298.15 K determinations around 224.7 J/mol/K and the 300 K Van Miltenburg value 225.14 J/mol/K, versus the fluid row's 225.829 | Typed 678 kg/m3 and 2254 J/kg/K. Cp differences are small at this reference; the frozen approximation understates warm-liquid Cp. Density has a second-reader/source-table check, not a newly independent density experiment |
| Fuel diffusivity | Main author read [Tang et al. 2015](https://acp.copernicus.org/articles/15/5585/2015/acp-15-5585-2015.pdf), Tables 1–2 and evaluation/extrapolation discussion, after the separate property review: heptane 55±2 and methanol 126±11 Torr cm2/s at 298 K | Rounded SI values 7.24e-6 and 1.66e-5 m2/s at 101325 Pa/298 K. The shared 1.75 temperature exponent is an empirical approximation. Air/N2 carrier equivalence is an approximation. These values do not establish He/CO2 binary transport |
| N2 conductivity | Collector's NIST fluid 300 K value 0.02597 W/(m K), compared by the main author with Table V of [Lemmon & Jacobsen 2004](https://trc.nist.gov/refprop/FAQ/NAO.PDF): dilute-gas 300 K conductivity 25.9361 mW/(m K). Introduction and Table V read, not the full 49-page paper | Typed 0.02597. Gas Cp=1040 J/(kg K) is a rounded room-temperature N2 approximation, not vapor-mixture calorimetry; conductivity and Cp stay constant over the simulated film |

The web tool could not reopen the fluid CGI URLs during the main author's second pass.
That access failure does not become a successful independent query: exact fluid-row
provenance remains the collector's recorded query, with the separate comparisons above.
No uncertainty interval from these tables is claimed as calibrated prediction coverage.

Before release assembly, the main author also executed four fixed-state evaluations with
[CoolProp 8.0.0](https://coolprop.org/coolprop/HighLevelAPI.html), following its `PropsSI`
documentation and the [Air reference bibliography](https://coolprop.org/fluid_properties/fluids/Air.html).
The independent implementation gives methanol rho/Cp 784.590/2546.023, heptane
678.033/2247.490, and N2 k/Cp 0.0259687/1041.356 in SI at 300 K, 101325 Pa. Air at
1250 K gives nu=1.77234e-4 and alpha=2.39790e-4 m2/s. The typed values agree within
about 0.6%; **none was changed to the computed output**. This is a property-library
cross-check, not independent underlying experimental data and not weight generation.
Heptane Cp differs between the library/reference formulations. Exact receipts and
versions: [property cross-check](../feedback/astra_property_crosscheck.json); reproduce
with `python tools/wsp/astra/property_crosscheck.py` after the isolated dependency install
documented in that script. No dataset or model score is read by that command.

Both liquid kernels allow a **declared extrapolation**: outside the Antoine interval,
continue vapor pressure with boundary-matched constant-latent Clausius–Clapeyron; outside
the latent interval, continue that correlation below its critical temperature. The trace
records actual use. These are new closure assumptions, not extensions of NIST's validity
claims. Heptane's separate low-temperature Antoine fit leaves a gap before the selected
high-temperature fit; silently joining unrelated polynomials would be less honest.

`nitrogen_only=1` disables the whole thermal mechanism whenever He or CO2 is present.
Unused carrier slots repeat N2 constants for schema completeness and are explicitly
inactive; they are **not invented He/CO2 measurements**. The old law remains available.
The new law applies only following a future hot radiative transition. Unknown pre-cutoff
liquid history, methanol water activity and heptane cool-flame chemistry remain on the
predecessor. Initial temperatures 330/360 K, ambient 300 K, no residual cloud heat, and
the 0.8 surface-fraction cap are author judgments/closures. None is an inferred temperature.

## Fabric closure

The combustible loading is total cotton mass per sheet area, 0.1354 kg/m2, not that value
per face. The two-face multiplier appears once. The SIBAL 75% cotton/25% fiberglass,
18.05 mg/cm2 material description was independently found in NASA's
[Saffire VI methods report](https://ntrs.nasa.gov/api/citations/20240002981/downloads/Saffire-ICES-2024-ICES-365-accepted%20final.pdf?attachment=true).
Only the abstract/title and material-description search excerpt were read; a subsequent
full-text find timed out. Saffire VI on Cygnus is a different campaign from the modeled
BASS-II ISS burns; **no Saffire burn result was used as a held-out check**. Transfer of
material specification to the actual BASS specimen remains the inherited apparatus claim.

The oxygen/fuel ratio 1.185 follows cellulose stoichiometry; O2/N2 molar masses use the
standard reference values. Ambient density 1.15 kg/m3 is the inherited nominal Ferkul
environment. Hot-layer nu=1.78e-4 and alpha=2.41e-4 m2/s use the corrected 1250 K dry-air
reference in `properties_verified.md`, contrasted with Ferkul's original model-gas
1.49e-4/2.13e-4. These are **different reference closures**, not an exact single-state
gas. The main author did not rerun the full Kadoya/Shomate derivation, but independently
evaluated the alternative Lemmon/Jacobsen implementation through CoolProp above. Agreement
of hot transport values does not justify combining them with ambient density: the
hybrid capture-layer closure remains a modeling assumption, carried in the candidate card.

The new residence time 10 s is directly informed by the roughly 9.4–10.4 s train
length/speed ratios. It counts as one new train-read coefficient. Width 0.022 m is the
inherited nominal calibration, not new metrology. `L=v*tau` is a steady closure; the
visible-tip dynamics, fan calibration, specimen edges and quenching still need separate
models. A balance of oxygen supply does not identify the transient burn-progress profile.

## Committees

ACME's +/-10% flow and +/-5% pressure nodes and their equal masses are round sensitivity
judgments. They neither correct known erroneous table entries nor express certified sensor
precision. Composition/Tad/Zst stay fixed. Weights describe uncertainty **at cutoff**;
no second, implicit survival conditioning is performed.

Joint weights are fixed equal-author statements before scores. Droplet quantiles come from
the pooled diameter CDF, not averaged quantiles; ACME mixes complete competing-risk curves.
BASS mixes full position distributions. For BASS events, both speed scenarios use the new
first-event mapping documented in F-007, so this is not a mixture of the old artifact's
incoherent event rows. Shared inherited tensors keep their original train-informed exposure.
