# Independent thermal/property review of new Astra W0 candidates

Status: completed for the explicitly identified snapshots; outcome-blind independent physical reviewer, not numerical author. These are unqualified zero-shot candidates. No claim of clean-room numeric authorship is made after inspection. Scope is only the three new property candidates and their new reports, plus allowed common types/physical core if needed.

## Independent derivation before implementation inspection

This section was written before reading any candidate implementation or candidate report.

Let qdot denote heat delivered to a control volume, mdot positive outward, h_i(T)=h_f,i(T_ref)+integral(cp_i dT), and J_i the species diffusive mass flux relative to the mixture velocity. A consistent low-Mach gas enthalpy balance is

    partial_t(rho h) + div(rho u h) = -div(q_cond + sum_i(h_i J_i)) + Dp/Dt + other work,
    q_cond = -k grad(T),   sum_i J_i = 0.

If total species enthalpies include formation energy, reaction heat is already represented through species conversion and must not be added a second time. With sensible-only enthalpy, an explicit reaction source must instead use a reference-consistent reaction enthalpy. Unequal species diffusivities generally make sum(h_i J_i) nonzero: setting this term to zero is not justified by a nonunity-Lewis mass-transfer correction alone. cp, k and diffusion coefficient units are respectively J/(kg K), W/(m K), m^2/s; alpha=k/(rho cp), Le=alpha/D are dimensionless.

For a well-mixed liquid drop with m=4 pi rho_l R^3/3, d(m)/dt=-mdot_v. Neglecting pressure-volume corrections, a common enthalpy datum gives

    d(m h_l)/dt = Qdot_gas + Qdot_rad + Qdot_support - mdot_v h_v(T_s),
    m cp_l d(T_s)/dt = Qdot_gas + Qdot_rad + Qdot_support - mdot_v L_v(T_s),
    L_v(T_s)=h_v(T_s)-h_l(T_s).

Thus a latent term plus d(m cp T)/dt is not equivalent to a latent term plus m cp dT/dt unless the advected liquid-reference energy is restored. Constant density gives d(R^2)/dt=-mdot_v/(2 pi rho_l R). A quasi-steady spherical stagnant-film approximation gives mdot_v=4 pi R rho_g D ln(1+B_M), with B_M=(Y_v,s-Y_v,infinity)/(1-Y_v,s), only for a valid interface vapor fraction and its stated transport assumptions. Heat and mass closures must share surface state, pressure and flux conventions. Equilibrium p_sat(T_s) alone cannot permit Y_v,s>=1: boiling, pressure, saturation validity or a distinct regime must resolve that boundary. Flame-sheet chemistry may conserve elements while failing energy; both are required.

For a finite solid inventory containing virgin material, char and ash, let M_j and u_j(T) be the present mass and internal energy of each component. The identity is

    d(sum_j M_j u_j)/dt = sum_j M_j cp_j dT/dt + sum_j u_j dM_j/dt.

The right-hand energy balance includes absorbed external/radiative/chemical heat, conduction to unheated material and apparatus, outward gas enthalpy and reaction/phase enthalpy under one consistent datum. A loss law must not consume more virgin/char mass than remains, and heat capacity, exposed area, optical properties and layer thickness must follow the chosen inventory approximation. Surface recession, char production and burn-through cannot all be inferred from a single exponentially decaying temperature memory. Once no reactant or radiating inventory remains, corresponding heat-release and radiative terms must cease.

For apparatus node a, H_a(T_a,M_a)=sum_j M_aj h_aj(T_a); dH_a/dt=sum_b G_ab(T_b-T_a)+Qdot_ext-Qdot_loss plus carried enthalpy for changing inventory. G has W/K, H has J, C=dH/dT has J/K. Fixed-capacity apparatus gives C_a dT_a/dt=the same net heat. A control step may jump an imposed flux or a zero-inertia algebraic boundary; it cannot jump a finite-capacity temperature or stored energy. Phase exhaustion terminates latent consumption exactly at the available mass; post-event state must remain causal and nonnegative. Event detection/substepping must not turn future controls into present heating.

Domain requirements follow from each constitutive equation: positive absolute temperature and pressure; nonnegative time, masses and dimensions; bounded compositions; finite coefficients with their documented temperature/pressure/species applicability. A property citation must identify the actual numeric relation, reference datum, units and range; a generic theory citation alone cannot substantiate specific coefficients or coverage across different substances.

