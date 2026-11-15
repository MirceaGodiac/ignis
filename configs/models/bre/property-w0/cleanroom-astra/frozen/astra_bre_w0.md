# BRE property W0 author record

Author: OpenAI `gpt-6-astra`, fresh BRE author, 2026-11-14. This is a zero-shot numerical
authorship record. Structural/compiler corrections and synthetic conservation checks do not
provide outcome-driven coefficient feedback. The coordinator owns integration and any release.

## Scope and interpretation

The deliverable is a conditional forward for an emulated phase-changing material and a separate
real apparatus thermal node. It has one handwritten scientific constant, no learned coefficients,
no fitting, no regression, no scan, no parameter search, no random initialization and no runtime
optimization. It does not reconstruct a burning-rate response from an imposed feed and call that
an independent measurement. No NASA data, NASA publication, development result, test partition,
prior release, prior candidate or other family's author notes was intentionally or incidentally read.

The existing shared `Quantity::Temperature` is exactly applicable to the **real apparatus node
temperature in kelvin**. `Family::Other` is used because BRE has no dedicated family variant.
Neither family nor quantity is aliased to an unrelated observable. Virtual gasification, heat flux,
heat release and ledger values stay internal scientific quantities. Publishing heat-flux/heat-release
series as standard targets would require a reviewed shared-contract extension. No release has
been assembled here, and no service/runtime integration is claimed.

This implementation is usable as a conditional calculation only when all of the following hold:

- The real node is approximately isothermal and has the supplied constant thermal capacity,
  exposed radiating area, emissivity and linear loss conductance over the declared domain.
- Future external commands/power and environment are independent scenario inputs. A first-order
  fuel actuator with independently supplied positive lag and maximum flow is applicable, and its
  initial delivered flow is known. There is no implicit instantaneous actuator.
- An explicit quasi-steady, well-mixed combustion assumption and supplied conversion fraction,
  oxygen requirement, heat of combustion and absorbed chemical-heat fraction apply. These inputs
  are not inferred from NASA behavior. Ignition, flame survival, finite-rate transport and kinetics
  cannot be established by stoichiometry.
- For virtual demand, a planar constant-area, lumped material obeys the explicitly asserted
  equilibrium phase-change closure. Its remaining mass, initial temperature, heat capacity,
  phase temperature and phase enthalpy are supplied. This is not a general model of pyrolysis,
  charring, multistep decomposition, melting/freezing or shrinking exposed area.
- A future absorbed heat-flux schedule for the virtual material is supplied independently. This
  might be an explicit scenario or an independently justified closure. Measured future flux is a
  conditional replay input, not evidence of autonomous forecast skill. The kernel never derives
  that flux from its own future predicted feed or silently equates it to chemical power.
- Temperatures and environment stay inside all supplied constant-property domains. The caller
  supplies step size and numerical audit tolerances. Exhaustion and unstable explicit steps stop
  the calculation; they are not clipped, extrapolated or turned into fabricated zero values.

## Symbolic model, established before coefficient transcription

All fluxes below refer to a specified interval of duration `dt` and named experimental time origin.
There are two distinct balances. The virtual material consumes **virtual** energy; its emulated
mass loss commands real feed only in the explicitly selected virtual-demand mode. No virtual
energy or virtual material mass is subtracted from the real apparatus balance.

For the virtual material, let `M`, `T_v`, `c_v`, `T_ref`, `T_p`, `L`, and `A_v` denote remaining mass,
temperature, specific heat capacity, sensible reference temperature, phase temperature, specific
phase-change enthalpy, and exposed area. Its sensible stored energy is

```
U_v = M c_v (T_v - T_ref)
P_loss,v = K_v (T_v - T_amb) + epsilon_v sigma A_v (T_v^4 - T_amb^4)
E = [A_v q_abs,v - P_loss,v(T_v at interval start)] dt
E_warm = M c_v (T_p - T_v)
```

If `E <= E_warm`, mass is unchanged and `T_v' = T_v + E/(M c_v)`. Otherwise,
`dm = (E-E_warm)/L`, `M' = M-dm`, and `T_v'=T_p`. This analytical phase split assumes the
specified net source is constant over the explicit step; temperature-dependent losses are
evaluated at its beginning. A step reaching exhaustion is refused. Departing vapor carries both
its separately declared phase enthalpy `L` and the sensible enthalpy of the departing material:

```
U_v' - U_v + dm c_v (T_p - T_ref) + dm L = E
```

