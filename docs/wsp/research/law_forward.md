# The generic law forward (`law.rs`): one strict forward for every hand-written law

Status: implemented on `feat/handwritten-science` (15 November). Code: `crates/ignis-wsp/src/law.rs`
(the forward), `crates/ignis-wsp/src/law_slugs.rs` (the table of slugs), declarations under
`configs/models/law/<slug>/decl.json`. Tests: `crates/ignis-wsp/tests/law_forward.rs`. Tools:
`tools/wsp/law_run.py`. Targets that the shared contract cannot name: [law_forward_targets.md](law_forward_targets.md).

**What it is for.** A physicist restates a law as "a small typed network over named input features"
(`runs/dev/gen7/BRIEF_PHYSICS.md`, item 7). This forward runs any such law without a bespoke forward:
every parameter is a tensor the author types in the edit language (`docs/wsp/PROTOCOL.md` section 2),
the forward fits nothing, searches nothing and fills nothing, and an input outside what the law
declares is a scientific refusal that names the input.

## 1. Two files per slug: a declaration (names) and edits (numbers)

The edit language carries numbers only, so what is *not* a number lives in a declaration file that
the forward compiles in:

| File | Holds | Who writes it |
|---|---|---|
| `configs/models/law/<slug>/decl.json` | feature names (= prefix condition keys), their unit and transform; targets, their kind, unit, causes, the optional shared quantity; no number at all | the engineer restating the physicist's law (once, before any release of the slug) |
| `configs/models/law/<slug>/<candidate>/edits.wsp` | every number: centres, scales, domain bounds, biases, skip weights, hinge units, spreads, shapes, stop offsets | the author (physicist), literally |

A declaration is part of the slug's spec (its hash) and of its forward identity (its text is one of
the slug's forward sources). **Once a release of a slug exists its declaration is frozen**; a new
declaration is a new slug (one line in `law_slugs.rs`). Adding a slug changes no other slug's spec
or forward identity, and no existing release id. A declaration change needs a rebuild of the
binaries (`include_str!`).

The ten investigation slugs (`sflame_event`, `bre`, `cld`, `efield`, `slice`, `spice`, `samer`,
`same`, `daft`, `bass1`) are registered with **placeholder** declarations (`"status": "placeholder"`,
no feature, one dimensionless target `outcome`). A placeholder spec compiles and can be inspected, but
its build check fails, so no release of a placeholder can be assembled. Restating a law = replacing
`decl.json` (status `declared`) and writing `edits.wsp`. The slug `synthetic_demo` is a complete
synthetic declaration used by the tests and by the example below; it is never NASA data.

### 1.1 Declaration format

The synthetic demo, abridged (the file also declares a positive target `spread` with
`"shared_quantity": "spread_rate"` and a one-cause event target `burnout`):

```json
{
  "slug": "synthetic_demo",
  "status": "declared",
  "family": "other",
  "investigation": "free text: PSI number, apparatus",
  "description": "free text",
  "features": [
    {"name": "pressure", "unit": "Pa", "transform": "ln", "description": "chamber pressure"},
    {"name": "x_o2", "unit": "1", "transform": "identity", "description": "ambient O2 mole fraction"},
    {"name": "fuel_flow", "unit": "kg/s", "transform": "reciprocal", "description": "fuel mass flow"}
  ],
  "targets": [
    {"name": "flame_length", "kind": "positive", "unit": "m", "shared_quantity": null,
     "description": "visible flame length at steady state"},
    {"name": "ignites", "kind": "binary", "description": "the flame ignites and is sustained"},
    {"name": "flame_end", "kind": "event", "causes": ["extinction", "blowoff"],
     "scheduled_stop": {"condition": "programmed_stop"},
     "description": "end of the flame, seconds since the time origin"}
  ]
}
```

Rules (checked when the spec is built; a bad declaration fails `cargo test -p ignis-wsp`):

- `family`: `flex`, `acme_normal`, `acme_inverse`, `bass` or `other` (default `other`); a prefix of
  another family is refused.
- Features: 0 to 16. `name` is the key of `PrefixBundle.known_conditions` read (its `value_si`, which
  must be resolved and finite) and must match `[a-z][a-z0-9_]*`. `unit` is the SI unit of `value_si`.
  `transform` is one of `identity`, `ln`, `reciprocal` (closed list).