## Review findings and independently verified limits

### P2: accepted solid step can leave the declared boundary-property domain

Location: `property_solids_w0.rs` lines 311-316, 590, 617-618 (SHA-256 `fe59a4cb7e15069f1427aaf5cb8e7b7c70043bbb2984b1b308f18342ec8bbc06`). Boundary properties are checked only at the old surface-cell temperature. The heated intermediate and final temperatures are checked against material properties, but not exposed-face properties. The returned state can therefore already be outside the supplied heat-transfer/emissivity/absorbed-flux validity domain. The failure appears only on the following call, after the invalid result has been returned.

Independent actual-kernel counterexample: one inert 1 m3 cell, loading 1 kg/m3 and cp 1 J/(kg K), initially 1000 K. Material properties permit [1,2000] K; every boundary property permits only [999,1001] K. Front prescribed absorbed power is 20 W over 1 m2; convection/radiation/back input are zero. A 1 s step returns `Ok(())`, T=1020 K and energy residual exactly zero. A subsequent step returns `temperature outside supplied constant-property domain`. This demonstrates missing enforcement, not an inaccurate NASA prediction. Before accepting an update, the exposed-face properties need validity checks covering the surface temperatures used/reached during the split, including intermediate heating if reaction subsequently cools it. No coefficients need changing.

### Quantified numerical limitation: conservation and thermal CFL do not bound reaction-temperature error

Location: `property_solids_w0.rs` lines 484-523 and 586-620, same hash. This is a limitation of the openly declared first-order split, not a newly discovered formation-energy double count. `stable_step_s` constrains conductive/radiative transport only. It can return infinity for an insulated reactive cell while accepted reaction steps have large temperature error. The author report already disclaims an accuracy guarantee; the following independent exact solution quantifies why that disclaimer matters.

Use one insulated cell with inert mass 1 kg, reactant mass initially 1 kg, both cp=1 J/(kg K), T0=1000 K, constant reaction rate k=1/s (activation energy zero), and endothermic L=500 J/kg. These are invented algebra inputs, not material properties or candidate weights. Equal gas/parent cp is exactly the candidate's own declared restriction. With m(t)=exp(-t) kg and C(t)=1+m(t) J/K, the continuous governing balance is C dT=L dm. Therefore

    T(t) = 1000 + 500 ln[(1+exp(-t))/2].

At t=1 s the exact temperature is 810.057253479139 K. The actual candidate accepts one step and returns 768.941421369995 K: error -41.115832109144 K, although mass is exact and the reported energy residual is -5.684341886081e-14 J. The split removes all released-gas sensible energy at the frozen heated temperature and all latent energy against the final inventory, explaining the finite-step discrepancy. Repeating with 10,000 equal steps, without changing any physical property, gives 810.053532756288 K (error -0.003720722851 K); residual -9.094947017729e-13 J. The thermal stable-step bound is infinity in this example.

This refutes using accepted status, a zero ledger residual, or the transport CFL as evidence of temperature accuracy. It does not refute convergence to the declared continuous equations. An application needs a separately demonstrated temporal/spatial error budget, especially with temperature-sensitive kinetics; this review prescribes no fitted physical parameters or new weights.

### Droplet: reference and nonunity-Lewis audit

The physical heat balance is consistent within the narrow stated closure. `liquid_sensible_enthalpy_j` stores sum(m_i cp_l,i)(T-Tr); `sensible_vapor` is L_i,ref+cp_g(T-Tr); differentiating the variable inventory recovers C_l dT/dt=Q_in-sum(mdot_i L_i(T)), with L_i(T)=L_i,ref+(cp_g-cp_l,i)(T-Tr). Thus subtracting vapor enthalpy once is required; subtracting an additional latent term would double count it. No such extra subtraction was found. The integrated saturation law has dimensionless exponent W/R times specific-energy/temperature, and its derivative is W L(T)/(R T^2). Bubble-boundary and nonpositive-latent cases fail rather than being relabeled as extinction.