The sensible export term matters: using only `M c dT` as the storage ledger would misreport energy
when mass leaves. The code records every term and the residual. A virtual-demand command is zero
during warming, then equals `(E/dt)/L` after the exact phase-change onset `E_warm/(E/dt)`. Its
interval mean `dm/dt` is reported separately and is **not** used to start fuel delivery before that
onset. The actuator integral is split at the known onset; no search is required. An external
open-loop command can instead be supplied. No future apparatus observation or apparatus
temperature feedback enters either command.

Let `u=min(r,F_max)` be the actuator-saturated command, `F0` the initial actual fuel flow and `tau`
the supplied lag. The real flow and its integrated delivered mass are

```
F(t) = u + (F0-u) exp(-t/tau)
m_del = u dt + (F0-u) tau [1-exp(-dt/tau)]
```

The code evaluates `1-exp(-x)` as `-expm1(-x)`. Let oxidizer total mass flow be `G`, its oxygen
mass fraction be `Y_O2`, oxygen required per reacted fuel mass be `s`, and the independently
supplied conversion fraction be `chi`. The oxygen-accessible fuel-flow cap is `B=G Y_O2/s`.

```
m_access = integral_0^dt min(F(t), B) dt
m_react = chi m_access
m_unburned = m_del - m_react
m_O2,in = G Y_O2 dt
m_O2,used = s m_react
m_O2,out = m_O2,in - m_O2,used
m_inert,out = G (1-Y_O2) dt
m_products = (1+s) m_react
```

Because actuator flow is monotonic within a constant-command interval, an oxygen-cap crossing
has an analytical time. The integral is split there, without a search or optimizer. Total input
fuel plus oxidizer mass equals unburned fuel, oxygen exhaust, inert exhaust and reaction products.
This is a bulk mass ledger, not a resolved product-species model. Oxygen starvation or zero
conversion can leave actual delivered fuel entirely unburned. Feed never proves combustion.

For the real thermal node with capacity `C_a`, absorbed chemical fraction `alpha`, and independently
specified external power `P_ext`:

```
E_chemical = H_c m_react
E_absorbed = alpha E_chemical
E_not_absorbed = (1-alpha) E_chemical
E_loss,a = [K_a(T_a-T_amb) + epsilon_a sigma A_a(T_a^4-T_amb^4)] dt
C_a (T_a'-T_a) = E_absorbed + P_ext dt - E_loss,a
```

`H_c` must specify fuel, LHV/HHV and product/reference states. It is a separate chemical source;
the real thermal capacity is a sensible capacity of the solid apparatus node. Gas-phase mass
flows do not cross that solid node. The unabsorbed chemical term includes all chemical energy
not deposited in this node; the code does not pretend to resolve exhaust enthalpy, radiation
directions or product dissociation separately. The supplied `alpha` embodies that missing closure.

Both thermal updates are explicit in the losses. The exact derivative of the radiative term gives
the monotonicity condition

```
dt [K + 4 epsilon sigma A T_max^3] / C <= 1
```

where `T_max` covers old/new node and ambient temperatures; virtual `C=M c` during warming.
The caller must reduce the step if it fails. This is a numerical admissibility condition, not an
experimental stability or safety claim. A smaller step may still be needed for physical accuracy;
there is no automatic local-error estimate or universal recommended time step.

## Parameter inventory and manual literal assignment

The only scientific tensor is a scalar named
`stefan_boltzmann_watt_per_m2_kelvin4`, manually assigned **`5.670374419e-8 W/(m2 K4)`**.
Its origin is NIST CODATA 2022's displayed Stefan–Boltzmann constant. The SI-derived constant is
exact; this finite decimal is truncated, and the tensor bounds
`[5.670374419e-8, 5.670374420e-8]` enclose the displayed decimal precision, not physical uncertainty.
The bounds are strictly increasing, as required by `ModelSpec::validate`. No calibration value,
controller gain, heat-transfer coefficient, kinetic constant or illustrative example parameter
was transferred from the read sources.

The following are **supplied physical inputs**, never hidden tensor weights or defaults:

| Group | Inputs and SI units | Interpretation/provenance obligation |
|---|---|---|
| Initial conditions | apparatus and virtual `T` (K), remaining virtual `M` (kg), actual delivered `F0` (kg/s), time (s) | Explicit initial state and named time origin; external measurement/scenario provenance |
| Environment | ambient `T` (K), pressure (Pa), oxygen mass fraction (1), whole oxidizer flow (kg/s), velocity (m/s) and gravity (m/s2) vectors | Coordinate frames match; pressure, oxygen, speed and gravity constrain supplied property domains |
| Virtual material | exposed area (m2), `c` (J/(kg K)), `L` (J/kg), `T_p,T_ref` (K), `K_v` (W/K), emissivity (1) | Constant-area lumped equilibrium phase-change assertion, common thermal reference identity |
| Real thermal node | capacity (J/K), area (m2), `K_a` (W/K), emissivity (1) | Independent geometry and thermal properties; not virtual mass/capacity |
| Combustion | `H_c` (J/kg), `alpha`, `chi`, oxygen requirement (kg/kg) | Independent composition/chemical/reference and mixing/kinetic/heat-return support |
| Actuator | `tau` (s), maximum flow (kg/s) | Independent apparatus determination; no controller tuning |
| Future forcing | absorbed virtual flux (W/m2), external real power (W), external fuel command (kg/s) or virtual-demand mode | Command/forcing is an input, never an independent prediction target |
| Optional threshold | real node temperature limit (K) | Supplied declared threshold only, not a universal hazard boundary |

Every `Property` carries its value, a closed SI-unit tag, constant-property domain, physical reference,
origin, source, locator, dependency identities and uncertainty (`Unquantified` or an absolute bound).
Only SI values are accepted: the retained value/unit is the raw supplied SI value/unit. A future
adapter handling non-SI source values must preserve originals and conversion provenance upstream.
Environment, initial state and forcing have provenance records. Property uncertainty is retained but
**not propagated**; the result has no confidence interval and reports unquantified model discrepancy.
Any future shared request contract must preserve these records rather than flattening away provenance.

Exact algebraic integers are separate from scientific assignments: `0` is an empty rate/inventory or
zero difference; `1` is the identity, fraction complement and unit slope; `4` is the derivative/power
of the Stefan–Boltzmann law. Vector dimension `3` and array indices are structure. The exponential
and logarithm implement the analytical first-order solution. No empirical coefficients are hidden
in these operations.

Numerical controls are supplied step seconds, caller maximum step seconds, energy absolute audit
tolerance (J), mass absolute audit tolerance (kg) and relative arithmetic tolerance. Synthetic checks
use manually chosen fixture states, property values and tolerances for algebra only; these values
are not production properties, WSP weights or NASA evidence.

## UNKNOWN safety output

Every accepted step returns `SafetyObservable.status = "UNKNOWN"`. If a temperature limit is
supplied, the output includes `limit-T_apparatus` and distinguishes exceeded/within-declared-limit.
An exceeded limit is visible; a positive margin is never a claim of safety. Integrated unburned fuel
is also exposed. Missing ignition/extinction, transport/chemistry validation, spatial hot spots,
toxic species, flame spread and predictive uncertainty prevent a qualified safety conclusion.

## Read ledger and clean-room disclosures

No memory lookup, historical handoff, goals/dossier, existing author research, existing candidate
forward, NASA document/result/data, `runs/dev`, `holdout`, immutable model release or validation/test
outcome was opened. `AGENTS.md` in `D:/dev/SpaceApps-Ardealu` was read completely; the current explicit
clean-room restrictions superseded its historical-handoff links. Only these interface/algebra files
were read: root `Cargo.toml`, `crates/ignis-wsp/Cargo.toml`, `crates/ignis-core/Cargo.toml`,
`crates/ignis-core/src/contracts.rs`, `crates/ignis-core/src/contracts/wsp.rs`,
`crates/ignis-wsp/src/assemble.rs`, and `crates/ignis-wsp/src/physical_core.rs`.

The first read of the permitted `physical_core.rs` used the whole file; its existing **generic
synthetic algebra tests** appeared incidentally in a truncated tool response. No family response
data or fitted weights were present or used. Subsequent reads were restricted to interface sections.
The coordinator supplied the serialized `Origin::System` tag (`"system"`) from the shared interface.