- Targets: 1 to 8, names unique, `[a-z][a-z0-9_]*`. `kind` is `positive`, `binary` or `event`.
  - `positive`: `unit` required (SI). `shared_quantity` (optional) names a member of the shared
    `Quantity` list (`liquid_diameter`, `visible_flame_diameter`, `uv_flame_diameter`,
    `droplet_position`, `pressure`, `mole_fraction`, `temperature`, `mass_flow`, `volume_flow`,
    `std_volume_flow`, `front_contour`, `front_position`, `spread_rate`, `flow_velocity`,
    `gravity_level`, `phase_end_time`) and must have the same SI unit; give it only when the target
    *is* that quantity. Otherwise the target is lane-local (see section 5).
  - `binary`: no unit.
  - `event`: `causes`, 1 to 3 names (not `active`, not `scheduled_stop`); optional
    `scheduled_stop.condition`, the prefix condition holding a planned stop time (as
    `programmed_stop` in `committee2.rs`).

## 2. The forward

### 2.1 Features

For feature `f` (declaration order `i = 0..n-1`) with raw value `x = value_si`:

```text
identity    z_i = (x - c) / s
ln          z_i = ln(x / c) / s          x > 0 and c > 0, else refusal
reciprocal  z_i = (1/x - 1/c) / s        x != 0 and c != 0, else refusal
refusal unless  lo <= x <= hi            (raw units, both ends included)
```

`c = x.<f>.centre`, `s = x.<f>.scale`, `lo = x.<f>.lo`, `hi = x.<f>.hi` are typed tensors. A missing,
unresolved or non-finite condition, a value outside `[lo, hi]`, or a value the transform cannot take
is a scientific refusal (gate `condition`) whose reason names the feature.

### 2.2 The network of every law

```text
N(z) = b0 + skip . z + sum_{j<8} w2[j] * relu(b1[j] + W1[j, :] . z)
```

The same form as `nn::hinge_net` and the burner-flame forwards. With every hinge weight zero, `N` is
the linear law `b0 + skip . z`; a hinge unit is only there if the author placed it. With no feature
declared, `N = b0` and the network has only `b0`.

### 2.3 Target kinds

**positive** (a length, a diameter, a rate, a time, a signal): in natural-log space

```text
m = N_median(z)                      ln(median / 1 unit)
sigma = N_spread(z)                  the ln-sd of the core; must be > 0 at z, else refusal (gate numerical)
F(y) = (1 - w) Phi(ln(y/e^m) / sigma) + w Phi(ln(y/e^m) / (k sigma))
w = tail_weight, k = tail_width      (w = 0: one log-normal; k = 1: no wider component)
```

The quantile at level `p` is `exp(m + q)` with `q` the root of `F` in ln space; for `w = 0` it is
`exp(m + sigma Phi^-1(p))` exactly, otherwise it is found by bisection of the known, monotone
mixture CDF between `sigma Phi^-1(p)` and `k sigma Phi^-1(p)` (a quantile of a fixed distribution,
not an estimate of a parameter). The mean `e^m ((1-w) e^(sigma^2/2) + w e^((k sigma)^2/2))` is
reported when finite.

**binary** (yes or no): `P(yes) = logistic(N_logit(z))`.

**event** (an end in time, with 1 to 3 competing causes): for cause `c`

```text
tau_c = exp(N_c(z))                 median time of cause c alone, seconds since the time origin
S_c(t) = 1 / (1 + (t / tau_c)^k_c)  log-logistic clock, shape k_c = <t>.<c>.shape
h_c(t) = (k_c / t) (t/tau_c)^k_c / (1 + (t/tau_c)^k_c)
S(t) = prod_c S_c(t)
```

conditioned on no event before the cutoff `tau` (`prefix.cutoff_s`):

```text
active(t)          = S(t) / S(tau)                                         (t < stop)
cause_c(t)         = integral_tau^t h_c(u) S(u) du / S(tau)
scheduled_stop(t)  = S(stop) / S(tau) for t >= stop, else 0
```

The rows sum to one at every time. The integral is taken in `num.substeps` equal steps per
requested interval: the mass lost over a step is `S(u0) - S(u1)` exactly, shared among the causes
in proportion to their hazards at the step midpoint. With one cause this is `1 - S(t)/S(tau)`
exactly; with several the split is accurate to the step size (`tests/law_forward.rs` compares it
with a fine quadrature).

The stop: the earliest control stage `scheduled_stop` or `flow_stop` of `prefix.known_controls`;
and, when the declaration names `scheduled_stop.condition`, the prefix carries that condition
(resolved, finite, positive) and `<t>.stop.offset_w` has a positive entry, a mixture over the three
offset nodes `k` with `offset_w[k] > 0` (weights normalised), each run with the stop
`min(control stop, condition + offset_s[k])`, as in `committee2.rs`. A stop at or before the
cutoff is a refusal. Without a stop the `scheduled_stop` row is zero.

