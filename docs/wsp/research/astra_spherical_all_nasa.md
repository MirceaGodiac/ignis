# Astra: spherical gas burners — boundary conditions before correlations

2026-11-14. Theory/methods research for the nineteen-investigation programme. No numerical candidate, property coefficient, fitted constant, implementation, experimental measurement or score is supplied. Symbols below are physical variables; algebraic factors follow from geometry. **S** marks source-supported facts, **D** an independent derivation under stated assumptions, **P** a proposed requirement/check.

## Exposure, source screening and applicability

This is an exposed theory analyst, not a clean-room W0 author. The historical aggregate handoff exposure recorded in `astra_all_nasa_dimensional_basis.md` remains applicable. A later generic spherical-theory search, despite exclusions for NASA/ACME/CFI/ISS, returned a publisher search snippet with a cited-paper expansion containing an ISS spherical-flame cohort summary, including outcome ranges and qualitative trends. No linked result paper was opened. No exposed values or trends are reproduced or used to select a law here. Broad flame-paper searches were then stopped. A separate generic BRE query also returned qualitative result-bearing snippets; its exposure is recorded in the BRE note. These notes must be screened into a restricted symbolic source/specification package before being given to fresh numerical authors.

No atlas records, protected data, per-run validation, test partition, result dossiers or existing model weights were read. All new-investigation outcomes remain closed under the instruction supplied to this analyst. Mapping CFI-G or s-Flame to a spherical reduction is provisional until safe apparatus metadata confirms geometry, feed and observation definitions. This note does not qualify either investigation or replace the earlier five-model checkpoint.

The substantial new primary teaching sources are Moshe Matalon's archived Summer 2013 derivations of diffusion flames and spherical liquid-interface conservation, plus Ronald Hanson's archived emission/absorption methods. These are author-produced theory/methods notes, not the original historical discovery papers. The complete selected theory portions were opened as described in the ledger. No experimental trend is used to choose model structure.

## 1. A sphere is a symmetry assumption, not a combustion regime

**D.** Spherical symmetry means all gas states depend on radius and time and the velocity is radial. It requires the boundary and forcing to have that symmetry: uniform porous injection, effectively symmetric ambient composition, negligible transverse forced flow, no important support wake, no dominant nearby wall and negligible gravity-induced distortion on the relevant scale. A spherical burner does not guarantee a spherical flame, nor does spherical symmetry imply steady state or unity Lewis number.

A radial model's inventory variables can include gas temperature/composition, burner solid temperature, feed-line/plenum state and an observation state. The gas core must conserve mass, elements and energy. For a steady gas domain outside radius `a`, continuity gives

\[
4\pi r^2\rho u=\dot m,\qquad M=\dot m/(4\pi).
\]

Here `m_dot` is total gas injection, including diluent, not necessarily fuel mass flow. This distinction matters for changing dilution at fixed total feed. Composition, gas temperature and pressure are required to interpret an actual volume flow; a stated standard volume flow additionally requires its reference state and calibration gas.

**S.** Matalon's [Lecture 12](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-12.pdf), pp. 3–9, derives spherical gas conservation and interface flux conditions. It is a droplet derivation; the point transferred here is conservation at an interface, not its boiling-temperature or vaporization boundary.

**P.** For a gas-fed burner, use prescribed total species fluxes and a separate thermal boundary. Do not copy a liquid droplet's unknown evaporation rate, latent heat or surface-equilibrium condition merely because both geometries are spherical. A prescribed burner feed and an evaporation rate determined by heat balance are different mathematical problems.

## 2. Flux-fed and concentration-fixed boundaries give different answers

**D.** Let `Z` be an injected-stream mixture fraction, one in the supplied stream and zero in the ambient. Assume steady radial flow, equal scalar diffusivities, constant `rho D`, no reaction source for `Z`, no surface chemistry and an infinite ambient. Write the radial outward total scalar flux as

\[
J_Z(r)=MZ-\rho D r^2\frac{dZ}{dr}.
\]