An independent general energy equation includes species diffusive enthalpy transport. In this candidate every gas species is assigned the same constant mass-specific cp. Consequently sum(J_i dh_i/dr)=cp (dT/dr) sum(J_i)=0. Different reference/formation enthalpies still enter the chemical source: the explicitly supplied **vapor** combustion heat gives the sheet jump; it must not be replaced by liquid heat of combustion without the matching conversion. Equal D is separately assumed for species transport. Nonunity Le alone does not invalidate this cancellation. Hence the stated nonunity-Le solution is not missing a nonzero sensible diffusion term under its common-cp restriction. Unequal species cp, differential diffusion, real multicomponent thermochemistry or independently inconsistent background composition do require another closure.

Writing A=mdot cp/(4 pi k), a=A/r_s, a_f=A/r_f, the inner gas solution has conserved sensible-energy constant e_i=T_s-Q_in/(mdot cp). Integrating the homogeneous sensible equation gives T=e_i+B exp(-A/r); the outer constant is e_o=e_i+q/cp. Continuity at the sheet and T(infinity)=T_inf yield Q_in=mdot[cp(T_inf-T_s)+q(expm1(a_f))]/expm1(a), agreeing with the implementation. This checks the governing flux independently of merely comparing its inner/outer test expressions. It does not qualify its gas properties or an actual flame.

Depletion is refused; neither completion through complete evaporation nor extinction is supported. Variable composition changes inventory and capacity. Motion/gravity are retained but zero is required unless explicit spherical extrapolation is selected. The constant-density, well-mixed, common-cp/common-D, ideal-solution, infinite-reservoir and already-established-sheet restrictions are substantial, openly documented coverage limits. There is no radiative/optical prediction, finite-rate ignition/extinction, support conduction or chamber depletion.

### BRE: apparatus separation, interfaces and causality

In the reviewed BRE kernel, virtual material energy and real apparatus energy are separate. The virtual ledger includes the outgoing material sensible enthalpy as well as phase enthalpy; this correctly reconciles variable remaining mass. Real gas does not advect through the solid apparatus node, so C_a delta(T) needs no fuel sensible-inventory correction. Heat of combustion and absorbed fraction form an externally supplied chemical-energy deposition closure, not a resolved gas enthalpy balance. Consistency of LHV/HHV, fuel and product state is a supplier obligation represented by free-text reference; the kernel cannot verify that chemistry from the string.

Virtual phase onset is computed within the explicitly frozen-loss step. The actuator uses zero command before that onset and the phase rate afterward; interval-mean gasification is not used before onset. A positive actuator lag gives continuous flow. Integrated oxygen-accessible fuel is split at its analytical crossing; conversion cannot invent oxygen or force burning from feed alone. Apparatus temperature remains a finite-capacity update. These are causal conditional calculations for the supplied held inputs. The exact onset claim is conditional on freezing thermal losses at the start; it is not exact for the nonlinear physical transient. No independent future heat-flux closure is supplied, and no autonomous fire feedback is established.

Exhaustion is deliberately rejected before a zero-mass virtual state. No finite exhaustion continuation or general pyrolysis/char/shape-change prediction is implemented. The exposed safety status remains UNKNOWN. A thermal-limit margin is an endpoint scalar, not assurance that an unresolved intra-step peak or a spatial hot spot stayed below the limit. Neither arithmetic tolerance nor the monotonicity bound certifies physical uncertainty or time accuracy.

## Actual property provenance and coverage

The new literal packets contain pi/R and numerical controls for droplets, R/sigma for solids, and sigma for BRE. The inspected literal ops agree with those inventories. They contain no experimentally determined fuel density, cp, conductivity, diffusivity, emissivity, reaction kinetic constants, heat-transfer correlation or apparatus response. Those properties are mandatory supplied inputs and the demonstrations use synthetic values. Theory citations explain conservation structure; they do not furnish material-specific qualified property tables or prediction evidence.

Provenance fields check source/locator/reference presence, some unit consistency and declared domains; they do not verify that the cited source supports the value, that the property identity is the modeled species, or that an independently asserted applicability range is true. Synthetic origin is expressible and must remain visible in any consuming artifact. No uncertainty propagation or quantitative model-discrepancy estimate was found. This review did not audit remote source bytes or qualify any real material/applicability claim. New candidates remain experimental conditional kernels, not qualified FLEX/ACME/BASS/BRE releases.

## Executed independent checks

Working directory: `D:/dev/SpaceApps-Ardealu`. Only standalone rustc was used; no Cargo compilation, tests from prior candidates, model execution on observations, fitting, parameter selection, or model-weight changes occurred.

