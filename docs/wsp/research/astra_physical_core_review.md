# Astra independent physical-core review

2026-11-14. Read-only review of `crates/ignis-wsp/src/physical_core.rs` and `docs/wsp/measure/PHYSICAL_CORE.md`; no implementation, property, model, or weight changes. No new experimental/model data opened. A temporary standalone Rust harness used synthetic inputs only.

Reviewed source SHA-256: `0B5D392EF73E8550AF3D3220EC3919D66FFA237793C9E196728B186DB6EFCA84`. Documentation SHA-256: `D9B8D3EB24CC33FA172CA74AF2737F43C4B21DB76321B49D6AC52BA057DE4A2A`. Line references below apply to those bytes. These are numerical-edge findings; no ordinary-domain equation or unit error was found.

## Actionable findings

### P2 — Stream mixing can silently erase a positive species contribution

Location: `physical_core.rs:677`, in `mix_stream_compositions`.

The multiplication `weight * fraction.get()` is not checked before accumulation. A positive contribution can underflow to zero, after which `GasMixture::new` receives an apparently legitimate zero and cannot apply its existing positive-fraction underflow guards at lines 438/488. This gives successful output with an absent species instead of refusing an unrepresentable composition. The point is consistency with the module's existing fail-closed fraction conversion, not a request for arbitrary precision or a new physical cutoff.

Executed reproducer: both synthetic components have molar mass 1 kg/mol, molar Cp 10 J/(mol K), a shared point domain at 300 K and 1 Pa, and identical reference conventions. Stream A has mass fractions `trace=1e-200`, `bulk=1`; stream B has `bulk=1`. Both inputs are accepted using the existing tests' explicit `NumericalTolerance(1e-12, 1e-12)`. Mixing at `z=1e-200` returns `Ok` with `trace=0`.

Direct derivation: the mixing equation requires a positive trace contribution `z*y_trace,A = 1e-400`; its `f64` representation is zero. The representational limit is unavoidable, but accepting it is avoidable. Check a positive weight and positive fraction for a zero product before accumulation, or otherwise return an explicit representability error. Do not replace the lost term with an epsilon.

Meaningful regression: this case should return `Err`; exact endpoint mixing with `z=0` or `z=1` and genuinely zero species fractions should still succeed. A nearby representable trace contribution, for example `1e-100 * 1e-100`, should remain positive.

### P3 — Richardson number can report exact zero with nonzero gravity and contrast

Location: `physical_core.rs:795`–`799`, especially the arithmetic at line 798.

The nonzero branch validates Richardson number only with `Finite::new`, which accepts zero after underflow. Other nonzero transport groups use `Positive` to reject this loss. Executed synthetic input: `L=D=nu=alpha=1`, velocity `[1e150,0,0]`, gravity `[1e-100,0,0]`, contrast `0.5`, same frame. The function returns `Pe=1e150`, `G=5e-101`, and `Ri=Some(0)`.

Direct derivation: `Ri=|g|*delta*L/|u|²=5e-401`, strictly positive but unrepresentable in `f64`. It should be refused or have an explicit numerical-underflow status rather than sharing the exact-zero representation used for absent gravity/contrast. If rejecting it, check the absolute value on the branch where gravity and contrast are nonzero, then restore the sign. Preserve `None` for zero velocity and exact zero for genuinely zero gravity/contrast.

Meaningful regression: test both signs of contrast with the extreme input; both should receive the chosen explicit underflow treatment. Existing true-zero and ordinary signed-Richardson cases should remain unchanged. This finding does not imply a measurable error for ordinary physical inputs.

## Mathematical and reference checks without findings