| Source | Requested/read extent | Use and exposure |
|---|---|---|
| [NIST SP 250-65, Heat-Flux Sensor Calibration](https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication250-65.pdf) | Initial open returned PDF lines 0–105 (front matter, abstract, contents). Method request at line 126 returned lines 70–188 (PDF indices 6–9). A request at line 300 unexpectedly expanded to lines 273–403 (indices 12–15). Screenshot requests for physical pages 9–12 and 16–17 were mostly timeouts; returned screenshots were not used as visual evidence. | Method distinction: incident irradiance is not automatically absorbed material heat; a sensor output needs an applicable calibration. **Incidental exposure:** the auto-expanded excerpt included reference-sensor calibration Figure 5/Table 3 numerical regression results and Table 4 settings. This was disclosed to the coordinator immediately. These non-NASA calibration example numbers were not copied into any tensor, input, test or closure. |
| [Åström and Murray, Feedback Systems 2e, chapter 2, version 2020-07-24](https://www.cds.caltech.edu/~murray/books/AM08/pdf/fbs-principles_24Jul2020.pdf) | Initial open lines 0–104 (PDF indices 0–3); targeted open at line 430 returned lines 395–529 (indices 8–11). Search for “conservation”/“heat” found none. | Distinction among command, controller, plant response, initial conditions and dynamical stability. Illustrative gains/transfer-function coefficients appeared in excerpts but were not transferred. Conservation equations above are the author's declared control-volume model, not attributed to this chapter. |
| [NIST CODATA Stefan–Boltzmann value](https://physics.nist.gov/cgi-bin/cuu/Value?sigma%7Csearch_for=stefan+boltzmann) | Search result snippet contained displayed value `5.670374419... x 10^-8`, exact uncertainty and CODATA 2022 attribution. Direct query requests returned ill-formed/invalid-input responses; the alternate static URL was inaccessible. | Single manual scientific assignment from the primary NIST search result. The source page itself was not successfully opened. Other search-result snippets included secondary sites and other NIST listings; none were relied upon. |

`PatchMeta.exposure.sources` records stable documentary source IDs corresponding to the rows above:
`nist-sp250-65-2004-methods-and-incidental-calibration-excerpt`,
`astrom-murray-feedback-systems2-chapter2-20200724`, and
`nist-codata2022-stefan-boltzmann-primary-search-snippet`. These name the actual read extent; they do
not claim a full-document byte hash or a completed corpus ingestion. Dataset, development-feedback
and prior-candidate lists are empty. No claim is made that absolutely no numerical example was
visible; the claim is that no NASA outcome or other model candidate informed numerical authorship,
and no illustrative gain was adopted.

## Verification and reproducibility

Owned kernel: `crates/ignis-wsp/src/property_bre_w0.rs`. The isolated harness at
`configs/models/bre/property-w0/cleanroom-astra/verify.rs` loads only the permitted algebra,
assembler interface and new kernel. The filter `bre_checks` excludes pre-existing physical-core
tests; no historical outcomes are consumed. Tests parse the **actual shared** `ModelSpec`,
`PatchMeta`, literal operations and `HandwrittenWeightPatch`; no parallel invented schema is used.

Ten checks authored cover command versus lagged/integrated response, open-loop independence, exact
constant-power thermal response, analytic cooling transient and steady limit, virtual sensible and
phase-change energy with departing enthalpy, causal phase-onset command timing, distinct real/virtual energy, oxygen-cap crossings,
zero oxygen/zero conversion, incomplete conversion mass accounting, UNKNOWN safety with exceeded
thermal limit, absent closures, units, domains, nonfinite inputs and explicit-step monotonicity.

Reproduce (PowerShell, using a coordinator-approved build target):

```powershell
$env:CARGO_TARGET_DIR = 'runs/tooling/astra-target'
cargo test --offline --manifest-path configs/models/bre/property-w0/cleanroom-astra/Cargo.toml bre_checks -- --nocapture
rustfmt --edition 2021 --check crates/ignis-wsp/src/property_bre_w0.rs
rustfmt --edition 2021 --check --config skip_children=true configs/models/bre/property-w0/cleanroom-astra/verify.rs
```

Local author formatting completed successfully. The coordinator requested shared-cache build
coordination; no compiler was launched by the author. An initial low-disk-space estimate was
corrected by the coordinator to 169.5 GiB free; it is not a remaining blocker. The coordinator ran
the final causal-timing implementation and completed source metadata's 10 `bre_checks`:
**10 passed, 0 failed**. The exact executed command was
`cargo test --offline --manifest-path configs/models/bre/property-w0/cleanroom-astra/Cargo.toml bre_checks -- --nocapture`,
with `CARGO_TARGET_DIR=D:/dev/SpaceApps-Ardealu/runs/tooling/astra-target`.
The actual shared `ModelSpec` hash was
`ad5d6a0ad14e988ea25cd787ebc2ef5aad1f6c90da730f27dc113f7ea12c3cea`.
The final actual `HandwrittenWeightPatch` hash was
`d91806821199bb1aedcfbe941ef02b82e6328bcb7b3420640f3d701c403121fa`.
These contract hashes are distinct from raw file SHA256 hashes. The final author source,
specification, literal assignments, provenance metadata, verification harness and this dossier
are copied byte-for-byte to `configs/models/bre/property-w0/cleanroom-astra/frozen/`.
`freeze_receipt.json` records raw SHA256 values and actual verification evidence; it does not hash
itself. This author snapshot precedes any subsequent independent outcome-blind thermal review.

These are synthetic mathematical checks, not experiment validation, model qualification,
predictive accuracy evidence, uncertainty coverage or an observed safety envelope.