### 2.4 What a request asks and what comes back

`ignis wsp forward --release <id> --prefix <file> --settings '<json>'`, settings:

| Key | Meaning |
|---|---|
| `target` | target name; default the first declared target |
| `quantile_levels` | positive targets; default `[0.05, 0.25, 0.5, 0.75, 0.95]` |
| `times_s`, or `horizon_s` with `n_points` | event targets, seconds since the time origin; required (no default horizon exists) |

| Kind | Artifact |
|---|---|
| positive | `kind = prefix_forecast`, `times_s = [cutoff_s]`, `quantile_levels`, `quantiles` (one column), `mean`; `quantity` = the shared quantity or none |
| binary | `kind = event_probabilities`, `times_s = [cutoff_s]`, `event_categories = ["yes", "no"]` |
| event | `kind = event_probabilities`, `event_categories = ["active", <causes...>, "scheduled_stop"]`, rows over `times_s` |

Every artifact carries `diagnostics.law`: `{slug, target, kind, label, unit, shared_quantity}` with
`label = "<slug>:<target>"` (the lane-local target label), the raw and normalised features and the
network outputs (`ln_median`, `ln_sd`, `logit`, `ln_tau` per cause).

## 3. Tensors of a slug (n = number of features, H = 8 hinge units)

Every element of an *authored* tensor must be written by a root patch; *sparse* tensors are zero
unless written. Bounds are support limits; a value outside them rejects the patch.

| Tensor | Shape | Fill | Role | Unit | Bounds | Meaning |
|---|---|---|---|---|---|---|
| `x.<f>.centre` | [] | authored | physical | unit of f | ±1e30 | centre `c` of the transform |
| `x.<f>.scale` | [] | authored | physical | f's transform unit (`unit`, `1` for ln, `1/unit`) | [1e-30, 1e30] | scale `s` |
| `x.<f>.lo`, `x.<f>.hi` | [] | authored | physical | unit of f | ±1e30 | declared domain, raw units |
| `<P>.b0` | [] | authored | per network (below) | per network | per network | value at z = 0 |
| `<P>.skip` | [n] | authored | weight | | | linear weights, feature order |
| `<P>.W1` | [8, n] | sparse | weight | | | hinge input weights, row j reads z |
| `<P>.b1` | [8] | sparse | bias | | | hinge positions |
| `<P>.w2` | [8] | sparse | weight | | | hinge output weights |

Networks `<P>` per target `<t>`:

| Kind | Networks `<P>` (b0 role, unit, bounds) | Other tensors |
|---|---|---|
| positive | `<t>.median` (physical, `ln(<unit>)`, [-80, 80]); `<t>.spread` (noise, `1` = ln-sd, [0.001, 10]) | `<t>.tail_weight` [] noise [0, 1]; `<t>.tail_width` [] noise [1, 100] |
| binary | `<t>.logit` (hazard, `logit(1)`, [-40, 40]) | |
| event | `<t>.<cause>.median` per cause (physical, `ln(s)`, [-20, 25]) | `<t>.<cause>.shape` [] hazard [0.1, 50]; with a declared stop: `<t>.stop.offset_s` [3] physical s [-1e5, 1e5], `<t>.stop.offset_w` [3] weight [0, 1] |

Constant: `num.substeps` = 512 (integration steps per requested interval; numerical, not physics).

With no feature declared, only the `b0` of each network exists.

### 3.1 The neutral root