The source-free equation is `dJ_Z/dr=0`. A physically prescribed incoming stream with `Z_feed=1` gives `J_Z(a)=M`. Define `A=M/(rho D)`, a length. Then

\[
r^2 Z'=A(Z-1),\qquad Z(\infty)=0,
\]

whose solution is

\[
Z(r)=1-\exp(-A/r).
\]

Substitution checks both conservation and the boundary flux. In particular,

\[
Z(a)=1-\exp(-A/a)\ne Z_{feed}
\]

in general. A composition in the feed line is not automatically the gas-side surface concentration when back diffusion contributes to the boundary flux. For a porous body with internal reaction, multicomponent diffusion or nonuniform injection, the simple flux boundary must itself be replaced by a justified porous-interface model.

The problem with **prescribed gas-side concentration** `Z(a)=Z_b` instead gives

\[
Z(r)=Z_b\frac{1-\exp(-A/r)}{1-\exp(-A/a)}.
\]

Its total scalar supply is `M Z_b/[1-exp(-A/a)]`; that supply need not equal `M`. As `A/a` tends to zero, the concentration-fixed solution tends to `Z_b a/r`. This is the diffusion-from-a-maintained-reservoir limit used in the earlier dimensional note. It is not the zero-feed limit of a fixed-mixture-flux burner. The two boundary models must never be interchanged silently.

For variable `rho D`, the same flux argument gives

\[
Z(r)=1-\exp\left[-\int_r^\infty \frac{M}{\rho(s)D(s)s^2}\,ds\right],
\]

only if the scalar-flux representation remains valid. The integral is a symbolic resistance relation, not permission to insert a numerical solver or to infer an effective diffusivity from a measured flame radius.

## 3. What a flame-sheet limit can and cannot produce

**D.** For two streams containing fuel and oxygen separately, define `s` as oxygen mass consumed per fuel mass, and let their feed mass fractions be `Y_F,feed` and `Y_O,ambient`. In the normal configuration,

\[
Z_{st}=\frac{Y_{O,ambient}}{sY_{F,feed}+Y_{O,ambient}}.
\]

Under the infinitely fast, equal-diffusivity reaction-sheet approximation, the flux-fed solution places the reaction sheet at

\[
r_f=\frac{A}{-\ln(1-Z_{st})}.
\]

An exterior sheet requires `r_f>a`; otherwise the assumed detached exterior sheet does not exist. That is a failure of this reduced branch, not a general proof of physical extinction or absence of surface-attached reaction.

For an inverse configuration, one may retain the fuel-stream convention for `Z` and exchange the actual boundary streams. Then a pure oxidizer-stream injection has `Z(r)=exp(-A/r)` and `r_f=A/[-ln Z_st]`, with the appropriate stream concentrations in `Z_st`. Alternatively use injected-stream fraction throughout and redefine its stoichiometric value. Both choices must give the same physical surface after a consistent change of variables. An unlabelled normal/inverse indicator is insufficient.

The derivative gives an analytical scalar-mixing check,

\[
\chi(r)=2D|Z'|^2=\frac{2DA^2}{r^4}\exp(-2A/r),
\]

using the local `D` only in this constant-property limit. The result identifies how flow, radius and mixing are related in that ideal problem. It does **not** supply a universal critical `chi`, a chemical timescale or an extinction-time distribution.

**S.** [Matalon Lecture 10](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-10.pdf) explicitly separates the fast-chemistry sheet, finite reaction-zone structure, nonunity diffusion and scalar-dissipation formulations. The note uses these distinctions, not its numerical example thresholds.

**D/P.** A conserved mixture fraction describes mixing; it does not close temperature when radiation, different diffusivities or solid heat transfer matter. A scalar solution cannot determine low-temperature reaction branches, radical leakage, soot, luminous emission or finite-rate extinction. An extinction-capable claim needs a named mechanism or justified reduction, its uncertainty and the branch/initial-state definition. No such numerical mechanism is authored here.