The mole/mass conversion, ideal-gas density, and molar/mass heat-capacity expressions are dimensionally consistent. `Cp_mix/M_mix = sum(y_i Cp_i/M_i)` follows directly from `y_i=x_i M_i/M_mix`. The public ideal-gas method checks positive `Cv_i=Cp_i−R`. [Cantera's primary ideal-gas documentation](https://cantera.org/3.1/cxx/d7/dfa/classCantera_1_1IdealGasPhase.html), standard-state and partial-molar sections read, supports the ideal-gas volume and heat-capacity relations. It does not qualify any caller-supplied property values.

For the documented CHONS product set, counting oxygen atoms gives `q=c+h/4+s−o/2` mol O2/mol; nitrogen contributes zero because the stipulated product is N2. Multiplying species mol/kg by q gives stream mol O2/kg. Linear mixing gives `q(z)=z q_f+(1−z)q_o`, whose root is `−q_o/(q_f−q_o)`. The code correctly requires opposing signed demands and returns a mass fraction of the complete fuel stream, including its diluents. This is not equilibrium, kinetic oxygen availability, or a flame-existence assertion. The optional atomic-mass consistency check and its mandatory use by the kg-O2/kg conversion are accurately documented.

`L²/D` has seconds; Pe, Sc, Pr, Le, G, and Ri are dimensionless. `Le=Sc/Pr` and, when defined, `Ri=G/Pe²`. The signed density contrast is bounded above by one to keep comparison density positive. A large negative contrast is algebraically permitted; no Boussinesq applicability is asserted. Scalar gravity groups do not determine geometry-relative orientation, and the documentation says so.

The flux checker requires matching species inventories and Cartesian frames and tests the mass-averaged condition `sum(j_i)=0`. This matches the invariant described in [Cantera's diffusive-flux equations](https://cantera.org/3.1/reference/onedim/governing-equations.html#diffusive-fluxes), section read. No mixture transport law or balancing correction is hidden in the checker. The tolerance's relative term compares the residual to zero, not to the individual flux magnitudes; callers needing cancellation-scale error bounds must provide an appropriate absolute budget. That behavior matches the documented comparison definition and is not raised as a defect.

The chemical-energy guard prevents the expressly forbidden combination of formation-inclusive enthalpy and a separate complete-combustion heat source. It does not prove that a sensible-only energy formulation has supplied all necessary chemistry or sensible enthalpy terms; neither the code nor documentation claims that it does.

## Deliberate restrictions and caller obligations

- Property domains are caller-declared rectangles with closed T/p bounds; the core does not establish phase stability or experimental validity. It does not introduce interpolation, branch selection, or a constitutive extrapolation.
- All species, including zero-fraction entries, must cover the state. Strict equality of property/reference/provenance records across shared identities is conservative and documented. Otherwise reconcilable reference states or separately sampled properties can be refused; this is intentional scope, not a mathematical bug.
- Positive scalar property values are supplied constants over their declared domains. Heat-capacity conversion itself does not evaluate a new thermodynamic state. Reference temperature need not lie within the Cp validity interval because the current code does not integrate enthalpy to that reference.
- Fractions are normalized only after the caller's explicit sum-error budget passes. A caller can choose an unjustifiably loose budget; there is no built-in operational tolerance or hidden default. The `1e-12` literals belong to synthetic tests only.
- The current neutral-CHONS representation cannot represent helium or argon. This is explicitly documented but blocks those real diluents until composition support is extended. Do not encode them as N2 or zero-element species. No assertion of all-investigation coverage follows from this core.
- Gas constant and atomic-mass provenance are caller responsibilities. No built-in property database, material-name switch, empirical transport coefficient, unity-Lewis approximation, optimizer, or model weight was found.
- Ordinary floating-point overflow/underflow may cause rejection even where an algebraically rearranged computation could succeed; documentation explicitly limits dynamic-range support. Such conservative rejections were not reported as correctness defects.

## Executed verification

Using `rustc 1.94.1 (e408947bf 2026-03-25)`, the isolated existing seven tests passed, zero failed. Compilation used `rustc --edition 2021 --test crates/ignis-wsp/src/physical_core.rs` with the executable under the OS temporary directory. This is independent execution of the inline tests, not workspace integration verification.

A separate temporary Rust harness imported the source via `#[path]`, created the synthetic fixtures described above, and asserted the two observed underflow behaviors. Both assertions passed, with output:

```text
mixing positive trace became 0.0
Pe 1e150; G 5e-101; Ri 0.0
```

No production source edits or new checked-in tests were made. Suggested regressions above are for the implementation owner. No NASA experiment, model, or validation/holdout execution occurred.