`ignis wsp law-root --family <slug> --out <dir>` writes `<dir>/edits.wsp` and `<dir>/meta.json` of the
slug's neutral root, generated mechanically from the spec (no claim): every `b0` 0 except the spread
`b0` at its upper bound (10, the widest spread the spec allows), every skip 0, no hinge, tail weight
0 and width 1, every shape at its lower bound (0.1), stop offsets and weights 0, every centre and
scale 1 (they have no effect while every feature weight is zero) and every domain bound at its tensor
bound (±1e30: no domain claim beyond the transform's own). The root forecasts a median of 1 unit
with an ln-sd of 10, a probability of 0.5, clocks of median 1 s with the widest shape. It is a
lineage root and is never a candidate.

A candidate is either a **revision of the root** (`parent_release_id` = the root; it writes only what
the law uses, the rest stays neutral) or a **root patch of its own** (it writes every authored
element). For a zero-shot W0 the revision of the neutral root is the convenient form; its exposure is
what the author saw, and the root carries no information.

## 4. Example: an invented law (synthetic, not NASA)

The `synthetic_demo` declaration above. Feature order: `pressure` (ln), `x_o2` (identity),
`fuel_flow` (reciprocal). A revision of the root, `configs/models/law/synthetic_demo/example/edits.wsp`:

```text
# synthetic_demo.wsp: an invented law to show the syntax. Not physics, not NASA.
# features z0 = ln(p / 101325 Pa) / 1, z1 = (x_o2 - 0.21) / 0.1, z2 = (1/mdot - 1/1e-6) / 1e6
x.pressure.centre = 101325
x.pressure.scale = 1.0
x.pressure.lo = 20000
x.pressure.hi = 200000
x.x_o2.centre = 0.21
x.x_o2.scale = 0.1
x.x_o2.lo = 0.15
x.x_o2.hi = 0.40
x.fuel_flow.centre = 0.000001
x.fuel_flow.scale = 1000000
x.fuel_flow.lo = 0.0000002
x.fuel_flow.hi = 0.000005

# flame length: median 2 cm at the centre, longer at lower pressure, one hinge above x_o2 = 0.31
flame_length.median.b0 = -3.9
flame_length.median.skip[0:3] = [-0.5, 0.2, 0.0]
flame_length.median.W1[0, 1] = 1.0
flame_length.median.b1[0] = -1.0
flame_length.median.w2[0] = 0.3
flame_length.spread.b0 = 0.25
flame_length.spread.skip[0:3] = [0.0, 0.0, 0.0]
flame_length.tail_weight = 0.1
flame_length.tail_width = 3.0

# ignites: logit 1.5 at the centre, more likely with oxygen
ignites.logit.b0 = 1.5
ignites.logit.skip[0:3] = [0.0, 2.0, 0.0]

# flame end: extinction clock median e^4.5 = 90 s, blow-off e^6 = 403 s
flame_end.extinction.median.b0 = 4.5
flame_end.extinction.median.skip[0:3] = [0.0, 0.8, 0.0]
flame_end.extinction.shape = 3.0
flame_end.blowoff.median.b0 = 6.0
flame_end.blowoff.median.skip[0:3] = [0.0, 0.0, -0.4]
flame_end.blowoff.shape = 2.0
flame_end.stop.offset_s[0:3] = [0.0, 5.0, 0.0]
flame_end.stop.offset_w[0:3] = [0.5, 0.5, 0.0]
```

## 5. Targets the shared contract cannot name

`AnalysisArtifact.quantity` is the shared `Quantity` enum, and `ignis-core` is not changed here. A
positive target with `shared_quantity` set returns artifacts with that quantity and its SI unit, so
the existing evaluator functions score it unchanged. A lane-local target (a flame length, a
smoke-point length, an ion current, a particle diameter, a detector signal) returns
`quantity = null` and `unit = null` in the artifact, and its identity in `diagnostics.law`:
`label` `<slug>:<target>`, `unit` (SI), `shared_quantity: null`. It is never written under an
unrelated shared quantity. Event targets return `quantity = null`, as the ACME event models do.

`ModelSpec.target` is one required shared quantity. A law spec sets it to the shared quantity of its
first target that has one (an event target counts as `phase_end_time`, as ACME's specs do); a slug
whose targets are all lane-local names `phase_end_time` there only because the field is mandatory,
and its `excludes` say so and list the lane-local targets. Proposals for the contract owners are in
[law_forward_targets.md](law_forward_targets.md).

## 6. Commands

Build (other agents run these binaries; if the link fails because a file is in use, wait and retry):

```bash
CARGO_TARGET_DIR=target-merged cargo build --release -p ignis-cli -p ignis-wsp-dev
```

```bash
IGNIS=target-merged/release/ignis
```

The spec a physicist writes against (tensor names, shapes, bounds):

```bash
$IGNIS wsp spec --family synthetic_demo --out configs/models/law/synthetic_demo/spec.json
```

The slug's neutral root, compiled and assembled:

```bash
$IGNIS wsp law-root --family synthetic_demo --out configs/models/law/synthetic_demo/root
```

```bash
$IGNIS wsp compile --family synthetic_demo --edits configs/models/law/synthetic_demo/root/edits.wsp --meta configs/models/law/synthetic_demo/root/meta.json --out configs/models/law/synthetic_demo/root/patch.json
```

```bash
$IGNIS wsp assemble --family synthetic_demo --patch configs/models/law/synthetic_demo/root/patch.json --binding configs/models/law/synthetic_demo/binding.json --out configs/models/law/synthetic_demo/root/release.json
```

A candidate from a typed `edits.wsp` (its `meta.json` names the root as `parent_release_id`, the
author, the exposure and the rationale; see any `meta.json` under `configs/models/acme/`):

```bash
$IGNIS wsp compile --family synthetic_demo --edits configs/models/law/synthetic_demo/example/edits.wsp --meta configs/models/law/synthetic_demo/example/meta.json --out configs/models/law/synthetic_demo/example/patch.json
```

```bash
$IGNIS wsp assemble --family synthetic_demo --patch configs/models/law/synthetic_demo/example/patch.json --binding configs/models/law/synthetic_demo/binding.json --out configs/models/law/synthetic_demo/example/release.json
```

Every number back as edits, and one prefix:

```bash
$IGNIS wsp inspect --release <release id> --edits
```

```bash
$IGNIS wsp forward --release <release id> --prefix prefix.json --settings '{"target":"flame_end","times_s":[10,30,60,120]}'
```

A case file of the data engineers (`runs/dev/data_all/work/transfer_<slug>/cases_train.jsonl`, one
JSON object of named inputs per line):

```bash
python tools/wsp/law_run.py --release <release id> --cases runs/dev/data_all/work/transfer_<slug>/cases_train.jsonl --out runs/dev/law/<slug>/train_forecasts.jsonl
```

`law_run.py` takes the slug's declaration from the release record (`applicability.declaration`),
reads per case the inputs it names (from `case["inputs"]` when the case has that dict, else from the
case; a dict `{"value", "unit", "known"}` is read as its value, a unit other than the declared one
or `known: false` makes the input unusable and the forward refuses it naming the feature; nothing is
converted), the declared stop condition of an event target the same way, `case_id` (or `test`,
`run_id`), optional `cutoff_s` (default 0) and `times_s`; it writes one prefix per case, runs the
forward once per target, and writes one line per case:
`{"case_id", "release_id", "targets": {<name>: {"kind", "label", "admitted", "reasons",
"quantiles": {"0.05": …, …}, "mean", "probability", "times_s", "cdf": {<category>: [...]}}}}`.
Options: `--targets a,b`, `--times 10,30,60`, `--levels 0.1,0.5,0.9`, `--root <registry>` (default
`$IGNIS_ARTIFACTS` or `artifacts`), `--ignis <binary>` (default `target-merged/release/ignis`),
`--workers 8`. Prefix files and artifacts are kept in `<out>.work/`. Checked end to end on synthetic
cases of `synthetic_demo` (15 November).

## 7. What the forward cannot express (tell the engineer before restating)

- A law that is not "transform, then hinge network": products of features beyond what a hinge or a
  log-transform gives (the ln transform makes power laws additive, so `L ~ p^a mdot^b` is linear in
  ln space), tabulated property lookups, a solved ODE, or a time series read from the prefix.
- More than 16 features, 8 hinge units per network, 8 targets or 3 causes.
- A spread that depends on the time, or a heavy tail whose weight or width depends on the features
  (`tail_weight` and `tail_width` are scalars; the core ln-sd `N_spread(z)` does depend on them).
- A categorical input: write it as a 0/1 indicator condition with `identity` (centre 0, scale 1,
  domain [0, 1]); the forward does not read text conditions.
- A clock shape that depends on the features (each cause's `shape` is a scalar, so the spread of
  ln T is the same at every condition; the median moves, the width does not), and a spread on a
  logit (a probability has no second moment to state; write the uncertainty into the logit itself).
- An event clock other than log-logistic, latent classes (the weak/main flames of `flame.rs`), a
  floor time before which a cause cannot act, or a cause-specific dependence on the stop.
- A positive target whose median depends on another target's forecast (targets are independent).

## 8. Verification (15 November)

- `cargo test -p ignis-wsp`: every suite passes; `tests/law_forward.rs` holds 14 tests (declarations
  and identity, the documented tensors, placeholders refused, the neutral root, transforms, the
  positive, binary and event hand computations, hinge slopes, one cause = one clock to 1e-12,
  competing risks against a 200 000-interval Simpson quadrature to 2e-5, the stop and its offset
  nodes, refusals naming the feature, patch bounds, a release refused by another slug).
- `cargo fmt --all -- --check` and `cargo check --workspace --all-targets` clean.
- Identity: every lineage rebuilt by `tools/wsp/reassemble.py` into an isolated registry (a copy
  of `configs/`, `IGNIS_ARTIFACTS` in a scratch directory) gives the id of each of the 182 existing
  `release.json` files (and of the two new demo releases); no weight hash differs. The law files add sources without editing a forward.
- The committed demo lineage: `configs/models/law/synthetic_demo/root` (neutral root) and
  `example` (the edits of section 4, a revision of the root), both rebuilt by `reassemble.py`.