## 4. Transience: local steady fields versus finite material inventories

**D.** The time-dependent scalar balance is

\[
\partial_t(\rho Z)+\frac1{r^2}\partial_r\left[r^2\rho uZ-r^2\rho D\partial_rZ\right]=0.
\]

During heating, `rho` changes and the instantaneous `r² rho u` need not be radially constant: total continuity contains storage. Enforcing its steady value everywhere during ignition can discard expansion-induced flow.

The steady infinite-domain flux-fed solution decays as `A/r`. Its integrated scalar inventory diverges as the outer radius grows. It is therefore a local long-time limit, not a finite-inventory state established throughout an infinite domain at a finite time. A finite chamber must supply an outer boundary and either exchange flow or accumulating composition/pressure. Quasi-steady use requires relaxation to be short relative to feed, burner-temperature and ambient changes in the region that influences the target.

The burner itself stores heat. A schematic balance is

\[
C_b\dot T_b=Q_{gas\to b}+Q_{rad\to b}-Q_{support}-\dot m[h_{exit}(T_b)-h_{feed}].
\]

Here `Q_gas->b` denotes nonradiative gas-to-surface heat transfer, and `Q_rad->b` is net radiative exchange; neither includes the other. `C_b` is a lumped heat capacity only if internal gradients are negligible. Flame-to-burner transfer, support conduction and feed preheating must not be represented twice. Unknown burner temperature and ignition history cannot generally be deduced uniquely from flame-radius images.

An outer-length increase changes mixing relaxation, confinement and radiative exchange together. Increasing a model's integration horizon does not create the missing chamber boundary. Gravity enters through vector momentum and can destroy radial symmetry; no intermediate-gravity prediction is certified by the radial law.

## 5. Radiation, energy closure and observation operators

**D.** Even with complete chemical consumption, only part of chemical energy returns to the burner. It is partitioned between enthalpy flux, net radiation to boundaries, solid transfer and transient storage. A constant radiated fraction is a closure requiring independent justification, not a conservation identity.

Let `j_lambda(r)` be isotropic emission per path length and solid angle under a stated convention. For an optically thin spherical field, a line-of-sight signal at projected impact parameter `b` has the Abel-type geometrical form

\[
I_\lambda(b)=2\int_b^{R}\frac{j_\lambda(r)r}{\sqrt{r^2-b^2}}\,dr.
\]

At finite absorption, integrate the radiative-transfer equation along each ray instead. Camera response then integrates wavelength, pixel area and exposure time and applies the optical point-spread function and calibration. Thus the brightest projected ring, a segmented visible boundary, peak temperature, peak reaction rate and `Z=Z_st` need not coincide. Camera thresholds belong to the measurement method; they are not flame-physics constants.