```powershell
rustc --edition 2021 -A dead_code runs/tooling/astra-thermal-review/review.rs -o runs/tooling/astra-thermal-review/review.exe
& runs/tooling/astra-thermal-review/review.exe
rustc --edition 2021 -A dead_code runs/tooling/astra-thermal-review/domain.rs -o runs/tooling/astra-thermal-review/domain.exe
& runs/tooling/astra-thermal-review/domain.exe
```

Both compilations/executions succeeded. Results are the exact numbers recorded above. These harnesses import the actual new solid kernel by path and build their own synthetic scenario through its public API. They do not rerun or copy the author's tests. The one-step versus small-step calculation verifies a closed-form limit, not a search for a better physical parameter or a new numerical release.

## Read ledger, identities and drift

- Root `AGENTS.md` in D:/dev was read completely. No linked historical handoff, workboard, dossier, goals, old candidate, old forward, NASA report/payload/outcome, `runs/dev`, `holdout`, `artifacts/models`, validation/test partition, or memory registry was opened. The explicit task restricted historical reads. A filename search for the three candidates and physical_core also returned the filename `astra_physical_core_review.md`; its contents were not opened.
- The initial independent derivation above was written before any of the three candidate files/reports was opened.
- Whole-file reads were requested for the three new kernels and the two then-present reports. That batched response was truncated. Subsequent line-number reads covered the complete production droplet path (original lines 1-801), complete solid production path (1-768), and complete BRE production path (1-744); portions/all of new inline synthetic tests were incidentally visible in the initial and later reads. New author reports `astra_bre_w0.md`, `astra_solids_w0.md`, and later `astra_droplet_cleanroom.md` were read completely. No numeric authorship claim is made by this reviewer after those reads.
- Allowed packet reads: full droplet `source_inventory.json`; full solid `freeze.json`; full three literal-ops JSON files. Directory listings of the three cleanroom-astra packets and file hashes were inspected; listed Cargo/spec/meta/harness files were not thereby read. `physical_core.rs` was searched only for named type definitions and lines 101-126 were opened for provenance types. No physical-core computation was executed by the independent harnesses.
- Generic external source: [Cantera 3.1 governing equations](https://cantera.org/3.1/reference/onedim/governing-equations.html), one web open. The returned extract was lines 0-200, including navigation, equations, diffusive-flux definition, generic boundary conditions and two-point-control description. Only energy/species equations and zero-sum diffusion (lines 86-91 and 135-144) informed the cp cancellation check. No linked examples, mechanism coefficients, numerical modeled-investigation outcomes or datasets were opened. This reference supports the general equation, not the spherical implementation's qualification.

Initial hashes:

| File | SHA-256 |
|---|---|
| property_droplet_cleanroom.rs | `7fefce13641c20db374149e7a2aef5961ee5b0b15e3ac57931a7c58d07d2bcbe` |
| property_bre_w0.rs | `aac7572e3068961a80053c0299d81cea30ad504426cb64261cb8eaf5ffe3e3b7` |
| property_solids_w0.rs | `fe59a4cb7e15069f1427aaf5cb8e7b7c70043bbb2984b1b308f18342ec8bbc06` |
| astra_bre_w0.md | `4b81fefb01c3416809447488be6071d7f506fc05e509a306e12b87de9523b0a3` |
| astra_solids_w0.md | `00fa3561afce53f82c978fc9cf2d4bf5111f227c9f2916c1bdbeb865fe757faf` |

The droplet report did not exist at the initial listing. Its first complete read/hash was `c39afde7106b23aff3d718992f43f36ea824833c0eebf660b751bcdcd8dbd591`. During review the droplet kernel changed to `554118dd64faa5509cd17f613b3cc0ba1742a427dcb60d49eb9721583ab3cbb1`. Drift was reported to the coordinator immediately, rather than silently treating it as the initial snapshot. Targeted reread covered the added state-budget/identity checks, optional chemistry cumulatives, finite combustion/sheet checks, sheet-temperature combustion-property domain checks and midpoint mass guard. The visible governing thermal formulas were unchanged. BRE and solids and their reports remained at the hashes above during the executed counterexamples. The final candidate freeze and any later edits require separate reconciliation; this report is not a blanket approval of unseen changes.


### Final snapshot reconciliation

Coordinator confirmed final-ready droplet kernel `d683f755441a5c730777254421731bfc9e34fe7a99f6bc5a992ba62ce9a69199`; direct SHA-256 readback matched. Its final report hash is `3645c6bc3d64749af48a4c44cf07b79a455d7a802ecbede7334f2c6f4ad952dc`. Final reread covered kernel lines 1-170 (stored-tensor constructor and input definitions), 571-943 (thermal/property path, state update and integrator), targeted reference/flux searches, the report's source/disclosure changes and final numerical/check section, and the full new droplet `meta.json`. The constructor validates exact spec identity, tensor inventory/shape, finiteness/bounds and positive/fractional controls; it introduces no new thermal relation. The reviewed thermal equations and limitations above still apply to this final kernel. Source aliases in metadata are documentary provenance pointers, not an independent property determination or outcome evidence.

The coordinator confirmed original solids `fe59a4cb...` stays frozen and any response to the two findings will be a separate new candidate. Such a candidate is outside this completed review. Original BRE remains `aac7572e...`. This review neither changed nor approved model weights, performed a release construction, or qualified an experimental prediction.

Independent harness source hashes: `review.rs` = `22c0b08a8a9972554ae4470b52b1f62267889e9b914af68a4c1d9a6eb546df4e`; `domain.rs` = `87894b5a2da3f42c98ef03ba2ebc1006be47364f163e45861ce4752c799bf690`.

## Independent re-review: review-informed solids v2

Scope extension explicitly requested by coordinator: `property_solids_v2.rs` and `astra_solids_v2.md`, new `law-review-astra` candidate. This is a review-informed successor, not another outcome-blind W0 authorship. No predecessor kernel/literal was edited. Initial and final direct hashes matched throughout the checks:

- Kernel: `efc43f07e245a8fa8ec8148810e3f4d45f56e550248e721c253046d83ebc5b09`.
- Author report: `7f2652d9fcbf0ff6b60a6f87e8d18a76cb3a46c09a1e323006c9afbc58438118`.

Read extent: full new report; targeted symbol search; v2 lines 306-485 and 551-860; full production changes in `git diff --no-index` against the previously reviewed original. That diff also displayed new synthetic tests and inherited-test call renames, with the long test portion truncated. No new external sources, original-data outcomes, model releases, packet literals, or NASA material properties were opened. No Cargo build or candidate modification occurred.

### Domain finding resolved for the reviewed numerical path

`cell_path_temperature` checks material plus the appropriate front/back interface properties. The initial state is checked through `stable_step_s`; the heated intermediate and post-reaction endpoint are checked before committing the cloned state. The two exposed faces are both checked for a single through-thickness cell. The thermal substage has a fixed linear-in-time source. With nonnegative endothermic extents and persistent positive inert capacity, the declared frozen reaction substage has T(s)=T_heated-Qreaction(s)/C(s): Qreaction is nondecreasing and C is nonincreasing, so T cannot increase. Endpoint membership in closed property intervals therefore covers that **numerical split path**, including an internal hot excursion. It does not certify the unknown exact continuous trajectory or physical property truth.

The original independent `domain.rs` was reused with only the module path changed to v2, the method renamed to `advance_checked`, and unchanged-state assertion added. The first 20 W/1 s step now returns the domain error and leaves T=1000 K, time and cumulative state unchanged. The refined mode also rejects the same physically out-of-domain requested interval and leaves the complete Debug representation of State unchanged.

A separate counterexample heats a reactive 2 J/K initial inventory by 600 W for 1 s. Its split heated temperature is 1300 K, while reaction subsequently yields 1068.941421369995 K. With a boundary domain [900,1100] K, both original and final temperatures are admissible, but the new checked mode correctly rejects the hidden intermediate excursion. The state remains unchanged.

### Checked-mode predecessor comparison

Compiled and executed the original independent logarithmic-capacity harness against both predecessor and v2 `advance_checked`. Captured output lines were identical for both the 1-step and 10,000-step valid cases: temperatures, exact-reference errors, masses, energy residuals and thermal stable-step report all matched at the printed precision. The source diff independently shows that admitted-state arithmetic is unchanged; the new checks add domain rejection, rather than rewriting the physical update. The original 41.1158 K one-step discrepancy remains in checked mode by design, and is not concealed as a fix to temporal accuracy.

### Refinement, nonmonotonic cases and resource boundary

The new method advances n/2n/4n trajectories from identical initial states. Temperature and each constituent mass are compared on synchronized nodes. The fine-medium discrepancy is accumulated at every medium node, not solely at the interval endpoint; contraction uses common coarse nodes. It commits only the completed fine trajectory on successful admission. Every attempted substep checks the caller work limit before invocation, including failed coarse-step attempts; resource exhaustion returns without committing any trial. Integer mesh multiplications are checked before loop indexing. Physical and numerical scope are kept separate: the receipt explicitly has `rigorous_error_bound=false`.

Independent actual-kernel results with requested absolute temperature tolerance 0.1 K (zero relative tolerance), fixed spatial grid and unchanged physical properties:

| Scenario | Executed result |
|---|---|
| Insulated constant-rate logarithmic solution, t=1 s | Refined T=810.039084939312 K versus exact 810.057253479139 K; error 0.018168539827 K. Accepted 2048 substeps; attempted 7161; observed contraction 0.499901666360; scaled tail estimate 0.545127696786. |
| Same reactant with 100 W fixed heating over 4 s | Nonmonotonic continuous solution T=1000+100t+600 ln[(1+exp(-t))/2]; minimum 854.448416983816 K at ln(5) s, then reheating. Final T=994.983985227661 K versus exact 995.001648414719 K. Maximum actual error across the accepted mesh 0.017663187058 K; 16384 accepted/57337 attempted substeps. Independently reconstructed three meshes match the receipt's discrepancy, scaled estimate and contraction ratio. |
| Inert cooling, C=1 J/K, G=1 W/K, ambient 300 K, initial 1000 K | Exact T=300+700 exp(-t). Over 4 s, fine-medium discrepancy peaks at t=1 s: 0.031444606780 K, versus only 0.006258699325 K at the endpoint. The receipt retains the larger interior discrepancy. Maximum actual exact-solution error is 0.031438208967 K; 16384 accepted/57318 attempted. This directly tests a nonmonotonic error history that an endpoint-only estimator would miss. |
| Inert exact heating with max-work 6 versus 7 | Six attempts cannot complete n/2n/4n and reject without changing state. Seven succeed with receipt attempted_substeps=7, accepted_substeps=4, zero discrepancy. |
| Endothermic reactive case with max-work 1, 6, or 7 | Every call rejects with work-budget-exhausted error and unchanged complete state. Seven complete the first trial but cannot justify the requested error budget; no partial coarse/fine trajectory leaks into the state. |

All three analytical checks returned `rigorous_error_bound=false`. No fitted parameters or new candidate numerical choices were generated by this reviewer. These executions are synthetic verifications of requested behavior, not evidence about materials or NASA experiments.

No additional blocking defect was demonstrated in this scoped re-review. The original domain-admission finding is resolved in this exact v2 snapshot. Temporal refinement supplies useful evidence beyond conservation, with important remaining limits: three-mesh agreement cannot prove continued contraction or exclude shared bias; only synchronized numerical nodes are compared; spatial discretization, control-history uncertainty, constitutive validity and physical discrepancy are not estimated. The author report and receipt retain those distinctions. A caller can choose an inadequate work budget and receive an error; that is intentional fail-closed behavior, not missing physical data replaced with zero.

### Reproduction and results

```powershell
rustc --edition 2021 -A dead_code runs/tooling/astra-thermal-review/domain_v2.rs -o runs/tooling/astra-thermal-review/domain_v2.exe
& runs/tooling/astra-thermal-review/domain_v2.exe
rustc --edition 2021 -A dead_code runs/tooling/astra-thermal-review/review_v2_checked.rs -o runs/tooling/astra-thermal-review/review_v2_checked.exe
& runs/tooling/astra-thermal-review/review.exe
& runs/tooling/astra-thermal-review/review_v2_checked.exe
rustc --edition 2021 -A dead_code runs/tooling/astra-thermal-review/v2_refined.rs -o runs/tooling/astra-thermal-review/v2_refined.exe
& runs/tooling/astra-thermal-review/v2_refined.exe
```

All standalone compilations and harness assertions succeeded. The captured old/new checked-output arrays compared equal with PowerShell `Compare-Object`. The separate `git diff --no-index` returned its expected status 1 because the candidates differ; this was not a compilation/test failure. This receipt applies to the exact hashes above, prior to any subsequent freeze or changes.