**S.** [Hanson, Lecture 5, Quantitative Emission/Absorption](https://cefrc.princeton.edu/document/162), slides 1–6, develops emission plus absorption, optical depth and the assumptions behind Beer–Lambert attenuation. Its distinction between emitted and transmitted signals supports separating these observation operators. Excited-state emission is not assumed to be a direct temperature or heat-release measurement.

**P.** A radius endpoint must specify which contour, calibration, projection model, bandpass and time origin it uses. Radiometers/photomultipliers need spectral and angular response, field of view, offset, saturation and temporal response. A signal falling below detection is a measured optical event unless an independent event definition establishes chemical extinction. A finite recording with no observed event supplies censoring, not a burn time equal to the last frame.

## 6. Property requirements and identifiability

| Needed input | Why / what cannot substitute |
|---|---|
| Full stream compositions and basis | Oxygen percentage alone cannot distinguish diluent effects or determine molecular mass |
| `rho(T,p,Y)`, `cp/h(T,Y)`, viscosity, multicomponent diffusivities, conductivity | One fitted effective diffusion constant cannot identify independent heat loss and chemistry |
| Reaction mechanism/reduction and its domain | Stoichiometry fixes consumption ratios, not rates or radical populations |
| Spectral gas/soot properties and wall radiation conditions | A visible flame size does not identify total radiative loss |
| Burner radius, permeability/injection uniformity, support and surface thermal data | Radius alone does not fix species or energy boundary conditions |
| Mass-flow history, meter convention/calibration, feed temperature | Commanded volume flow is not automatically delivered fuel mass flux |
| Ignition, initial burner/chamber state, outer volume and exchange | A stateful transient cannot be reconstructed from nominal steady setpoints alone |
| Instrument transfer functions | Raw intensity cannot independently identify temperature, composition and emissivity |

**D.** In the ideal radius law, the combination `m_dot/(rho D)` is identifiable from radius only if stoichiometry and the boundary model are known. Radius alone cannot separate flow calibration error from transport error. Adding more frames of the same unknown calibration does not resolve that structural ambiguity. A property entry inside its handbook range does not demonstrate that the entire coupled system lies inside an experimentally checked domain.

## 7. Prospective outcome-free verification

These checks are proposed, not run, and cannot establish NASA validity:

- Substitute the analytical scalar profiles into the differential and total-flux boundary equations; check dimensions and signs.
- Distinguish zero feed from fixed surface concentration explicitly. The two limits must not collapse into the same API meaning.
- Exchange inlet/ambient stream labels with a consistent `Z -> 1-Z` transformation and recover the same physical target.
- Enforce exterior-sheet admissibility; report unavailable branch rather than negative stand-off or a fabricated extinction event.
- Use finite-shell inventory conservation to check any transient discretization, including gas expansion and boundary exchange.
- Test shared thermochemistry/transport identity with the jet and droplet families at identical state.
- Propagate calibration and property covariance; near a missing/appearing branch report scenario uncertainty instead of a small symmetric error bar.
- Forward-project a synthetic shell through a declared optical operator; verify that measurement and physical contours remain distinct. Synthetic checks are measurement-method verification, never experimental evidence.

## Source/read ledger and version pins

Access date: 2026-11-14. Network PDFs were inspected in memory; no source files were added to the repository. Hashes identify the bytes actually fetched. PDF mathematical text extraction had missing minus signs/glyphs in places; no source equation was copied into a candidate. Symbolic formulas above were derived explicitly from the stated balances.

| Source | Actual reading extent / use | Pin |
|---|---|---|
| Matalon, Summer 2013, [Lecture 10](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-10.pdf) | Page-head screening all 29 pages; full extracted pp. 3–14, 17–20, 24–29; pp. 15–16 only partial due tool truncation. Generic theory only. Source facts summarized above, no numerical thresholds selected. | SHA-256 `3f139912cd10d7671b701a8fe5e8c4513446f35c9c1c6c1473d5ae01118fa67a` |
| Matalon, Summer 2013, [Lecture 12](https://cefrc.princeton.edu/sites/g/files/toruqf1071/files/Files/2013%20Lecture%20Notes/Matalon/Notes-Lecture-12.pdf) | Full extracted pp. 3–9: assumptions, jump conditions, spherical balances and limiting solutions; earlier web excerpt included pp. 1–2. Other pages not used. | SHA-256 `2fb1d345f189ecc8b061f9ce3b01ac7fa0a5f122f0b5679b101deb96d438389e` |
| Hanson, archived 2013 CEFRC [Lecture 5](https://cefrc.princeton.edu/document/162) | Full text of slides 1–6 returned by the radiative-transfer find; additional returned excerpts included slides 10–12 and 19–22, but only 1–6 support this note. | Archived lecture/URI; no byte hash obtained. This is not yet a frozen numerical-data input. |

Cooper's theoretical spherical-pressure report was screened by repository metadata only; its PDF returned HTTP 403 and was not read. The RWTH summer-school PDF returned 404. Neither is claimed as substantive evidence. Publisher snippets containing outcome summaries were excluded and logged above. No per-investigation apparatus manual was audited, no release was qualified, no model was executed, and no commit was made.
