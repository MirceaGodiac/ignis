# Strict WSP protocol, as implemented

How a literal release comes to exist in IGNIS, and what each step is allowed to do. Code:
`crates/ignis-wsp` (assembler and strict forwards), `crates/ignis-wsp-dev` (development feedback),
`tools/wsp` (rebuild and evaluation drivers).

## 1. Roles

| Piece | Does | Never does |
|---|---|---|
| **Spec** (`ModelSpec`, `configs/models/<family>/wsp/*spec.json`) | Fixes target, features with their transforms, every tensor with role, fill rule and bounds, constants, forward entry point | Hold a default for a physical, noise or hazard parameter |
| **Author** (a model, here `claude-fable-5-2`) | Types literal numbers in the edit language, with a public explanation | Run an optimizer, a regression, a search, or a program that computes the numbers |
| **Assembler** (`ignis wsp compile / check / assemble`) | Applies assignments, checks shapes, bounds and completeness, serialises, records provenance | Estimate, fill or repair anything |
| **Forward** (`ignis_wsp::{flex,acme,bass}::forecast`) | Deterministic features, fixed weights, explicit numerical integration | Gradient, MAP, NUTS, least squares, sampling, or a fallback to a fitted expert |
| **Feedback** (`wsp-dev`, `ignis eval`) | Scores forecasts on train and validation with the evaluator's functions | Read the test partition; change a parameter |

## 2. The edit language

```text
# comment
rate.b0 = -0.58                              # scalar tensor            -> set
rate.W1[2, 5] = 1.0                          # one element              -> set
rate.W1[3, :] = 0.0                          # constant over a box      -> fill_box
rate.skip[0:7] = [0.16, -0.075, 0.11, 0.08, -0.08, 0.052, 0.0]   # literal list -> set_box
haz.W1[0:2, 1:3] = [[1, 2], [3, 4]]          # nested list, may span lines
```

Selectors are non-negative integers and half-open slices `a:b` or `:`. Values are decimal literals.
There are no expressions, names on the right-hand side, strides, negative indices, repetition
operators, in-place arithmetic or includes; one bad statement rejects the whole file
(`tests/assembler.rs`). The MIT language also has strides, `*=`/`+=`/`-=` and `ZERO`; we left them
out so that a patch is a set of independent literal facts, each element written once.

## 3. Assembly

```text
zero tensors -> the spec's fixed constants -> parent's tensors (revision only) -> the patch
```

`HandwrittenWeightPatch::validate_against` rejects an unknown tensor, an out-of-range index, a value
outside declared bounds, an element written twice, a write to a constant, and — for a root patch —
any mandatory element left unwritten. The release record carries:

| Field | Meaning |
|---|---|
| `trained_on = not_trained`, `state = experimental` | Enforced by the registry for every literal release |
| `lineage` | `literal_authoring` (W0) or `weight_edit` (W1+) with `parent_id` |
| `construction.spec_hash`, `patch_hash`, `weights_hash` | Spec, the literal patch, the resulting tensors |
| `construction.forward_hash` | Digest of the forward sources (whitespace removed, so formatting is not identity) |
| `construction.author`, `exposure` | Exact provider model id; sources, datasets and partitions, feedback and other candidates seen |
| `construction.checks` | `spec_valid`, `patch_valid`, `tensors_complete`, `strict_forward_audit`, `finite_forward_on_fixture` |
| `construction.uncertainty_kind` | `assumed_noise` for every release here |
| `cohort_id`, `split_id` | The population the release is **declared for** so that existing commands find the runs; it was not fitted on it |

The release id is the content hash of that record. `code_version` is derived from the crate version
and the forward digest, not from the git tree, so re-assembling the same spec and patch gives the
same id. A release refuses to run under a forward other than the one it was assembled with
(`forward_mismatch`): a release is its weights **and** its forward.

## 4. W0 and W1

**W0 — one shot.** The author has the spec, the physics, the repository's code and configuration,
and input metadata of the train partition (condition ranges, time-stamp cadence). It has no measured
outcome, no score and no forward run of the candidate. The edits file is committed before the
candidate is run on anything; its hash is in the commit message.

- FLEX W0 edits: commit `2aaede4`, sha256 `f25ce9ec…2952`, unchanged since.
- ACME W0 edits (normal `3500499a…b194`, inverse `08ccd9d1…2c0e`): commit `b0415b3`.
- BASS W0 edits: `20d1eeff…78a1`.

**W1 and later — direct revision.** The author reads development feedback and types new numbers
over the parent. This is data-informed and is never called zero-shot. Each revision's `meta.json`
lists the feedback artifacts it was written from (kept in `docs/wsp/feedback/`), and its edits file
states which failure each block answers.

What was actually looked at, per candidate:

| Candidate | Train | Validation | Test |
|---|---|---|---|
| FLEX W0, ACME W0 ×2, BASS W0 | input metadata only | nothing | nothing |
| FLEX W1 | per-run detail | aggregate scores of W0 | nothing |
| FLEX W2 | per-run detail | **per-run detail of W1** | nothing |
| ACME normal W1, inverse W1 | endpoints of every run | aggregate scores of W0 | nothing |
| ACME normal W2 | endpoints of every run | aggregate scores of W0 and W1 | nothing |

Two disclosures. The author's pretraining may include published FLEX and ACME results; that
cannot be measured, so no test here is "fully blind". And the forward code itself changed twice
after the FLEX W0 edits were written (the cutoff state estimate, on the evidence of a unit test with
test-only weights; then a refactor and the formatting-proof identity), which is why the W0 release
has had three ids while its edits and weight hash never changed.

## 5. What a forward may read

Only the `PrefixBundle`: conditions known before the run, observations admitted at the cutoff,
control stages known at the cutoff. Local state (droplet size and rate offset, contour modes and
their velocity) comes from window means and finite differences that the spec declares as input
transforms. Root-finding or optimisation over parameters or latents does not exist on the route;
`tests/strict.rs` checks that the crate's dependency graph contains no workspace crate but
`ignis-core` and no optimizer, sampler or random-number crate.

## 6. Commands

```bash
cargo build --release -p ignis-cli -p ignis-wsp-dev
```

```bash
ignis wsp spec --family flex --out configs/models/flex/wsp/spec.json
```

```bash
ignis wsp compile --family flex --edits configs/models/flex/wsp/w0-fable/edits.wsp --meta configs/models/flex/wsp/w0-fable/meta.json --out configs/models/flex/wsp/w0-fable/patch.json
```

```bash
ignis wsp assemble --family flex --patch configs/models/flex/wsp/w0-fable/patch.json --binding configs/models/flex/wsp/binding.json
```

```bash
ignis wsp inspect --release <release id> --edits
```

```bash
python tools/wsp/reassemble.py
```

```bash
python tools/wsp/dev_eval.py --baselines runs/dev/baselines.json --out docs/wsp/RESULTS.md
```

```bash
cargo test -p ignis-wsp -p ignis-core
```

`reassemble.py` rebuilds every release in lineage order from the committed edits and prints whether
each weight hash is unchanged. `dev_eval.py` needs fitted baselines in the local registry
(`ignis fit run <model> --cohort … --split … --execute`; for the FLEX D² baselines add
`--set fit.skip_invalid_condition_runs=true`, one train run has a stored pressure of 0 Pa) and
validation continuations (`ignis eval deposit-cohort … --partition validation`). Never pass
`--final-test`, `--partition test` or `--partition all`.

## 7. Phase 2: several models per family, and data-informed candidates

**Model keys.** `--family` takes a model key: `flex`, `acme_normal`, `acme_inverse`, `bass`
(first generation, unchanged), `flex_methanol`, `flex_heptane` (the multistate droplet forward),
`acme_normal2`, `acme_inverse2` (the clock forward). A release finds its forward through its
`model_name`; the forward identity of a model covers the shared forward files and its own forward
file only, so adding a model never changes another model's release ids.

**Naming.** `W0` is reserved for a candidate written before any outcome of its fuel or family was
seen. A candidate written after its author read development data is named `D1`, `D2`, … and its
edits file says in its first lines what was read. The registry records a zero-shot candidate as
`literal_one_shot` and refuses development exposure on it, so the first data-informed candidate of
a new spec is a `literal_revision` of that spec's reference baseline (a condition-free D²-law or
clock, itself `literal_one_shot` with no data exposure). The baseline doubles as the comparison
"what does a forecast score that knows nothing".

**Order of work for a fuel or family not yet looked at** (n-heptane, 13 November):

1. Freeze cohort and split from metadata only (`wsp-dev heptane-cohort --execute`); commit.
2. Write and commit a literature-informed W0 (edits hash in the commit message).
3. Only then open the train outcomes; score W0; write D1 as a revision of W0.

**Validation discipline.** A candidate is frozen in git before its validation comparison. Train
feedback is read per run; validation is read as an aggregate table only. One calibration-only
revision (spread or clock width, no law) was made per model after its first aggregate validation
look and is named as such; after that a candidate replaces the carried one only by a rule on train
stated before validation is run. What counts as development exposure is recorded in every
`meta.json`.

**What tools may do.** Measurement scripts (`tools/wsp/analysis/`, `tools/wsp/bass_optical/`) print
per-run finite differences, window means and group counts, minima, medians and maxima. Analysis may
be delegated to other model instances under the same rules (train only, no fitting). No script and
no delegate writes a weight: the author reads the tables, checks them, and types the numbers.

**Reference baselines.**

```bash
ignis wsp assemble --family flex_heptane --patch configs/models/flex/wsp/heptane/baseline-d2/patch.json --binding configs/models/flex/wsp/heptane/binding.json
```

```bash
wsp-dev feedback --release <candidate> --release <baseline> --partition train --out runs/dev/fb.json
```

```bash
python tools/wsp/analysis/droplet_compare.py runs/dev/fb.json runs/dev/analysis/flex_heptane_profile.json 0 crps
```

## 8. Third generation (14 November): audits, a frozen forward per generation, the selection rule

**A forward is frozen with its releases.** `droplet.rs` was not edited to improve it. The
third-generation droplet forward is a new file, `droplet3.rs`, with its own model names
(`flex_methanol.wsp2`, `flex_heptane.wsp2`; keys `flex_methanol2`, `flex_heptane2`) and specs
(`flex_methanol-wsp-v3`, `flex_heptane-wsp-v2`). Every earlier release keeps its id, its forward and
its score, so the zero-shot heptane W0 stays exactly the object that was frozen before the heptane
outcomes were opened. The burner-flame revisions needed no forward change and are revisions inside
`acme_*.wsp2`.

**Delegated analysis.** Four analysis agents (claude-opus-5-5) each audited one model against its
train partition: error budget, residuals of each law by condition and by session, what transfers
between sessions, and law-level proposals with the runs that would contradict them. Their rules:
train partition only, no fitting, no forward or weight changes, nothing written to the repository.
Their reports are kept in [feedback/](feedback/). The author rechecked the claims it acted on
against the train tables (and declined some: the heptane helium clock, single-session effects),
then typed every number. One agent listed the run ids of validation forecast artifacts while looking
for train forecasts and opened none; one printed three rows of non-train BASS tests by accident and
did not use them. Both are recorded here and in the candidates' `meta.json`.

**Selection, stated before validation was run.**

| Model | Rule | Outcome on train |
|---|---|---|
| Droplet models | The candidate replaces the carried one when its median train CRPS over common cases is not worse and it is better or level (within 1 µm) in at least 60 % of train sessions (`tools/wsp/analysis/session_compare.py`). | methanol D3 against D2: median 17.7 against 23.1 µm, better or level in 24 of 26 sessions. Heptane D2 against D1: 21.8 against 28.2 µm, better in 17 of 20. |
| Burner-flame clocks | No score rule: a change is taken when it holds in the sessions that test it (at least three, or two with a physical reason), and a number that came from one session is weakened, not sharpened. | normal D3, inverse D3 |

The four candidates were committed (`d3e00f4`) before any validation score of them existed.
Validation is reported in [RESULTS.md](RESULTS.md), was read once as aggregate tables, and adjusted
no number. It acted in one way only, as it had for the inverse D2: **as a veto**. A candidate that
passes on train but does not improve its family's primary validation score is not adopted and stays
in the tables as a negative result. Three candidates improved (methanol 45.3 → 40.0 µm, heptane
51.9 → 46.1 µm, normal-flame extinction Brier 0.170 → 0.159). The inverse D3 did not (extinction
0.146 → 0.156, flow termination 0.097 → 0.096), so the inverse D1 stays the carried candidate. That
veto is a use of validation and is recorded as one; with two validation sessions per configuration
it is a weak one.

**Spread is not chosen by the train rule.** A narrower band always scores better on the runs a law
was typed from. The spreads of the third generation are the train residuals with a stated margin for
sessions the laws have not seen; the reasoning is in each edits file.

```bash
python tools/wsp/analysis/session_compare.py runs/dev/fb_train_flex_d3.json runs/dev/analysis/flex_methanol_profile.json 0 1
```

## 9. Fourth generation (14 November): published science, three new forwards, a blind BASS validation

**What was read.** Three literature reviews by research agents (claude-opus-5-5), kept in
[research/](research/): droplet combustion, burner-supported spherical flames, concurrent flame
spread. Their rule: mechanisms, theory, numerical-model predictions and published correlations, with
the kind of every number stated, and no per-test result of the series being modelled. For flame
spread the measured results of the BASS-II concurrent tests were not read in any source, because
those burns are what the front model is tested on. Three more train-only analyses (error anatomy of
the two droplet candidates, what transfers between ACME train sessions) and the packaging of the BASS
train tracks are in [feedback/](feedback/). The author rechecked what it acted on and typed every
number.

**Forwards.** Earlier forwards were not edited; every earlier release keeps its id.

| Forward | Models (keys) | What is new |
|---|---|---|
| `droplet4.rs` | `flex_methanol.wsp3`, `flex_heptane.wsp3` (`flex_methanol3`, `flex_heptane3`) | a broad component when the prefix shows an unstable tracker (four prefix signals); a law for what follows a flame lost in helium |
| `flame3.rs` | `acme_normal.wsp3`, `acme_inverse.wsp3` (`acme_normal3`, `acme_inverse3`) | the published growth law of a spherical flame as the radiative clock, with a spread on the reduced flow; a second, empirical clock; a burner-heating law in place of the burner-limit clock |
| `front.rs` | `bass_front.wsp` (`bass_front`) | the first BASS model on real burns: tip of the flame against time, in sample widths, with the flow history as a known control |

`droplet4.rs` reads one more prospective condition, the sheet's initial diameter, so its prefixes
have their own hash and `ignis eval compare` cannot pair its cases with older releases. Each fuel
therefore has a **carry** release: the third-generation numbers on the fourth forward with the new
entries neutral. On train its forecasts equal the third generation's case by case (largest
difference 0.0 on every metric), and it stands in for it in paired comparisons.

**Reference baselines.** For the burner flames the lineage root of the new spec is the published
theory with its published constants and nothing from the PSI-159 tables (`baseline-theory`). For
BASS it is persistence: the tip keeps the speed the prefix shows, whatever the flow does.

**Candidates, frozen in git before any validation score of them existed.**

| Model | Candidate | Release | Train |
|---|---|---|---|
| `flex_methanol.wsp3` | D4 | `sha256:4e975d3b5b034c4529bf62823e45a7356f49490190bfe989cc663c9d1b8e2d02` | CRPS 26.9 µm mean, 16.4 median (D3: 29.9, 17.7); better or level in 24 of 26 sessions |
| `flex_heptane.wsp3` | D3 | `sha256:d3cf6b7c2569c455329c17009d1d30f055017dcc3f4763677b28d962d3771d6f` | 39.6 mean, 19.1 median (D2: 46.2, 21.8); better or level in 18 of 20 sessions |
| `acme_normal.wsp3` | D1 | `sha256:221a2ea22aca8a8c1e3813745ea566e591863fca13d8dd456c69c38986d49027` | extinction 0.063, flow termination 0.024 (second-generation D3: 0.052, 0.041; theory alone: 0.141, 0.075) |
| `acme_inverse.wsp3` | D1 | `sha256:19c156e60036d15f46fa92f7827d1122f8e595f6747ad1d5b7363463777422eb` | 0.036, 0.064 (second-generation D1: 0.057, 0.067; theory alone: 0.488, 0.247) |
| `bass_front.wsp` | D1 | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | CRPS 0.076 sample widths on the three tracked train burns (persistence: 0.242) |

**Selection, stated before validation is run.**

- *Droplets.* The train session rule of section 8. Both candidates pass it.
- *Burner flames.* The candidates are a different kind of model, so the train score is not the
  argument: the normal D1 is slightly behind the hinge law of the second generation on the train
  extinction score, as a less fitted law should be. The argument is that its laws are written in
  quantities that hold outside the sessions they were typed from. Each candidate replaces the carried
  one (normal: second-generation D3; inverse: second-generation D1) unless its validation extinction
  score is worse; then it stays in the tables as a negative result.
- *BASS.* There is no carried model. D1 is reported against persistence on the validation burns; no
  number is adjusted afterwards.
- Validation is read once, as aggregate tables. For the droplet and burner-flame models this is one
  more aggregate look at the same validation partitions, so their validation numbers are development
  numbers; the test partition is the only clean measure left, and it stays unread.

**BASS: measurement and scoring, fixed here.**

- *Tracks.* A track file (`wsp-bass-front-track/1`) holds, per top-view still, the downstream tip and
  the upstream base of the blue flame in sample widths, the fan history read from the video overlay,
  and the day's fan calibration. The four train tests are in `tools/wsp/bass_optical/tracks/train/`.
  The seventeen validation tests were measured by two measurement agents that knew nothing of the
  model, with the train method; their files are kept outside git (`runs/dev/bass_front/val_sealed/`).
  The author has read their inputs only (oxygen, sample label, band width, restrictor, igniter
  interval, calibration rows, the set of fan values) and no position, time or outcome. The
  measurers reported: fourteen tracks in sample widths, one in frame widths only (no top view in the
  archive), two with no flame track.
- *Reading rules* (`front_tools::rules`, not model parameters): flow speed from the day's calibration
  rows by linear interpolation, with a zero-flow row at fan 38 when a block has none; the sample class
  from the measured band width (below 44 px: the 1 cm sample, taken as 1.2/2.2 of the reference
  width); the window length from the file.
- *Scoring* (`wsp-dev front-feedback`). Cutoffs 5, 10, 20 and 40 s after the igniter is off. A case
  needs two flame stills at or before the cutoff and three within the next 60 s; those later stills
  are the targets. Per target: CRPS from the quantile levels 0.05, 0.25, 0.5, 0.75, 0.95 (the
  evaluator's formula), absolute error of the median, coverage of the 50 % and 90 % bands, all in
  sample widths. A case is the mean of its targets, a run the mean of its cases, the report the mean
  and median over runs. Paired difference to persistence over common cases with a bootstrap over
  runs. Events: the probability that the track has ended by 10, 20, 30, 45, 60 s after the cutoff
  against whether it had (ends by loss of tracking are censored). Ignition: the probability of no
  spread at 20 s from the conditions alone against tracks that never spread.
- The forecast is conditional on the flow history up to each forecast time. It says what the flame
  does under a fan schedule, not what the operator will do.

**Disclosures.** While reading the experiment table, three agents printed rows of tests outside
their lists (B1-B5; B1 and B2); none was used. The flame-spread reviewer's search results twice
showed one-line outcomes of BASS-type tests; they are not in its report and were not used. The
spherical-flame review reported one count over the whole CFI-G campaign; it is withheld from the
saved text and was not used. The ACME analysis agent read the line of the clock card that says which
fuel and pressure combinations the validation sessions hold; it did not use it. A reviewing agent
found that the inverse-flame flow in the stored conditions is the oxygen alone, not the injected
stream; the forward and every number typed afterwards use it that way.

**Outcome of the validation look** (aggregate tables, read once; [RESULTS.md](RESULTS.md)).

| Model | Candidate | Validation | Carried before | Decision |
|---|---|---|---|---|
| `flex_methanol.wsp3` | D4 | CRPS 44.8 µm, coverage 0.61 / 0.82 | D3: 39.5 µm, 0.71 / 0.90 (carry, same 59 cases) | **not adopted**; paired +5.3 [−1.4, +15.4] |
| `flex_heptane.wsp3` | D3 | 57.6 µm, 0.50 / 0.87 | D2: 46.1 µm, 0.71 / 0.97 | **not adopted**; paired +11.6 [−0.9, +33.0] |
| `acme_normal.wsp3` | D1 | extinction 0.163, flow termination 0.100 | second-generation D3: 0.159, 0.100 | **not adopted**: a tie, and the rule asked for not worse |
| `acme_inverse.wsp3` | D1 | 0.137, 0.088 | second-generation D1: 0.146, 0.097 | **adopted**; the best fitted model scores 0.143, 0.118 |
| `bass_front.wsp` | D1 | CRPS 0.117 sample widths | persistence: 0.234 | first BASS model; paired +0.117 [+0.051, +0.182] |

Two droplet candidates passed the train rule in 24 of 26 and 18 of 20 sessions and are worse on
unseen sessions, the median as well as the band. What they changed against the third generation was
of one kind: a class made sharper, bands made narrower, a heat-up term and a helium law added, each
because the train runs looked that way. Section 8 says that spread is not chosen on train; narrowing
it from train coverage broke that rule in spirit, and validation says so. The helium law has a flaw
that needed no validation to see: it multiplies the after-flame rate by `exp(3.85 x_He)`, typed from
30–35 % helium, and nothing bounds it at the hot rate for higher fractions. The candidates stay as
negative results with their releases.

The published theory alone, with no number from the PSI-159 tables, scores 0.206 on normal-flame
extinction (a constant guess: 0.214; the zero-shot model of phase 1: 0.219) and 0.346 on inverse
flames (constant: 0.252).

**One further candidate, registered before its validation score exists.** The broad component for an
unstable tracker is the only fourth-generation droplet change that acts solely when the prefix shows
trouble, and it was bundled with the changes above. `flex_methanol.wsp3` D5 is the third-generation
numbers plus that component and nothing else
(`sha256:0b57dba76eda173ffa57c8061cbb17b3873efb931de210ca944ce9e0cadef2c7`). On train its median is
level with D3 (17.7 µm), its mean 28.7 against 29.9, and it is better or level in 25 of 26 sessions.
Rule: it replaces D3 as the carried candidate only if its validation CRPS is not higher than the
carry's on the same cases. This is a second aggregate look at the methanol validation partition in
this generation and is reported as one.

The same construction for heptane (`sha256:b9a121a1a71109b90273900283f6626a38ced7058eb271f1f0cdd67d74d964cc`)
does not pass the train rule: the mean falls from 46.2 to 41.4 µm, no session is worse, but the
median rises from 21.8 to 22.1 µm. It is not taken to validation.

**Outcome of the second methanol look.** D5 scores 40.1 µm (coverage 0.72 / 0.90) against 39.5 µm
(0.71 / 0.90) for the carry of D3 on the same 59 cases: higher, so **not adopted**. The broad
component costs 0.6 µm on unseen sessions and buys nothing there; the tracker trouble it was typed
for is in the train runs. D3 stays the carried methanol candidate, D2 the heptane one. No further
droplet candidate is taken to validation in this generation.

**Three more candidates, registered before their validation scores exist.** None of them types a new
number from the train tables; each is a statement about models that already exist.

*Two laws answering together* (`blend.rs`: `acme_normal.wsp4`, `acme_inverse.wsp4`, keys
`acme_normal4`, `acme_inverse4`). The hinge law of the second generation and the growth clock of the
fourth were typed from the same train table and are different kinds of statement: an interpolation of
the observed lives, and a mechanism with few constants. A blend release holds one complete set of
numbers for each (`a.*`: a `flame.rs` release, `b.*`: a `flame3.rs` release) and two weights; its
forecast is the weighted mean of what the two member forwards return, each run unchanged. The mean
of two forecasts is never scored worse than the mean of their scores and is scored better by a
quarter of their mean squared disagreement, whatever the data: that is arithmetic, not a fit.

| Model | Candidate | Release | Members | Train: extinction, flow termination |
|---|---|---|---|---|
| `acme_normal.wsp4` | D1 | `sha256:8be14acec4f1f7b7a9870c631ba6bedbf746465c740f518810068fbcc175b2d3` | `wsp2` D3 and `wsp3` D1, weights 1 : 1 | 0.052, 0.028 (members: 0.052, 0.041 and 0.063, 0.024) |
| | baseline pair | `sha256:d579e6da25659b2e5b2def6e37fc0c49da29b2911e5e962acc3181d4099d53c7` | the two reference baselines | 0.151, 0.068 |
| `acme_inverse.wsp4` | D1 | `sha256:664694894f886dc110254c03542137218da295c87d075934a185e4370e3b980d` | `wsp2` D1 and `wsp3` D1, weights 1 : 1 | 0.041, 0.061 (members: 0.057, 0.067 and 0.036, 0.064) |
| | baseline pair | `sha256:989c7b1e14c3a097fc40a8be0b2558769c124a16149045041fd4b4bfc3724eb2` | the two reference baselines | 0.315, 0.198 |

The member numbers were not typed again: they are the stored values of the member releases, copied
statement by statement with a prefix by `tools/wsp/blend_edits.py`, which chooses and computes
nothing; a test compares the committed blend releases with the member releases tensor by tensor.
The only numbers typed for these models are the two weights. They are equal. The author had seen
the aggregate validation scores of all four members when typing them (the normal members tie; the
inverse `wsp3` member is ahead); equal weights are what one types knowing nothing, and no other pair
was tried. Rule, the one already in force for burner flames: a blend replaces the carried candidate
(normal: second-generation D3, 0.1592; inverse: `wsp3` D1, 0.1373) unless its validation extinction
score is worse. This is a second aggregate look at the two burner-flame validation partitions in
this generation.

*BASS without the oxygen boundary* (`bass_front.wsp` D2,
`sha256:0187941810a89347a9ea3953add9aa6058152bb643b009ca1ffa3e9471f216a7`). D1 with the part of its
quench law that places a boundary in oxygen withdrawn: that boundary rested on one train burn that
never spread and one that did, was typed as a sharp line, and scored worse on validation than saying
that every burn spreads. Nothing is typed in its place. What stays needs no boundary: the background
rate of a spreading flame, and a quench below 1 cm/s (the lowest quench velocity of the published
boundary at any oxygen up to 21 %) at the rate that puts a flame out five seconds after the fan is
turned off. The spread rate, overhang, burn-through time, encoder and bands are D1's. On the train
tracks the tip forecast is identical case by case (CRPS 0.076), the track-ended score is level
(0.003) and the spread-or-not score goes from 0.003 to 0.241: the one train burn that never spread
is no longer explained, and the model says so instead of guessing where the limit is. This
candidate is validation-informed and its validation numbers are not blind. Rule: D2 becomes the
release to serve only if its tip scores on validation are those of D1 and its track-ended score is
not worse; the blind result of the front model remains D1's. This is a second aggregate look at the
BASS validation tracks.

**Outcome of these three looks** (aggregate tables; [RESULTS.md](RESULTS.md)).

| Model | Candidate | Validation | Carried before | Decision |
|---|---|---|---|---|
| `acme_normal.wsp4` | D1 | extinction 0.155, flow termination 0.098 | `wsp2` D3: 0.159, 0.100 | **adopted** |
| `acme_inverse.wsp4` | D1 | 0.142, 0.092 | `wsp3` D1: 0.137, 0.088 | **not adopted** |
| `bass_front.wsp` | D2 | tip CRPS 0.117 (identical to D1 case by case); track ended 0.183; spread or not 0.121 | D1: 0.117; 0.178; 0.170 | **not adopted**: the track-ended score is worse |

The normal members disagree by about 0.15 (root mean square) on validation, as on train, and the
blend gains what the arithmetic says: 0.006 below the mean of their scores, 0.004 below the better
one. The inverse members share their self-extinction law and disagree by 0.03; their mean is the
mean of a better and a worse score. The pair of reference baselines, which holds no number from the
PSI-159 tables, scores 0.181 on normal-flame extinction (its members: 0.216 and 0.206; the best
fitted model: 0.185).

The BASS rule was written before the look and stands: D2 loses 0.005 on flames already burning and
gains 0.05 on spread-or-not, and it is not the release to serve. The oxygen boundary is wrong as a
statement about which samples carry a flame and still worth a little as a hazard.

**Carried after the fourth generation:** `flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2,
`acme_normal.wsp4` D1, `acme_inverse.wsp3` D1, `bass_front.wsp` D1. No further candidate is taken to
validation in this generation. The heptane D4 of the paragraph above was by mistake a member of the
group when the results document was regenerated; its validation row was removed by script without
being read, and the group was scored again without it.

## 10. Fifth generation (14 November): physics dossiers, independent checks, two new inputs

**What was asked.** The owner: rebuild the models from the physical mechanism of each experiment;
parameters are welcome when each has a physical meaning; do not overfit; analysis by
claude-fable-5-2 at high effort.

**What was read and measured.** Five physics dossiers ([research/](research/) `phys_*.md`), each
evaluated against train at literature constants by a delegated agent; an independent re-derivation
of four of them by fresh agents with their own code ([feedback/](feedback/) `verify_phys_*.md`); a
look-up of every quoted property value in standard sources (`research/properties_verified.md`);
train measurements of the flame cameras, of the raw channels and script files of the burner tests,
and of the burns of the earlier BASS campaign, which belongs to no partition. What each said, how
far the author checked it, and every known exposure are in the review dossier
([dossier/](dossier/README.md)), which a second author or reviewer should read before this file.

**Two new inputs, held in a private copy of the data root** (`runs/dev/data_ext/`, not in git; the
shared records and contracts are untouched). The flame-camera series of the droplet runs
(observations admitted up to the cutoff) and the planned stop of a burner test from NASA's script
parameter file (a condition known before ignition). Both exist there for train and validation runs.
The author reads that root only through the prefix builder and the scorer; nobody who writes
weights opens its validation series. Whether each is a legitimate input is argued in
`dossier/02_data_classification.md`; a reader who rejects one should look at the rows below that do
without it.

**Forwards.** Earlier forwards were not edited; every earlier release keeps its id.

| Forward | Models (keys) | What is new |
|---|---|---|
| `droplet5.rs` | `flex_*.wsp4` | flame cameras as evidence of the state; a heavy-tailed radiative clock. Its candidates R1 and F1 were not registered and are superseded |
| `droplet6.rs` | `flex_methanol.wsp5`, `flex_heptane.wsp5` (`flex_methanol5`, `flex_heptane5`) | the same, plus an unlit droplet evaporating on its stored heat, a flame that dies below its end diameter, and the cool-flame end as a law in oxygen partial pressure; all off by default, in which case it equals `droplet5.rs` exactly |
| `committee.rs` | `acme_normal.wsp5`, `acme_inverse.wsp5` (`acme_normal5`, `acme_inverse5`) | up to six member releases of the two burner-flame forwards with a weight each; a second weight vector and an offset for prefixes that carry the script's planned stop |
| `front2.rs` | `bass_front.wsp2` (`bass_front2`) | separate oxygen powers, separate growth and shrink times, a slow-down downstream, flow paths; all off by default, in which case it equals `front.rs` exactly |

**Candidates, frozen in git before any validation score of them exists.** Train numbers are from
`wsp-dev feedback` on the private data root (droplets: CRPS in micrometres, mean / median; burner
flames: integrated Brier scores of extinction / flow termination over 6 to 120 s).

| Model | Candidate | Release | What it is | Train | Carried on the same cases |
|---|---|---|---|---|---|
| `flex_methanol.wsp5` | P1 | `sha256:7884e61c9995d40b7f12f140a68b782aa02a811e774143cd128f6e71e25c141a` | self-cooling evaporation with constants from property tables; critical diameter capped above 18 % oxygen and falling with pressure above 1 atm; no pressure term in the water extinction; flame cameras; heavy-tailed clock | 28.7 / 17.5 | 29.9 / 17.7 |
| | P0 | `sha256:2e4c55f44b78b5eaae555b879358c22fafc675002d6f441255acbe8af4989397` | the same without cameras and heavy tail | 29.7 / 18.2 | 29.9 / 17.7 |
| `flex_heptane.wsp5` | P0 | `sha256:c2c7b700867f3fab86e956100db6aed3db100438f6584347c9af5be6b75ad760` | cool-flame rate without oxygen or pressure dependence below 1 atm; cool-flame end in oxygen partial pressure; a flame born below its end diameter dies away (the helium patch removed); helium in the radiative clock | 43.9 / 19.8 | 46.2 / 21.8 |
| | P1 | `sha256:c27df7bb7f70bc3ce8cda22a1edc5c1b4a8b8c94b7760de61cedf901e40de360` | P0 plus flame cameras and heavy-tailed clock | 44.6 / 21.0 | 46.2 / 21.8 |
| `acme_normal.wsp5` | P1 | `sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7` | the carried pair of laws with the burner-limit weights of the growth member corrected; reads the planned stop when the prefix carries it | 0.0536 / 0.0215 with the planned stop; 0.0518 / 0.0274 without | 0.0517 / 0.0275 |
| `acme_inverse.wsp5` | P1 | `sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2` | the carried law (weight 2) and a scenario that holds a marginal flame at a high oxygen flow (weight 1); reads the planned stop when the prefix carries it | 0.0281 / 0.0341 with the planned stop; 0.0271 / 0.0608 without | 0.0357 / 0.0639 |
| `bass_front.wsp2` | P1 | `sha256:d53e2944b62863014b875a8023b57b50ca28095f4d69c86e1a0c28fcf49d429b` | D1 with the narrow sample corrected from two burns of the earlier campaign; for the 2 cm sample it is D1 number for number | tip CRPS 0.076 sample widths (identical to D1); on three burns of the earlier campaign 0.157 against 0.220 | 0.076 |

Carry releases (the carried numbers on the new specs, identical forecasts on train, the comparison
partners): `sha256:2181b643430a31aefa8e774e9b9523f6f09b343957f1bdcae04ea435512545be`, `sha256:03e0f63cf88c1b2c626e7726356746dd16d673a3a208cca5461c9d1ec360bf0b`, `sha256:dce4d7f0b3ce75b6908d8be60313ecf7f7b94d0c25aefd79120af08a71f94a35`, `sha256:ff3449c4123f0ae5e1434f72b9a031588bb93a39a6514369624a906337035c26`, `sha256:c50c7144e433799b116829cc0f5ff21dbc1e412c9ec38524e2bea6c8c7cba4bc`.

Typed, tried on train and **not** taken (each stays in the repository or in the edits text as a
record): a pressure term for inverse flames (train extinction 0.041 against 0.036); for the fabric
flame a slower growth of the flame, a weaker oxygen dependence and flow paths (train tip score
0.082 and 0.085 against 0.076); a flattening of the methanol critical diameter from 15 % oxygen
(train worse by 3 micrometres; replaced by the cap from 18 %); the self-cooling law for n-heptane
(its size scaling fails after a cool flame, by the independent check). The total-flow scenario for
inverse flames (`acme_inverse.wsp3` D2, `acme_inverse.wsp5` D1) was weakened by a recount and is not
registered.

**Selection, stated before validation is run.**

- *Droplets.* The train session rule of section 8 first. Methanol P0 fails it (its median is
  0.5 micrometres above the carry's; 85 % of sessions better or level) and is not taken to
  validation. Methanol P1 passes (81 %), heptane P0 (95 %) and P1 (90 %) pass. On validation each
  candidate is compared with the carry of the carried model on the same cases, on the private data
  root: it replaces the carried model when its CRPS is not higher. If both heptane candidates pass,
  the one with the lower validation CRPS is taken; that choice between two is a use of validation
  and is said here beforehand. A camera candidate is adopted for prefixes that carry camera series;
  on the shared records, which carry none, the carried model stays unless the candidate without
  cameras was adopted.
- *Burner flames.* The rule in force: a candidate replaces the carried one unless its validation
  extinction score is worse. It is applied twice, to the same release: with the planned stop (private
  data root) and without it (shared records). The flow-termination score is reported beside it.
- *BASS.* P1 becomes the release to serve if its tip score on validation is not higher than D1's and
  its track-ended score is not worse. It is not blind: the author had seen D1's and D2's aggregate
  validation tables.
- Validation is read once, as aggregate tables. For every family this is one more look at a
  partition that has been looked at several times (`dossier/07_limits_and_disclosures.md`, section
  7.2): the numbers are development numbers. The test partition stays unread.

**Disclosures of this generation** are the entries X-01 to X-19 of the dossier's exposure ledger. In
short: the experiment dossier of 13 November quotes published results of the BASS-II fabric tests
(X-01); the author read the conditions of the validation runs, and conditions of consecutive tests
in one chamber fill encode the previous test's outcome (X-09, X-18); helpers of three physics
dossiers saw fragments of published per-test results and report having discarded them (X-16, X-17,
X-19).

**Outcome of the registered look** (14 November, after the candidates were committed in `00635b2`).
Tables: [RESULTS_GEN5.md](RESULTS_GEN5.md). Validation was read as those aggregate tables and nothing
else.

| Model | Candidate | Validation | Carried model on the same cases | By the rule above |
|---|---|---|---|---|
| `flex_methanol.wsp5` | P1 | CRPS 42.5 µm (65 cases) | 39.5 | **not adopted** |
| `flex_heptane.wsp5` | P0 | 47.2 µm (43 cases) | 46.1 | **not adopted** |
| | P1 | 48.4 µm | 46.1 | **not adopted** |
| `acme_normal.wsp5` | P1, planned stop known | extinction 0.1452, flow termination 0.0747 (84 cases) | 0.1553, 0.0984 | **adopted** |
| | P1, shared records | 0.1552, 0.0968 | 0.1553, 0.0984 | **adopted**, level with the carried model |
| `acme_inverse.wsp5` | P1, planned stop known | 0.1348, 0.0478 (54 cases) | 0.1606, 0.0939 | **adopted** |
| | P1, shared records | 0.1288, 0.0880 | 0.1606, 0.0939 | **adopted** |
| `bass_front.wsp2` | P1 | tip CRPS 0.1137, track ended 0.1976 (12 burns) | 0.1168, 0.1780 | **not adopted**: the track-ended score is worse |

Carried after the fifth generation: `flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2 and
`bass_front.wsp` D1 as before; `acme_normal.wsp5` P1 `sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7` in place of `acme_normal.wsp4` D1;
`acme_inverse.wsp5` P1 `sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2` in place of `acme_inverse.wsp3` D1.

What the numbers do and do not show. The droplet candidates were better than the carried models on
train and are 1 to 3 µm worse on validation, with every paired interval containing zero (methanol:
carry minus P1, −3.1 [−12.1, +4.6] µm over nine days): no gain is shown, and the rule refuses. The
normal-flame release without the planned stop is the carried pair of laws with one corrected pair of
weights and scores as it does; its gain is the planned stop. The inverse-flame release gains at
extinction from its second scenario, with or without the planned stop; the planned stop halves its
flow-termination score and costs 0.006 at extinction. Both burner-flame validation partitions hold
two independent sessions, so the paired intervals are not informative and the adoptions rest on
point estimates. The carried inverse model scores 0.161 here and 0.137 in section 9: there it was
scored on the 49 cases the fitted baselines also answer, here on all 54; its sealed forecasts and
those of its carry release are identical on every validation prefix (checked on the forecasts, with
no outcome read).

How the look was run, where it differs from a plain run of `tools/wsp/dev_eval.py`:

1. *Droplets and the vault.* The evaluator's shared vault holds the continuations of the shared
   records, and its write-once check refused the continuations of the private copy of the data root
   (whose runs carry the flame-camera series): the first run stopped there, after sealing, with no
   table written or read. The private copy's validation continuations were then deposited by the
   evaluator's own command into a vault of their own (`IGNIS_HOLDOUT=runs/dev/holdout_ext`; 71
   methanol and 43 n-heptane cases), and the look was run against it. Nobody opened either vault.
   Nothing in the shared vault was written or changed.
2. *Burner flames, two tables from one comparison.* The releases were sealed on the private copy
   (planned stop in the prefix) and on the shared records. The evaluator keys a sealed forecast by
   run and prefix, so `ignis eval compare` now returns both as one table of twice the cases.
   `tools/wsp/analysis/cmp_split.py` splits it by prefix into the two tables above; for the half the
   evaluator had scored alone it reproduces the evaluator's numbers to 6e-17. It reads per-case
   scores of validation and prints aggregates only.
3. *The fabric flame* was scored once on the sealed tracks: the third aggregate look at them.
4. The planned stop is an input of the private copy only. It is not in the shared records or
   contracts, and whether a forecast may know it is for the data and evaluation lanes to accept or
   refuse (`dossier/02_data_classification.md`). On the shared records the two adopted releases
   answer as in the rows "shared records".

## 11. Transfer tests on investigations outside the partitions

**Why.** The owner's goal of 14 November is models usable on all of NASA's combustion investigations.
PSI lists 19 flight investigations; the models were written on four. The first check costs no
validation look: run the frozen models, unchanged, on investigations they have never met. Such data
are outside every partition and may be read in full, so the only protection a result has is that
the test is written down before its outcomes are read. Each test of this section is registered
here, in a commit that precedes its scoring.

### 11.1 ACME Flame Design (PSI-10), registered 14 November

Written after the feasibility report ([feedback/flame_design_transfer_feasibility.md](feedback/flame_design_transfer_feasibility.md)),
which gives formats, counts and ranges of conditions and no outcome of any test. The author has not
opened the trial endpoints file that report names.

**Releases, frozen at commit `53402dc`.** `acme_normal.wsp5` P1
`sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7` and `acme_inverse.wsp5` P1
`sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2`. Beside them: the models
they replaced (carry releases `sha256:dce4d7f0b3ce75b6908d8be60313ecf7f7b94d0c25aefd79120af08a71f94a35`
and `sha256:ff3449c4123f0ae5e1434f72b9a031588bb93a39a6514369624a906337035c26`), the reference sets
that hold no train number (`sha256:6044b69bd5d10a150734bea2b6d549a8903dbdc49ddc911f7ff1b6bf2c0b2bff`,
`sha256:3d7d010fd8488d7071e756719f4bf8e7a2fb1f8a768179b063f55ec38d226424`), and an incidence
baseline: for each cutoff, the fraction of CFI-G train cases of the same family that had ended by
each horizon, by cause. The baseline is computed once from CFI-G train and not from Flame Design.

**Tests.** From the experiment tables and the file listing only: (1) a constant-flow row with every
input numeric; (2) all features inside the release's declared domain, with the initial pressure, the
initial compositions and the mixture fraction recomputed from the initial composition (the table's
mean values use the final composition and are not read); (3) the sensor file and the timeline file
exist. That gives 132 normal-flame tests in 13 sessions and 100 inverse-flame tests in 8 sessions.
The five tests used to try the endpoint script stay in.

**The fuel.** Flame Design burns ethylene on 310 of its 327 tests. The models know ethane, propane
and n-butane and refuse any other fuel. Two arms, fixed now:

- **Arm A, unchanged models:** the 17 propane normal-flame tests of session 21301, and the count of
  ethylene tests the models refuse. A refusal is the model's answer for those tests and is reported
  as such.
- **Arm B, a surrogate fuel:** every included ethylene test is run with ethane's indicators
  (primary), and with propane's and n-butane's as a sensitivity. This is a declared extrapolation,
  not an application inside the domain: 115 normal and 100 inverse tests.

Each arm is run without the planned stop (primary, as on the shared records) and with it (the
script's stage sum and the typed offset of 0.4 s).

**Endpoints.** Measured from the raw channels by `tools/wsp/ext/flame_design.py`: onset where the
radiometer sum rises above its level before ignition; flame end where the photomultiplier sum is
back at its dark level for 0.5 s; flow change at the first commanded departure from the test's
setpoints; extinction if the flame end precedes the flow change by more than 0.5 s, operator
termination otherwise. Before it is run on Flame Design the same method is checked on CFI-G train
tests against their table burn times and labels. Its thresholds are not changed after a Flame
Design outcome has been computed.

**Scoring.** Cases are (test, cutoff) at 1, 3 and 5 s after onset with the flame still burning.
Integrated Brier score over 6 to 120 s, for extinction and for operator termination, as in
development. Paired differences to the incidence baseline and to the carry release; intervals by
bootstrap over sessions. The scoring chain (prefix files, `ignis wsp forward`, the score) is first
run on CFI-G train and has to reproduce the train numbers of `wsp-dev feedback` for the same
releases (normal P1 0.0518 / 0.0274, inverse P1 0.0271 / 0.0608, without the planned stop); if it
does not, nothing is run on Flame Design.

**What each result will be taken to mean.**

- Arm A is one session of 17 tests: its numbers are reported and no conclusion is drawn from them.
- Arm B: the models *carry over to ethylene as a surrogate* if the extinction score of P1 is lower
  than the incidence baseline's and the session-bootstrap interval of the difference excludes zero;
  they *do not* if it is higher with an interval that excludes zero; otherwise the test is undecided.
- No model is revised from these outcomes in this generation. A later candidate that uses them
  treats Flame Design as development data and says so.

**Known threats, from the feasibility report.** The delay between the ignition stamp and the flame
is 0.4 to 2.6 s on the trial tests against 2.4 s in CFI-G. A scripted stop came about 2 s after the
script's stage sum, so the planned-stop runs may misplace it. The adiabatic temperature is taken
from the table and may carry a little after-test information. Table flows were not checked against
the commanded setpoints. The burner limit of each session is not known.

**Amendment of 11.1, made before any Flame Design score was seen (14 November).** The evaluation
groups of Flame Design were frozen after the registration above
([atlas/splits/PSI-10.json](atlas/splits/PSI-10.json); a group is a test day; normal and inverse
flames are split separately, with the project's rule and seed). The tests of its four test days
(normal flames 19123, 19200 and 19206; inverse flames 21361) are left out of this transfer test and
stay unread, so that Flame Design keeps a part nobody has scored. Everything else stands: the arms,
the endpoints, the metric and what each result will be taken to mean. The remaining tests are
reported together, as registered, and also separately for the days frozen as train and as
validation. Arm A is unchanged: day 21301 is a validation day. The agent that scores was told so
before it reported anything.

### 11.2 Evaluation groups of the other investigations (frozen 14 November)

The owner's rule, that evaluation groups are frozen before outcome data are viewed, is applied to
the 15 investigations the models have not been written on. `tools/wsp/atlas/freeze_groups.py` read
the test catalogue (conditions and group ids; the outcome values are in another file, which it does
not open) and applied the project's split rule and seed; it reproduces the project's own split of
the CFI-G normal flames from the same inputs. One file per investigation is in
[atlas/splits/](atlas/splits/README.md). Nine investigations are split 60 / 20 / 20 by test day
(ten strata, because Flame Design has separate normal and inverse strata).
Six have fewer than three groups, or only a test matrix, and are held whole: no outcome of them is
read per test by anyone who types a number, and a model is scored on them only as an aggregate in a
registered look. From now on an author reads outcomes of train groups only.

Known before this freeze, by agents and not by the author: the agents that built the catalogue and
the feasibility reports saw outcome values of these investigations while reading their tables; none
is in a file the author reads, and the catalogue keeps outcome values in a separate file.

### 11.3 Outcome of the Flame Design transfer test (14 November)

Report: [feedback/flame_design_transfer_result.md](feedback/flame_design_transfer_result.md);
aggregate tables in `runs/dev/data_all/work/transfer_acme/fd/report_tables.txt`. The author read
aggregates only.

**The chain.** On CFI-G train the scoring chain returns the numbers of `wsp-dev feedback` to within
1e-16 for all six releases (normal P1 0.051752 / 0.027399, inverse P1 0.027091 / 0.060816), and its
prefix file for one case equals the one the Rust path stores. The endpoint method, with its
thresholds unchanged, agrees with the table label on 148 of 153 CFI-G train tests and puts the flame
end within 2 s of the table burn time on 140; nine differ by more than 5 s.

**Tests scored, after the amendment.** 103 normal-flame tests in 10 sessions (86 ethylene, 17
propane) and 80 inverse-flame tests in 7 sessions. The 29 normal and 20 inverse tests of the four
test days were neither measured nor scored.

**Arm A, unchanged models.** Every ethylene case is refused at the fuel condition (258 normal and
236 inverse cases) by P1, by the carry and by the reference set. The 17 propane tests of day 21301
(one session; every case ended with the flow being stopped) score, extinction / flow termination,
0.0522 / 0.5007 for P1, 0.0517 / 0.4862 for the carry and 0.2704 / 0.5802 for the incidence
baseline; with the planned stop P1 scores 0.0081 / 0.0980. No conclusion is drawn from one session,
as registered.

**Arm B, ethylene with ethane's indicators, without the planned stop (primary).**

| | P1 | Carry | Reference set (no train number) | Incidence baseline | P1 minus baseline, extinction |
|---|---|---|---|---|---|
| Normal flames, 86 tests, 9 sessions | 0.1203 / 0.3166 | 0.1201 / 0.3138 | 0.1591 / 0.3232 | 0.2473 / 0.3336 | −0.1270 [−0.1816, −0.0722] |
| Inverse flames, 80 tests, 7 sessions | 0.1466 / 0.2599 | 0.1466 / 0.2599 | 0.1453 / 0.4681 | 0.2460 / 0.3413 | −0.0994 [−0.2089, +0.0116] |

By the rule written beforehand: **`acme_normal.wsp5` P1 carries over to ethylene as a surrogate;
`acme_inverse.wsp5` P1 is undecided.** The sensitivity runs (propane's and n-butane's indicators)
and the runs with the planned stop give the same two answers.

**With the planned stop** (ethane's indicators): normal 0.0823 / 0.0887, inverse 0.1594 / 0.1162.
Against the same release without it, the flow-termination score falls by 0.2251 [−0.3095, −0.1226]
for normal flames and by 0.1437 [−0.2118, −0.0644] for inverse flames; the extinction score falls by
0.0378 [−0.0634, −0.0163] for normal flames and rises by 0.0128 [+0.0031, +0.0227] for inverse
flames.

**What the result does not show.**

- P1 is not better than the model it replaced on ethylene: worse by 0.0002 for normal flames,
  identical for inverse flames, whose second scenario does not act in these conditions.
- For inverse flames the reference set, which holds no train number, scores 0.1453 at extinction,
  as well as P1. For normal flames it scores 0.1591, between P1 and the baseline. Much of what
  carries over is the structure of the law, not the numbers typed from CFI-G.
- The baseline is weak here: extinction is rarer in Flame Design than in CFI-G train (84 of 258
  normal cases, 57 of 236 inverse).
- The inverse verdict turns on the interval. The pooled interval, fixed as primary before any
  outcome was computed, includes zero; the evaluator's session-equal statistic gives −0.1210
  [−0.2136, −0.0124]. Undecided stands.
- The endpoint labels of Flame Design were not adjudicated: the sensor label and the script's own
  account disagree on 18 of 176 tests.
- The threats listed at registration were not resolved.
- This test spent one aggregate look at the train and validation days of Flame Design. Its test
  days are unread.

**Process notes.** The `target/` folder was deleted and rebuilt by another process during the run;
the agent finished with the copies in `runs/dev/bin/` after showing that they return the same 1,242
artifact ids on CFI-G train. The amendment reached the agent before any Flame Design endpoint had
been measured.

### 11.4 Astra source-bound FLEX TRAIN admission and point comparison

Registered 14 November 2026 by Astra, before executing this comparison. This is
a TRAIN-only development comparison, not a validation look or an all-19
qualification. The integration candidate is **data-informed**, wrapping unchanged
zero-shot physical literals. No new physical parameter is selected by scoring.

**Exact candidates**, model `flex.property_droplet.source_bound.v1`, stored in
`runs/dev/astra_flex_source_bound_registry`:

- Methanol: `sha256:357112c4518283f576e655bd0de203e5690b7a61cc26dcada26e2f82976e7bb4`.
- Heptane: `sha256:a93aa81dd009a0bf1ebf596c6ffbf4e3f676286db3248b5ce5b37f3b0b9cbd32`.
- Forward identity: `sha256:a566a727615492a3807a287d0b882fd9929a084bb0ab3de6426818742ea2d1a2`.

**Adopted baselines**, read unchanged from `artifacts`:

- Methanol `flex_methanol.wsp2`: `sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce`.
- Heptane `flex_heptane.wsp2`: `sha256:43f7871f778040550291789ca97e319729675a9dd0df9c9529525c1805ad588d`.

**Population and timing.** Use the frozen TRAIN membership from the existing
`configs/models/flex/wsp2/{methanol,heptane}/binding.json` splits. Every run has
cutoffs 1, 3 and 5 seconds, before exclusions: 68 methanol runs/204 cases and
49 heptane runs/147 cases. No at-risk filter derived from future outcomes.
Missing records/prefixes, unsupported conditions, refusals and unscoreable cases
stay in the accounting. Predict at cutoff+0.1 through cutoff+10 seconds with
0.1-second spacing. No observation outside that window is scored.

**Inputs and known admission limitation.** Freeze
`configs/models/droplets/source-bound-train-astra/input_manifest.json` with this
registration. It contains zero admitted packets because no complete initial-state
and independently supported material/transport bundle exists. Consequently zero
candidate predictions are expected before any score is read. This execution
tests actual adapter admission and supplies baseline/persistence reference scores;
it cannot establish the new law's accuracy when the paired set is empty.
Any later populated manifest or numerical assumption requires a new registration.

**Routes.** Candidate, adopted baseline and last-valid-diameter persistence use
strict prospective prefixes without endpoint-derived activity markers. The adopted
baseline requires such a marker and may refuse. Therefore also report a separate
`baseline_inherited_activity` route using its unchanged historical
`build_run_prefix` activity rule. Label its different information policy explicitly;
do not use that marker as evidence for the candidate's reaction-sheet assumption.
No validation or test records are materialized. The TRAIN reader filters frozen
identities before domain-record deserialization.

**Metrics.** Liquid-diameter point MAE: linearly interpolate forecast values to
valid observation times, without interpolating or extrapolating target values.
Use the same observed targets for paired comparisons. Primary aggregation is
case mean, then run mean, then equal experimental-session mean. Report equal-run
and observation-weighted means as secondary metrics, units, all denominators,
and refusal reasons. The simple baseline persists the last valid pre-cutoff
diameter. The candidate has no predictive distribution: no CRPS, coverage or
calibration claim is fabricated. Reference outputs use these exact rules and
need not equal historical CRPS or differently weighted MAE.

**Interpretation.** A smaller paired MAE is a descriptive TRAIN gain only;
coverage must accompany it. Lower paired error with lower coverage is not an
unqualified win. An empty paired set has null scores/differences and verdict
`not_comparable_no_candidate_predictions`, never a tie, infinity or victory.
Zero accepted cases establishes current operational unusability under this
evidence policy, not that the physical law is inaccurate.

**Execution and identity.** The dedicated example
`crates/ignis-wsp/examples/astra_flex_train_compare.rs` requires `--execute`,
`--registration-commit`, this protocol, both exact candidate IDs, the candidate
registry, the committed input manifest and a new output path. Its companion
`tools/wsp/astra/flex_comparison_design.md` specifies implementation details.
The runner checks committed registration/code/input bytes before reading data.
Record that commit, source identities and output hash in the results. Run only
after required local gates and registration commit; no code/input change after
scores without a separately recorded amendment and rerun identity.


### 11.5 Astra BASS matched-window operator, frozen weights, TRAIN only

Registered 15 November 2026 before scoring. Data-informed correctness candidate
`bass_front.wsp4` inherits every numeric tensor from joint Astra
`sha256:79d27b0203a7935f1bdaa275e2f508f24f0d2a52edd67ebe38fd327647c5a1e0`.
The new forward averages modeled tip components and delayed base positions at
the observation timestamps and carries the window anchor consistently. No
physical law, inherited weight or scoring function is changed.

Full preregistration: `dossier/ASTRA_BASS_OPERATOR_20261115.md`; exact three
release IDs, data/code hashes and executable snapshot:
`feedback/astra-bass-operator-20261115/freeze.json`. Compare against adopted
Fable and joint Astra with the unchanged `front-feedback` protocol on the four
TRAIN track files, 12 scoreable cases from three burns on one day. Primary:
tip CRPS; guard: track-ended Brier; report coverage/refusals and all paired
differences. No independent-day interval is available and no automatic
adoption follows. This is not a quench-law fix or an experiment on validation.
Run once after the protocol commit; retain the result without weight revision.

## 12. The night of 15 November: who scores, how new data is used, how a model is revised

Written before any of this section's work produced a score.

**12.1 Who scores.** On 15 November the owner removed the separate evaluation lane from this work.
From here the author's lane runs every comparison. What protected a result stays: a candidate and
the rule that will judge it are committed before its score on unseen data exists; a score is taken
once; aggregates only reach the author for groups that are still held; `holdout/` and
`runs/dev/holdout_ext/` stay closed and `--final-test` is not passed until the one final run of
section 12.5.

**12.2 Test, then learn.** New data from NASA's archive enters in batches (an investigation, or a
set of test days). For each batch, in this order: the releases are frozen and named; the batch is
measured by an agent that sees no forecast, with its method fixed on train tests; it is scored once
(the *blind score*); only then may its train groups be studied for the next revision. Its
validation groups judge that revision once. Its test groups stay unread until 12.5. For an
investigation that is held whole, the blind score is its only use until 12.5.

**12.3 How a model is revised ("typed back-propagation", asked for by the owner).** The analogue of
a training step, with the author in place of the optimiser:

1. *Forward.* The frozen release is run on the train groups; the loss of every case is kept.
2. *Decompose.* Loss by session, regime and horizon; centre against spread (is the median wrong, or
   the width of the band).
3. *Attribute.* A script perturbs each physical block of numbers by a stated small step and reports
   how the loss of each case moves. It says which numbers *can* explain which errors. It writes no
   weight and proposes no value.
4. *Reason and type.* The author moves a number only if (a) the error pattern holds across several
   independent sessions, not one test; (b) the new value stays inside the range its sources allow;
   (c) the step is damped, about half of what the attribution suggests; (d) it is typed to two or
   three significant figures; (e) a physical reason is written beside it. An error that no number
   can explain inside its allowed range is a missing mechanism: that is a new forward, not a patch.
5. *One step per batch.* An epoch is new independent data, never another pass over the same cases.
   The next unseen batch is the check on the step.

Everything written this way is **data-informed** and is named so. Widths of forecast bands are set
from the blind errors of earlier batches, never from the batch being scored.

**12.4 Adoption.** As in sections 8 to 10: a candidate replaces the carried model of its family if
its primary validation score is not worse; a rule written before a look stands on a near miss. Where
a family now has several investigations, the primary score is the equal-weight mean over
investigations of the session-mean score, so that one large investigation cannot carry a model that
fails on another.

**12.5 The final run.** Once, at the end, on releases frozen in git: the test groups of every
investigation that has them, the held investigations not yet spent, and the project's sealed test
partitions. A scoring agent runs it and returns aggregate tables. Nothing is revised afterwards in
this generation.

Registrations of this night follow as 12.6 onward, each committed before its score.

### 12.6 Sixth generation of the normal burner flames: P2 and P3, registered 15 November

**What was done.** A train-only analysis of `acme_normal.wsp5` P1
([feedback/acme_normal_gen6_train_analysis.md](feedback/acme_normal_gen6_train_analysis.md),
claude-fable-5-2 at high effort; the author reran its tables of limit trips and stop times and
re-evaluated the member laws at the corrected inputs). Two new strict forwards, written by engineers
to the author's specification and verified by a second engineer (288 tests; 80 committed releases
reassembled with unchanged ids): `flame4.rs` (`acme_*.wsp8`: the hinge law with the burner-heating
law of `flame3.rs` in place of its own burner-limit clock) and `committee2.rs` (`acme_*.wsp7`: ten
member slots, slot weights by the era of the test read from the date in the run id, three readings
of the planned stop).

**Candidates, both data-informed (train).** Members: hinge law `acme_normal.wsp2` D4
`sha256:08ad7337…5901` (two statements corrected that stood on mis-tabulated rows) and D4-NOSTOPS;
growth clock `acme_normal.wsp3` D3-EARLY `sha256:d42cd550…3995`, D3-LATE `sha256:02cad964…e531`
(butane corrected; the burner limit of one era each) and their NOSTOPS copies; `acme_normal.wsp8`
D1-EARLY `sha256:b97791ba…d52c`, D1-LATE `sha256:d184b6d1…9554` and their NOSTOPS copies (no number
typed: hinge tensors of D4, burner tensors of D3).

| Release | Id | What it is |
|---|---|---|
| `acme_normal.wsp7` P2 | `sha256:ec1aa02e991f8e3feec469edbb374c54dd135faa1c58a4e8ce6bb7d03fae9ba0` | the carried pair of laws; burner limit by the date of the test (early through 30 August 2021, late from 27 September, even odds between, 0.15 late without a date); planned stop at the stage sum plus 0, 1, 2 s with weights 0.25, 0.5, 0.25; three statements corrected |
| `acme_normal.wsp7` P3 | `sha256:96a91f15ab94ff6a0b32e51f34e5c1f71d8377ae618b4cdd331bdb1b20fdaa34` | P2 with the hinge member answering through the burner-heating law |
| `acme_normal.wsp7` carry of P1 | `sha256:2a898731e18dcefefff39146810fb4736392fe24aab02b40079202bc316b0963` | the carried model on the new spec, for paired comparison (train scores equal P1's to four decimals) |
| `acme_normal.wsp7` baseline set | `sha256:dd05f4f8b6157cdbbccc0f9b0168284f6425df77c01e1a16db5c79cae406a513` | reference baselines |

**Train, real forwards, 210 cases (extinction / flow termination).** Shared records: carry 0.0518 /
0.0274, P2 0.0526 / 0.0281, P3 0.0522 / 0.0240. With the planned stop: carry 0.0536 / 0.0215, P2
0.0542 / 0.0166, P3 0.0538 / 0.0140. These equal the author's Python transcription of the forwards.
On train the gain is in flow termination; extinction is level to slightly worse, because the
corrected laws are scored on rows whose tabulated inputs are wrong.

**The look, and why it is wider than before.** The CFI-G validation partition of normal flames has
two sessions. Two sessions cannot tell models of this closeness apart; the fifth generation said so.
Flame Design (PSI-10) is the same burner on another campaign, its outcomes have not been read by the
author per test, and P2 and P3 were typed without them. So the look has two parts, read together:

- (a) CFI-G validation, 168 cases in 2 sessions, as in section 10, without and with the planned stop;
- (b) the Flame Design transfer set exactly as registered in 11.1 with its amendment (arm B, ethane
  as the surrogate for ethylene, plus the propane session; the four test days stay unread; endpoints
  by `tools/wsp/ext/flame_design.py` with its thresholds unchanged), without and with the planned stop.

**Score.** A forecast is one table over two causes; its proper score is the sum of the two
integrated Brier scores (extinction + operator termination, 6 to 120 s). The decision score is the
mean over the two investigations of that sum (case mean within an investigation, as the tables quote).

**Rule, written before either part was run.** A candidate replaces the carried model if (i) its
decision score is not higher than the carry's, and (ii) the mean over the two investigations of its
extinction score alone is not higher than the carry's. If both candidates pass, the one with the
lower decision score is taken; that choice between two is a use of validation and is said here
beforehand. The rule is applied twice: on prefixes without the planned stop (what the shared records
give) and with it. The verdict the older rule would have given (CFI-G validation extinction alone,
section 10) is reported beside it. Why the rule changed: the older one judged half of the forecast
on two sessions.

**Known threats.** The burner limit of Flame Design's sessions is not known; the committee treats
every date through 30 August 2021 as the early era. The planned-stop offset was typed from CFI-G
normal flames; 11.1 noted a scripted stop about 2 s after the stage sum on Flame Design. Ethylene
runs under ethane's indicators, a declared extrapolation.

**Outcome of 12.6 (15 November).** Both parts were run once after commit `2750ab3`; tables in
[RESULTS_GEN6.md](RESULTS_GEN6.md). Decision score without / with the planned stop: carried 0.3538 /
0.1902, P2 0.3503 / 0.1872, P3 0.3416 / 0.1862. Extinction mean: carried 0.1322 / 0.1078, P2 0.1323 /
0.1088, P3 0.1306 / 0.1075. P3 passes (i) and (ii) on both paths and replaces `acme_normal.wsp5` P1.
P2 fails (ii) on both paths and is not adopted. The older rule gives the same verdict.

### 12.7 Droplet transfer tests on FLEX-2 (PSI-68) and CFI (PSI-39), registered 15 November

Written before any FLEX-2 frame was downloaded and before any CFI series was read by the author.
Feasibility, conditions only: [feedback/flex2_catalogue.md](feedback/flex2_catalogue.md),
[feedback/cfi_transfer_feasibility.md](feedback/cfi_transfer_feasibility.md). The scoring chain
`tools/wsp/ext/transfer_droplet.py` reproduces `wsp-dev feedback --partition train` on FLEX train
exactly (largest difference 0.0 on every per-case metric, six releases).

**Groups.** FLEX-2 had no per-test list and was held whole. The catalogue of 15 November dates 591
of its 596 rack test points; the project's split rule and seed, unchanged, applied to those dates
give 64 train, 21 validation and 21 test days ([atlas/splits/PSI-68-days.json](atlas/splits/PSI-68-days.json),
`tools/wsp/atlas/freeze_flex2_days.py`). Frames of test days are not downloaded before the final
run. CFI keeps its split of 14 November.

**Measurement (FLEX-2 has no processed diameters).** `tools/wsp/media/droplet_diameter.py`, the
tool of the pilot, with its code and thresholds as they stand when the pilot report is committed. On
six FLEX train tests against NASA's own processed series it gave a diameter ratio of 0.995 to 1.005,
a burning-rate ratio of 0.996 to 1.10 and an RMS difference of 9 to 32 micrometres after the first
second. The measuring agent sees no forecast. A series enters the score only if the tool's own
quality flags accept it; that rule is the pilot's and is not changed after a FLEX-2 series exists.

**Batch 1** ([atlas/splits/PSI-68-batch1.json](atlas/splits/PSI-68-batch1.json), frozen with the
groups, by conditions and file sizes only): 48 single free droplets of a pure fuel inside the
declared domain, on train and validation days, 70.7 GB of backlit frames: n-heptane 10, iso-octane
2, n-octane 12, n-decane 24.

**Arm H, the adopted model unchanged.** The 10 n-heptane tests (3 days, all train days by the
freeze). Releases: `flex_heptane.wsp2` D2 `sha256:43f7871f…588d` (adopted, data-informed on FLEX
train), `flex_heptane.wsp` W0 `sha256:6ea680d1…6d8f` (zero-shot), the D-squared law of the same spec
(`configs/models/flex/wsp2/heptane/baseline-d2`) and persistence. The model *carries over* if D2 has
the lower CRPS than both baselines on each of the three days; it *does not* if it is higher than the
D-squared law on at least two; otherwise undecided. Three days cannot give an interval worth quoting.

**Arm F, new fuels, zero-shot.** n-octane, iso-octane and n-decane on FLEX-2; n-dodecane on CFI
(the 7 tests with a processed series that are free droplets inside the domain and off the test
days: 5 train, 2 validation). Releases: the candidates `flex_<fuel>.wsp7` W0 typed from property
ratios in [research/phys_droplet_fuels.md](research/phys_droplet_fuels.md) before any outcome of
these fuels was read (their ids are added here when the forward `droplet8.rs` is merged and they are
assembled, before any of them is run); against the root of each fuel, which is the heptane law
unchanged ("treat the fuel as heptane"), and persistence. For each fuel with at least three days:
the typed ratios *add something* if the candidate's CRPS is lower than the root's with a day
bootstrap interval that excludes zero; they *do not* if it is higher with an interval that excludes
zero; otherwise undecided. Iso-octane (2 tests) and CFI (7 tests) are reported and no conclusion is
drawn. The registered expectations of section 4 of that dossier are compared with what is measured.

**Cases and score.** As on FLEX: cutoffs 1, 3 and 5 s, horizon 10 s, quantile CRPS, MAE of the
median, coverage of the 50 % and 90 % bands; reported per fuel, pooled over train and validation
days and for each separately. Refusals are counted and are the model's answer.

**After the score.** Outcomes of train days become study data for one revision by 12.3; the
validation days judge that revision once; test days stay unread until 12.5.

**Known threats.** The fuel of a FLEX-2 test point comes from the folder NASA filed it under, not
from a file of the test (27 points are unresolved and are not in any batch). The ambient
composition is the gas reading of the same day taken last before ignition. The CFI series are read
under the FLEX time origin, which was judged plausible and not checked. The measurement differs from
NASA's processing by up to about 30 micrometres RMS, which is of the size of the model's own error on
FLEX, so differences between releases of less than that are not resolved. Disclosed by the agent
that wrote the catalogues: a few outcome cells of two CFI train-day tests and two FLEX-2 timeline
rows were printed while it learned file layouts; none reached the author.

### 12.8 The fuel table on Flame Design: ethylene as itself, registered 15 November

**What exists.** Four new strict forwards, built by an engineer to the specification of
[research/phys_spherical_fuels.md](research/phys_spherical_fuels.md) section 3 and committed as
`ea3c531`: `flame5.rs`, `flame6.rs`, `flame7.rs` (the three member laws reading the fuel from a
typed table) and `committee3.rs` (`acme_*.wsp12`). For ethane, propane and butane they return the
forecasts of the stored releases bit for bit on every CFI-G train prefix (310 tests; 120 release ids
unchanged).

**Candidates.** The fuel tables were typed from that dossier, sections 3.4 and 3.5
(`configs/models/acme/fuel_tables/`). Zero-shot for the new fuels with respect to per-test outcomes
of Flame Design and s-Flame; **not blind to the aggregate arms of 11.3**, which the dossier's author
knew, and Flame Design's aggregate scores have been read twice by the lead author.

| Release | Id | What it is |
|---|---|---|
| `acme_normal.wsp12` F1 | `sha256:131060f2b1389f70e27c171b2996b584e7f5cfb7e64aabb450b97069e957ffdf` | the adopted `acme_normal.wsp7` P3 with the fuel table in every member; on CFI-G train its per-case scores equal those of P3 exactly, on both data roots |
| `acme_inverse.wsp12` F1 | `sha256:d59251cfba37d46cf264dd706f1df58bb59be8e4ef8964335944dd6c372c3b42` | the adopted `acme_inverse.wsp5` P1 with the fuel table in every member; on CFI-G train its per-case scores equal those of P1 exactly, on both data roots |

Because F1 equals the adopted model on every fuel that model answers, serving F1 changes nothing on
CFI-G. What is tested is what it says where the adopted model refuses.

**The test.** The Flame Design set of 11.1 as amended, by the chain of 11.3 and 12.6 with its
endpoint thresholds unchanged; the four test days stay unread. Ethylene tests only, each run twice on
identical cases:

- *as itself*: F1 with the fuel as recorded (`fuel_species` ethylene);
- *as ethane*: the surrogate of 11.1 (for normal flames `acme_normal.wsp7` P3, for inverse flames
  `acme_inverse.wsp5` P1, with ethane's indicators), and the incidence baseline of 11.1.

Without and with the planned stop. F1 refuses an ethylene flame whose adiabatic temperature is below
1900 K (its weak-flame law is an alkane law); refusals are counted, and the comparison is made on the
cases F1 answers, with the surrogate's score on the refused cases reported beside it.

**Score and what each result will be taken to mean.** Integrated Brier, extinction and operator
termination, 6 to 120 s, case means; paired differences with a bootstrap over sessions. For each
family, on the path without the planned stop: the fuel table *adds something* if F1's extinction
score is lower than the surrogate's with an interval that excludes zero; it *loses something* if it
is higher with an interval that excludes zero; otherwise it is level. In every case F1 is the release
that answers for ethylene from now on, because it states a fuel instead of borrowing one; if it
*loses something*, its ethylene row is the first thing the revision of 12.3 looks at. The registered
expectations of section 4 of the dossier (ethylene normal flames as ethane within 10 % in air and
0.85 of ethane at 38 to 40 % oxygen; ethylene inverse flames at twice the adopted clock, range 1.2 to
3.7) are compared with what the train days show.

**After the score.** The train days of Flame Design (8 normal, 5 inverse) become study data for one
revision by 12.3 of the fuel table and, if the pattern holds across sessions, of the laws; its
validation days (3 normal, 2 inverse) judge that revision once together with CFI-G validation, by
the rule of 12.6 with the surrogate replaced by F1.

**s-Flame (PSI-23)** is not registered yet: its table gives no adiabatic temperature, 49 tests have
no oxygen fraction per test, and its burner limit is 250 C, which no era of the committee knows
([feedback/sflame_transfer_feasibility.md](feedback/sflame_transfer_feasibility.md)).

### 12.9 Fabric flames: will it ignite and stay lit (BASS-II and BASS), registered 15 November

**What exists.** A new strict forward `front5.rs` (`bass_front.wsp5`), built by an engineer on its
own branch to the specification of [research/phys_quench_w0.md](research/phys_quench_w0.md) section
6 and merged as `eeb2003`: the tip law of `front.rs` unchanged, its single quench hazard replaced by
an oxygen limit with a propagated width, ignition, and a mixture of 41 equal-probability nodes over
the boundary error that is the same node at every time of a burn. A carry of D1 on it reproduces
D1's tip forecast bit for bit on the train tracks.

**Releases, frozen now.**

| Release | Id | What it is |
|---|---|---|
| `bass_front.wsp` D1 | `sha256:ac15e49f1cb60d4b4679bcdff82a07c18a4a501454428e35ac86e935e168855c` | the adopted model; its quench boundary was typed as a line from one train burn on each side |
| `bass_front.wsp5` carry of D1 | `sha256:4da9967f32a8935890dbe07c700f7b89cfd589e177800fc2116ffcf5069b8ed4` | D1's tip law with the boundary law switched off: every sample ignites and stays lit |
| `bass_front.wsp5` Q0 | `sha256:00a8bc356d8e4e02a58b87d4786feeafd25b55b90a74b45e29c797054b9c5e72` | D1's tip law (data-informed) with the **zero-shot** boundary law of the dossier, section 6.4; one number changed before any score for a numerical reason (`ext.width` 0.0005, see its header) |

Beside them a constant: the share of BASS-II train burns that ignited and spread, from the newly
measured train tracks only.

**On the four train tracks** (in-sample for D1, whose line was typed from them; out-of-sample for
Q0's boundary law): tip CRPS 0.0761 for all three; ignition-outcome Brier D1 0.0033, carry 0.2500,
Q0 0.1469.

**Measurement.** Every burn video of BASS-II (PSI-25) and BASS (PSI-26) is being measured by an
agent that sees no forecast, with its method frozen on BASS-II train burns
(`runs/dev/media/fabric/FREEZE.json`); the author has read no track of a held burn.

**Groups.** BASS-II keeps the project split. PSI-26 was declared outside every partition and has no
groups: its burns are split by test day with the project's rule and seed, from the measurer's
manifest of burn dates and nothing else, before the author reads any PSI-26 track; the split file is
committed before the score.

**The test.** Scored once by `wsp-dev front-feedback` on: the BASS-II validation burns (already
scored for D1 on 14 November, so development numbers for D1 and blind for Q0's boundary law) and the
train and validation days of PSI-26 (blind for every release). The BASS-II test partition and
PSI-26's test days stay unread until 12.5. Burns outside what a release declares (another material,
no usable track) are counted as refusals.

**Scores.** Primary: the Brier score of the ignition outcome (did the sample ignite and spread).
Beside it: the track-ended Brier score, and the tip CRPS on the burns that spread, which is the same
for all three releases by construction on concurrent burns of the fabric.

**What each result will be taken to mean.** The zero-shot law *carries* if Q0's ignition Brier on
the pooled set is lower than the constant's and lower than D1's, and the paired difference to the
constant has a day-bootstrap interval that excludes zero. It *fails* if it is higher than the
constant's with an interval that excludes zero. Otherwise undecided. If it carries, Q0 replaces D1 as
the release to serve. Whatever the outcome it is reported with its counts.

**After the score.** BASS-II train burns and PSI-26 train days become study data for one revision by
12.3 of the boundary constants (each stays inside the uncertainty the dossier gives it); BASS-II
validation and PSI-26 validation days judge that revision once.

**Known threats.** The dossier's author was not blind to the published literature on BASS-II. The
duct speed is read from the fan setting or the video overlay, and the law's constants hide the
mapping from duct speed to the speed at the flame. PSI-26 burned several materials and geometries;
only the SIBAL fabric and the 1 mm PMMA sheet are declared. The ignition outcome of a burn is the
measurer's class ("spread", "went out", "never lit", "cannot tell"), and "cannot tell" is left out.

**Amendment of 12.7, made before any FLEX-2 series existed and before any candidate was run (15
November).** The forward `droplet8.rs` (`flex_<fuel>.wsp7`: the law of `droplet6.rs` for eight
fuels, with the pressure bound, the helium bound and the horizon widened for CFI and FLEX-2) and
`droplet9.rs` (`flex_<fuel>.wsp8`: mixtures, staged burning, warm flame, direct cool ignition, fibre
support, all switched off in its roots) were built by an engineer and merged as `298c3ee`; the
merged crate passes 338 tests; the methanol and heptane carries on `droplet8.rs` return the forecasts
of `droplet6.rs` on FLEX train exactly. The zero-shot candidates of arm F, assembled without a change
to any typed number:

| Fuel | `flex_<fuel>.wsp7` W0 | Used on |
|---|---|---|
| n-octane | `sha256:2c7a77162fa7f71975ae36c4e6928ed7c96c2b2241274000ca5425b86c968297` | FLEX-2 |
| iso-octane | `sha256:b2e57d80a8a76c37448e71456ae7231b9a13c8e9e108f92a2149084dce73f7e2` | FLEX-2 |
| n-decane | `sha256:62e406617af4a791bd4706088f788d386a6c44305a81ce8f30bfffd6ed51e5bf` | FLEX-2 |
| n-dodecane | `sha256:70042961d2a45df07a93a0cc3ed828dd8b022de5dbebbe9b97d8b18f7f8d3e74` | CFI |
| farnesane | `sha256:66b398e44d9c673352a39f41c59da5c5ccb2d97d41248991100e67b2c0fea0d3` | no test (every CFI farnesane test is fibre-supported or of unknown support) |
| ethanol | `sha256:ae03363f6a973c2bc581b20e96a6936bdc83cfbcfc02742bcb24b3a60dbad139` | no test (flown only in droplet arrays) |

The root of each fuel ("treat the fuel as heptane", for ethanol as methanol) is its
`baseline-d2` under `configs/models/flex/wsp7/<fuel>/`. For arm H a third comparator is added: the
heptane law of the mechanism-based forward, `flex_heptane.wsp7` carry of P1
`sha256:d87166c21c371f256bf46b6ce050498bea90b05b2a3e7670d9a870170564616f`, because the new-fuel
candidates descend from it and not from the adopted `flex_heptane.wsp2` D2; it is reported and does
not enter the rule of arm H.

**Outcome of 12.8 (15 November).** Run once after commit `787f5ce`
([feedback/flame_design_fuel_table_result.md](feedback/flame_design_fuel_table_result.md)).
Extinction on the path without the planned stop, ethylene as itself against ethylene as ethane on
identical cases: **inverse flames** 0.1077 against 0.1466, difference -0.0388 [-0.0658, -0.0101] over
7 sessions: the fuel table *adds something*. **Normal flames** 0.1853 against 0.1128, difference
+0.0725 [+0.0402, +0.1022] over 9 sessions: it *loses something*; F1 refuses 18 cases of 6 tests
below 1900 K. With the planned stop the two verdicts are the same. The registered expectation for
lean inverse flames (twice the adopted clock) is borne out on the train days (1.7); the one for
normal flames at 38 to 40 % oxygen (0.85 of ethane) is refuted in direction.

### 12.10 The burner flames revised once from the train days: F2, registered 15 November

**The step (12.3).** Forward and decompose: [feedback/flame_design_train_days_anatomy.md](feedback/flame_design_train_days_anatomy.md).
Attribute: seven diagnostic variants of the ethylene row and two of the planned stop, scored on train
days only ([feedback/flame_design_fuel_table_attribution.md](feedback/flame_design_fuel_table_attribution.md),
[feedback/acme_planned_stop_v2_attribution.md](feedback/acme_planned_stop_v2_attribution.md)). Reason
and type: [research/phys_ethylene_held.md](research/phys_ethylene_held.md) (claude-fable-5-2, high
effort; six stated questions to the model, no sweep) and
[feedback/acme_planned_stop_v2.md](feedback/acme_planned_stop_v2.md). No validation-day or test-day
test of Flame Design and no CFI-G validation outcome was read for any of it.

**What changed, and from what.**

| Change | From | Typed |
|---|---|---|
| Ethylene, self-extinction coordinates, every member | F1's half-way to butane was the wrong direction in 8 of 8 train sessions; the train days ask for a position beyond ethane | (1.0, 0.0): the half-step, "no butane term" |
| Ethylene, flow shift of the hinge members | the hinge law reads a fuel mass flow, and ethylene burns as ethane does at about 0.58 of its flow (the flow at which flames at 36 to 40 % oxygen go out inside a test: ethylene between 1.95 and 2.52 mg/s on the train days, ethane between 1.0 and 1.5 mg/s on CFI-G train); the train days ask for -0.54, theory gives -0.21 of it | -0.31: the half-step from -0.085 |
| Ethylene, critical-flux shift of the growth members | F1's -0.085 made the growth clock 1.2 times early; the growth law already carries the flame radius | -0.02: the half-step |
| The planned stop (an input, both families) | the stage sum left out the igniter delay before retract and the fibre step; computed from the script file alone the planned stop is within 1 s of the recorded one on 77 to 100 % of train tests that ran to the scripted end | `tools/wsp/ext/acme_script2.py`; the stage sum where the file is incomplete; read at -1, 0, +1 s with weights 0.25, 0.5, 0.25 |
| The days on which the burner limit changed (normal flames) | the script writes the limit at the start of each test: 500 C through 23 August 2021, 450 C from 13 September | days 2021235 and 2021256 (an input; no train test lies between the old and the new reading) |

Not changed: every law statement, the committee weights, the methane and alkane rows, the 1900 K
bound, the inverse fuel table. Known and left for a new forward: a nitrogen-diluted ethylene flame
lives longer than the growth law says; ethylene below 1900 K is refused.

**Candidates.**

| Release | Id |
|---|---|
| `acme_normal.wsp12` F2 | `sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d` |
| `acme_inverse.wsp12` F2 | `sha256:dff7da0da8e77785acecc0ffb5acaf222358d4274b36e2c52f39426aaebd594e` |

**Train (the days the row was typed from: not a test).** Flame Design normal train days, 204 cases,
extinction / termination: F1 0.2061 / 0.2905, ethylene as ethane 0.1272 / 0.2763, the F2 row
0.0990 / 0.2728 without the planned stop. On CFI-G train the F2 row changes nothing.

**The look.** One, in two parts read together, as in 12.6:

- (a) CFI-G validation through the sealed chain. Without the planned stop: F2 against F1 on the
  shared records (for normal flames they differ only where a validation test changes era). With the
  planned stop: F2 on the new input (`runs/dev/data_ext2`) and F1 on the old one (`runs/dev/data_ext`),
  each as an aggregate over the same validation runs; the evaluator cannot pair cases whose inputs
  differ, so no paired interval is given there.
- (b) The validation days of Flame Design (normal 3 days, inverse 2 days), by the chain of 12.8 with
  its endpoint thresholds unchanged: F2 with ethylene as itself and the new planned stop (the stage
  sum where the script file is incomplete), against F1 as scored in 12.8, and ethylene as ethane
  beside them. The test days stay unread.

**Rule, written before either part is run.** As in 12.6, per family and per path: F2 replaces F1 if
(i) the mean over the two investigations of (extinction + operator termination) is not higher than
F1's and (ii) the mean over the two investigations of extinction alone is not higher than F1's. If
F2 passes without the planned stop and fails with it, or the reverse, it is adopted for the path it
passes. What the score of "ethylene as ethane" would have been is reported beside it and does not
enter the rule.

**Known threats.** Three validation days of normal flames and two of inverse flames are few. The
hinge shift is read from the train days more than derived (theory gives -0.21 of -0.54); it could be
a difference between the 2019 and 2021 campaigns, which no alkane flown on a Flame Design train day
can separate. The endpoint method puts the flow change inside the ramp on tests with an
ignition-to-test ramp; its thresholds are registered and are not changed here.

**Outcome of 12.10 (15 November).** Both parts were run once after commit `7a37ab5`; tables in
[RESULTS_GEN6.md](RESULTS_GEN6.md). Normal flames, decision score without / with the planned stop:
F1 0.3542 / 0.1579, F2 0.3374 / 0.1532; extinction mean F1 0.1048 / 0.0792, F2 0.0927 / 0.0771.
Inverse flames: decision score F1 0.2499 / 0.1493, F2 0.2499 / 0.1384; extinction mean 0.1080 /
0.1134 for both. F2 passes (i) and (ii) in both families on both paths and replaces F1. On CFI-G
validation with the planned stop F2 is worse than F1 for normal flames (0.2271 against 0.2191) and
level for inverse flames; the rule passes on the Flame Design half.

**Second amendment of 12.7 (15 November, before any FLEX-2 score existed).** The note of the day
split said that frames of test days are not downloaded before the final run. That is changed to:
they are not *scored* before the final run. The eleven single free droplets of a covered pure fuel
inside the domain on the three test days that have such tests (n-octane 6, n-decane 5; 14.9 GB;
[atlas/splits/PSI-68-batch-test.json](atlas/splits/PSI-68-batch-test.json), fixed by conditions and
file sizes only) are downloaded and measured by the same blind measurer with the same frozen tool,
into a folder of their own. Their series are read by nobody and enter a score only in the one final
run of 12.5, as the held tracks of the fabric burns do. Reason: without them the new-fuel droplet
candidates would have no final test at all except one CFI test.

**Outcome of 12.7 so far (15 November).**

*CFI, arm F* ([feedback/cfi_transfer_result.md](feedback/cfi_transfer_result.md)): 7 n-dodecane
tests on 5 days, 21 cases, no refusal. CRPS in micrometres, pooled: candidate 239.9, root (the
heptane law) 259.1, persistence 431.5, classical d-squared law 707.6; candidate minus root -19.3
[-85.6, +58.5]. Reported; as registered, no conclusion from 7 tests. On the five train-day tests the
main expectation of the fuels dossier failed by its own stated test: 4.0 to 4.4 mm droplets in air
lost the hot flame and a cool flame followed every time, where it expected neither below 4.7 mm.

*FLEX-2, first scoring: void as a test of the models*
([feedback/flex2_transfer_result.md](feedback/flex2_transfer_result.md)). The measurer's report,
written before the scoring, says the pixel scale of FLEX-2 was assumed (a field of view of 50 mm) and
not measured, and that its own rod check suggests every diameter may be about 30 % too large. The
scoring ran anyway, which was a mistake of sequencing by the lead author. Only 15 of 48 series passed
the measurer's acceptance rule (n-heptane 8, n-decane 6, n-octane 1, iso-octane 0). The numbers it
produced (arm H: D2 460.5 against 177.6 for the d-squared law; arm F n-decane: candidate 202.1
against root 66.8) carry the pattern of a scale error: every forecast median runs high, the gap
grows with lead time, and burning rates read 0.7 to 1.4 mm2/s where FLEX gives about 0.6, which
penalises releases that type a rate against laws that read it from the prefix. **No verdict of 12.7
is drawn from them.** The lead author has seen these aggregates.

**Third amendment of 12.7 (15 November).** The pixel scale is pinned by the measurer from evidence
that involves no model and no forecast (the camera's own settings against the FLEX tests where the
scale was verified; FLEX-2's calibration material; an object of known size identified as the same
part in both investigations), never from droplet sizes, burning rates or agreement with an expected
value. The series are rescaled, the acceptance rule is applied unchanged, and arms H and F are scored
once more by the same chain. If the scale cannot be pinned to about 5 %, the FLEX-2 series are
declared usable for scale-free quantities only and no CRPS verdict is drawn. The same scale rule
applies to the held test-day series.

**Outcome of 12.9 (15 November).** Scored once by `wsp-dev front-feedback` on the BASS-II validation
burns (the tracks measured blind on 14 November: 17 tracks, 12 scoreable, 40 cases).

| | D1 (adopted) | carry of D1 ("every sample lights and stays lit") | Q0 (zero-shot boundary law) |
|---|---|---|---|
| tip CRPS, sample widths (the same law in all three) | 0.1168 | 0.1168 | 0.1168 |
| track-ended Brier, 12 burns | 0.1780 | 0.1902 | **0.1356** |
| ignition-outcome Brier | 0.1703 (16 tracks) | **0.1333** (15 tracks, 2 observed no spread) | 0.1928 (15 tracks) |

On the primary score, whether a sample ignites and spreads, Q0 is higher than the constant and
higher than D1: the zero-shot law does **not carry**, and with 15 tracks on a handful of days the
difference to the constant has no interval worth quoting, so by the registered wording it is
undecided and it is not adopted. On when a burning flame has ended it is the best of the three. D1
stays the release to serve.

**The second half of the registered set does not exist.** The measurer went through all 114 videos
of BASS (PSI-26): none shows a fabric sample. They are PMMA spheres, slabs and rods, candles and wax,
and most are setup or dilution recordings (the 2013 as-run sheet has 37 test rows). The day split of
PSI-26 stays frozen and unused. The assumption of the lead author that BASS holds the fabric burns
the quench boundary lacks was wrong. Of the 142 BASS-II burn files, the fabric ones are 2 cm and
1 cm SIBAL sheets; the method frozen on the 2 cm train burns rates only 12 of them usable (5 train,
4 validation, 3 test), and the 1 cm strips are not measurable by it.

**What this leaves.** The question "will it ignite and stay lit" remains short of burns: no model of
this lane beats a constant on it. The boundary law's ignition block is too pessimistic at the oxygen
levels flown; one revision by 12.3 from the train burns is possible and was not made, because the
train burns that went out or never lit number one.

### 12.11 The final run of this generation, registered 15 November

The run of 12.5. It is made once, by a scoring agent, with `tools/wsp/final/run_final.py` (prepared
and rehearsed on the validation partition and validation days only: every known number reproduced;
commits `ad0e698`, `5349d98`, `31a2906`), on the release list
`tools/wsp/final/releases_final_20261115.json`, whose sha256 the run prints and the result records.
Each step writes a marker before it reads anything and cannot run twice. The author receives
aggregate tables only. **Nothing is revised from these outcomes in this generation**; a later
candidate that uses them treats them as development data and says so.

**What is scored.**

| Step | Data nobody has read | Releases |
|---|---|---|
| 1 | the sealed test partitions of the project: FLEX methanol and n-heptane; CFI-G normal and inverse flames, on the shared records and with the planned stop of `runs/dev/data_ext2` | `flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2; `acme_normal.wsp12` F2, `acme_inverse.wsp12` F2; beside each: the reference baseline of its spec, the zero-shot W0 (droplets), the model of 14 November it descends from (`acme_*.wsp5` P1, shared records), the fitted baselines of the data lane |
| 2 | the four test days of Flame Design (normal 19123, 19200, 19206; inverse 21361), endpoints by `flame_design.py` with its thresholds unchanged | F2 with ethylene as itself and the new planned stop; F1; ethylene as ethane; the incidence baseline |
| 3 | the CFI test-day droplet that the rule of 12.7 admits (one test) | `flex_dodecane.wsp7` W0, its root, persistence, the classical d-squared law |
| 4 | the BASS-II test partition: the two burns whose blind tracks the measurer rated usable, in the scorer's format | `bass_front.wsp` D1, the carry, `bass_front.wsp5` Q0 |
| 5 | the eleven FLEX-2 test-day droplets, only if the pixel scale is pinned (third amendment of 12.7) | the zero-shot n-octane and n-decane candidates, their roots, persistence, the classical d-squared law |

Not scored, and left unread: the test days of FLEX-2 if the scale is not pinned; PSI-26 (no fabric
burn); s-Flame (not registered).

**What will be reported**, whatever it is: per release the cases, tests, sessions or days and
refusals; the metric each family is judged on with the paired differences to its comparators and the
cluster bootstrap the evaluator gives; the size of every set, so that a number resting on two burns
or one droplet is read as such. The verdict words of this protocol are not applied to the final run:
it adopts nothing and refuses nothing. It says how the releases that were adopted on development
data do on data none of them was written or chosen from.

**Why now.** The owner asked for the models to be completed without supervision and approved the
plan whose last step before the benchmark is this run, made by this lane because the separate
evaluation lane is gone (12.1). What stays open after tonight (the quench boundary, diluted ethylene
flames, the FLEX-2 scale, s-Flame, the other families) is work for a next generation, which will have
the unread batches of NASA's archive as its tests.

**Outcome of 12.7 on FLEX-2, the one rescore on the pinned scale (15 November).**
[feedback/flex2_transfer_result_v2.md](feedback/flex2_transfer_result_v2.md). The scale: FLEX-2 has
the FLEX scale, 33.85 px/mm with 1.5 % uncertainty, from the FLEX-2 requirements document and the
needle arms in first frames (`runs/dev/media/droplet/SCALE_FLEX2.md`); no droplet size, rate or
forecast was used. 15 of the 48 series pass the measurer's unchanged rule; no release refused a case.

*Arm H, the adopted n-heptane model unchanged on another investigation* (8 tests, 23 cases, 2 days,
all train days; CRPS in micrometres):

| | `flex_heptane.wsp2` D2 | d-squared law of its spec | `flex_heptane.wsp` W0 | persistence | classical d-squared law |
|---|---|---|---|---|---|
| 8 March 2013 | 25.5 | 35.5 | | 490.1 | |
| 14 March 2013 | 15.9 | 29.5 | | 395.4 | |
| pooled | **20.9** | 32.6 | 35.6 | 444.8 | 124.7 |

D2 minus the d-squared law: -11.7 [-13.6, -10.0]. D2 is below both baselines on both days. The rule
named three days and the third n-heptane day gave no usable series, so by its letter the test is
undecided for want of a day; on the two days that exist the model **carries over**, and nothing in
them says otherwise. Stated sensitivity, run once each: with every diameter 1.5 % smaller the
difference is -8.4, 1.5 % larger -12.4; with the time origin 0.4 s earlier -7.6, later -16.0; D2
stays below both baselines on both days in each. The measurer's worst case for the scale (7 %
smaller, only if FLEX-2 used thicker needle arms, for which nothing speaks) was not run and could
close the margin. D2's bands are too wide on FLEX-2: 88 % of targets fall in its 50 % band.

*Arm F, zero-shot new fuels.* n-decane (6 tests on 4 days): candidate 45.8 against its root 38.8,
difference +7.0 [-11.5, +34.8]: **undecided**. The measured rate lies between the two: the typed
decane rate is too slow, the heptane rate too fast. n-octane: one test, a droplet that hardly
shrinks; reported, no conclusion. Iso-octane: no usable series. The registered expectations hold for
heptane at 2 mm (0.61 and 0.57 mm2/s against 0.62) and for decane at 17 % oxygen (0.35 to 0.38
against about 0.37), and are 10 to 20 % high for heptane droplets of 3 to 3.6 mm.

No droplet candidate is adopted and none is revised tonight: the train days of FLEX-2 and CFI are
study material for the next step.

## 13. Seventh generation (15 November, day): a model for every investigation

The owner's goal is a hand-written model with a scored forecast for each of NASA's 19 flight
combustion investigations. After the night of section 12 nine had one. From the morning of 15
November every remaining investigation has a lane: a data and scoring engineer who builds its case
files from the text tier (train days readable; validation outcomes written apart and unread; test
days unopened; briefs in `runs/dev/gen7/`), a physicist who types a zero-shot law before any outcome
of it is seen (`research/phys_<slug>_w0.md`), and one generic strict forward `law.rs` that hosts
those laws. The rules of section 12 hold: the law and its test are committed before the score, the
blind score is taken once, and only then are the train days studied.

### 13.1 s-Flame (PSI-23): the adopted normal-flame model, unchanged, registered 15 November

Inputs and case files: [feedback/sflame_inputs.md](feedback/sflame_inputs.md),
`tools/wsp/ext/sflame_cases.py`. The adiabatic flame temperature that NASA's s-Flame table does not
give is computed as the tables of CFI-G and Flame Design compute theirs (equilibrium of the
stoichiometric mixture of the two streams from 298.15 K; reproduced within 4.1 K on 154 CFI-G train
rows and within 3.5 K on 140 Flame Design train-day rows). Endpoints by `flame_design.py`, unchanged.

**Release.** `acme_normal.wsp12` F2 `sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d`,
as adopted in 12.10. Its fuel table answers for ethylene and methane in nitrogen (the methane row is
zero-shot and has never met data) and refuses hydrogen, hydrogen blends and helium dilution.

**The test.** The validation days of s-Flame, constant-flow tests with a known oxygen fraction that
F2 answers (16 tests in nitrogen by the inputs; the 17 helium-diluted ones and the hydrogen tests are
refusals and are counted). Cutoffs 1, 3, 5 s; integrated Brier, extinction and operator termination,
6 to 120 s; without and with the planned stop. Beside F2: the incidence baseline of CFI-G train
(frozen, nothing from s-Flame) and the incidence of the s-Flame train days. The test days stay unread.

**What each result will be taken to mean.** F2 *carries over to s-Flame* if its sum is lower than
that of the CFI-G incidence baseline on the cases it answers, on both paths; it *does not* if it is
higher on both; otherwise undecided. With two or three validation days no interval is quoted as
stable. No number of F2 is changed from this outcome before the s-Flame train days have been studied
by 12.3.

**Already seen.** F2 was run on the s-Flame train days while the inputs were built (53 cases of 22
tests on 3 days: 0.059 / 0.032 against 0.235 / 0.038 for the CFI-G incidence). Those days are study
data, not a test.

**Known threats.** Every s-Flame script sets the burner temperature limit to 250 C; F2 assumes 510 C
for these dates, and by its own heating law half of the train tests it answers reach 250 C before
their planned stop. The oxygen fraction is the nominal 21 or 30 %, and the adiabatic temperature
moves 40 to 47 K per 0.01 of oxygen. The endpoint labels are unreliable on dim flames.

**Outcome (15 November, run once; [feedback/sflame_transfer_result.md](feedback/sflame_transfer_result.md)).**
Undecided. Of the 34 constant-flow tests on the validation days, 9 (methane in helium) have no oxygen
fraction per test and are outside the set, 8 (ethylene in helium) are refusals by this registration,
and 17 in nitrogen were run; the release refused one methane test (adiabatic temperature below its
1900 K bound). Answered: 46 cases of 16 tests on 2 days (ethylene 3 tests on one day, methane 13 on
the other).

| 46 cases, extinction + termination | without planned stop | with planned stop |
|---|---|---|
| F2, case mean | 0.2619 | 0.3698 |
| CFI-G train incidence, case mean | 0.2793 | 0.2793 |
| s-Flame train incidence, case mean | 0.3218 | 0.3218 |
| F2, mean of the two day means | 0.1783 | 0.2454 |
| CFI-G train incidence, mean of the two day means | 0.2579 | 0.2579 |

The registration did not name the mean. The case mean is the one sections 11.1 and 12.10 used, and by
it F2 is lower on one path and higher on the other; by the mean of day means it is lower on both. With
two days that disagree, the stricter reading stands: undecided.

What the parts say. Ethylene (9 cases): 0.041 against 0.223 on both paths. Methane, the row that had
never met data (37 cases): 0.316 against 0.293 without the stop, 0.450 with it. The split that the
registration named as a threat explains the methane loss: on the 7 tests (all methane, 19 cases)
that F2's own heating law brings to 250 C before their planned stop, F2 scores 0.539 / 0.798 against
0.307 for the baseline, and 18 of those 19 cases ended as extinctions while F2 forecast burning to the
stop; on the other 9 tests (27 cases) F2 scores 0.068 on both paths against 0.260. The train days say
the same: 64 % of F2's train loss without the stop comes from the one ethylene test that the 250 C
limit stopped.

What follows. The burner temperature limit is a setting of each script, already read by
`tools/wsp/ext/acme_script2.py`; F2 takes it from the date. The next revision of the burner-flame
committee takes the limit as an input of the test. It is typed from the s-Flame and CFI-G train days
and judged once on these validation days, whose aggregates above have been seen (stated in its
registration). The test days 20142, 20170 and 20176 stay unread.

### 13.2 The nine investigations without a model: one blind look each, registered 15 November

The laws are the dossiers `research/phys_<slug>_w0.md` as committed in `3b33929` and `99b3d57`, typed by
claude-fable-5-2 before any score existed. An engineer transcribed their numbers into releases on the
generic forward `law.rs` without choosing any (element-by-element check, and the forward reproduces each
dossier's own table of expectations within the rounding of that table:
`runs/dev/gen7/transcription_checks.md`). No release below has been scored on any data, train included.
The full registration of each look (target, how the law gives it, baselines, cases and days, refusals
expected, threats, what the author had seen) is [feedback/gen7_registrations.md](feedback/gen7_registrations.md),
committed with this section; what follows is the list and the rule.

| Investigation | Slug and candidate | Release | What the author had seen | Scored on |
|---|---|---|---|---|
| BRE (PSI-20) | `bre` w0 | `sha256:aa0eef5b…2b4d` | zero-shot; published aggregates of the experiment read for comparison | validation days (35 tests, 4 days): when the flame ends and why |
| SPICE (PSI-107) | `spice` w0 (variant B) | `sha256:93e75024…f2a8` | zero-shot for the smoke-point length | validation (7 smoke points, 2 days) |
| SPICE (PSI-107) | `spice` lit | `sha256:40c345b0…d3bf` | NOT blind: its centre is the investigators' published fit of the same smoke points | the same; reported beside, decides nothing |
| E-FIELD (PSI-22) | `efield` w0 | `sha256:73f91b55…86a9` | typed with train aggregates seen (count of flame-outs, current range) | validation (36 tests, 4 days): ion current per level; flame out under the field |
| SAME-R (PSI-101) | `samer` w0 | `sha256:05f77ae2…7fb0` | zero-shot; train facts of the note read after the constants were typed | validation (24 rows, 3 days): particle diameter; detector at full scale |
| SAME (PSI-102) | `same` w0 | `sha256:e1362b55…2062` | zero-shot | the whole investigation, held (45 rows, one group) |
| DAFT (PSI-47) | `daft` w0 | `sha256:061f93de…5e5d` | zero-shot | the whole investigation, held (53 readings, 2 groups) |
| BASS and BASS-II plastics (PSI-26, PSI-25) | `bass1` w0 | `sha256:97c0daf9…39a2` | blind to the project's files; four published papers on these burns read | validation (41 burns, 12 days): the flow speed below which the flame goes out |
| SLICE (PSI-106) | `slice` w0 | `sha256:d10c94bc…006e` | zero-shot; one paper on the experiment read as a check | validation (75 lines, 2 days): blow-off and lifting per line |
| CLD Flame (PSI-21) | `cld` w0 | `sha256:0995df66…fe4e` | typed with train aggregates seen (share of flames lost, radiometer range) | validation (72 test points, 6 days): flame lost before fuel-off; radiometer level |

**Decisions of the coordinator on the open points of the draft, taken before any score.**
- BRE: the lane's clock (from the measured onset, about 0.8 s after ignition) is used unchanged; the law's
  one cause is extinction, and the mass left at the planned stop is operator termination, with the stop
  the lane's planned-stop baseline uses (two train medians of the data lane). Named as threats.
- SLICE: no validation line carries the cabin oxygen. The value supplied is the median of the cabin
  oxygen recorded on the train lines (conditions only), fixed before the look and stated in the result.
  The 41 lines of the ethylene day are refusals (a fuel outside the law).
- CLD: the planned fuel-off is declared as the scheduled stop, as the typed block needs. Schedules
  without a constant-flow stage are refusals.
- Plastics: only the quench speed is scored now. Whether the sample kept burning after the igniter is
  scored after the author states, without seeing data, how the law answers it; rods in concurrent flow
  and samples with no stated configuration are refusals.
- SPICE: the flame-length target is scored as typed in both candidates.

**What each result will be taken to mean**, per scored target, on the cases the release answers:
it *beats the simple baseline* if its score is lower than that of the constant or incidence built from
the train days (for SAME and DAFT, from SAME-R train); it *beats the best baseline* if it is also lower
than every other baseline the scorer builds for that target; it *loses* if it is higher than the simple
baseline. Refusals are counted and reported with their reasons. With two to six days per look no
interval is quoted as stable, and nothing is called validated. Whatever the outcome, the release is the
first hand-written model of its investigation and is reported as experimental, with its score beside the
baseline; no number is changed from this look before the train days have been studied (12.3). The test
days of every investigation stay unread.

**Amendment to 13.2, plastics, before any score of this target (15 November).** The author stated, from
the dossier alone, how the registered release answers "kept burning after the igniter" (section 9 of
`research/phys_bass1_w0.md`; no typed number changed): the smaller of `holds_low_flow` and
`holds_high_flow` for rod tips, spheres and sections, the smaller of the `flat_` pair for films and
slabs, at the initial oxygen, the duct speed at ignition, nose radius half the diameter (0.01 for flat
samples) and `heat_time` = the planned igniter on-time plus 60 s where a planned on-time is among the
inputs known before ignition, otherwise 200 s. Refused: duct speed at ignition below 0.3 cm/s, oxygen
outside 12 to 25 %, radius outside 2 to 30 mm, candles and wax, wake-lit spheres, rods lit at the
downstream end, Nomex, Ultem, fabric, and flat PMMA thicker than 5 mm. The law forecasts whether an
established flame is still there 60 s after the igniter is off; it does not forecast ignition, and it
does not use earlier burns of the same sample. The score is the Brier score against the lane's
`sustain`, beside the train incidence (simple baseline) and "always lit", read by the rule above and
reported as "given that a flame was established".

**Outcome of 13.2 (15 November; each look taken once, no rescoring; result files
`feedback/<slug>_w0_result.md`, train anatomies `feedback/<slug>_train_anatomy.md`).** Scores on the
cases the release answers; lower is better.

| Investigation | Target | Answered / refused | Release | Simple baseline | Best baseline | Reading |
|---|---|---|---|---|---|---|
| BRE | extinction, integrated Brier | 87 / 3 | 0.2755 | 0.2274 | 0.2129 (planned stop) | loses |
| BRE | operator termination | 87 / 3 | 0.1383 | 0.1799 | 0.0924 (planned stop) | beats the simple baseline |
| SPICE, clean variant | smoke-point length, mm | 7 / 0 | 5.862 | 6.099 | 5.042 (per fuel) | beats the simple baseline |
| SPICE, clean variant | length given the flow, mm | 7 / 0 | 2.533 | 6.099 | 1.261 (Lin and Faeth law) | beats the simple baseline |
| SPICE, published-fit candidate (decides nothing) | smoke-point length, mm | 7 / 0 | 2.914 | 6.099 | 5.042 | beats the best baseline |
| E-FIELD | ion current CRPS, µA | 1289 / 0 | 0.546 | 0.620 | 0.374 (space-charge law) | beats the simple baseline |
| E-FIELD | flame out under the field | 36 / 0 | 0.0753 | 0.0884 | 0.0884 | beats the best baseline |
| SAME-R | particle diameter (ln) | 23 / 1 | 0.167 | 0.393 | 0.339 (per material) | beats the best baseline |
| SAME-R | detector at full scale, Brier | 23 / 1 | 0.228 | 0.411 | 0.247 | beats the best baseline |
| SAME (held whole) | particle diameter (ln) | 45 / 0 | 0.184 | 0.220 | 0.192 | beats the best baseline |
| SAME (held whole) | detector at full scale | 45 / 0 | 0.059 | 0.205 | 0.055 | beats the simple baseline |
| DAFT (held whole) | number concentration (ln) | 53 / 0 | 0.777 | 2.678 | the only baseline | beats it |
| Plastics | quench speed, interval CRPS cm/s | 28 / 13 | 0.517 | 0.230 | 0.180 (published correlation) | loses |
| Plastics | kept burning after the igniter | 22 / 19 | 0.086 | 0.120 | 0.120 | beats the best baseline |
| SLICE | blow-off per line | 34 / 41 | 0.155 | 0.161 | 0.149 (by tube) | beats the simple baseline |
| SLICE | lifted per line | 34 / 41 | 0.089 | 0.056 | 0.056 | loses |
| CLD Flame | flame lost, integrated Brier | 61 / 11 | 0.593 | 0.478 | 0.328 (by dilution ramp) | loses |
| CLD Flame | radiometer, CRPS log10 V | 61 / 11 | 0.108 | 0.263 | 0.174 | beats the best baseline |

Of 18 scored targets of the registered releases, the zero-shot law beats every baseline on 8, beats
the simple baseline but not the best one on 6, and loses to the simple baseline on 4 (BRE extinction,
plastics quench speed, SLICE lifting, CLD flame loss). SLICE was judged on its one methane day with
the train-median cabin oxygen 0.2191; the 41 lines of the ethylene day are refusals. The plastics
"kept burning" answer is given that a flame was established, with the 200 s default on every case; the
two plastics targets treat opposed rods differently (covered for the quench speed, refused for "kept
burning"), which is recorded as an inconsistency of the mapping, not corrected after the look. The
CLD validation set is harder than train for every forecaster (the incidence baseline moves from 0.15 to
0.48).

What the train days say of the four that lose (anatomies): BRE forecasts extinction too early at
reduced pressure and at 30 to 36 % oxygen; the plastics law puts the quench speed of thick opposed
slabs about 5.7 cm/s too low; SLICE's loss is on the 0.4 mm tube; CLD under-forecasts the loss of
diluting flames (86 % lost against 26 % forecast).

### 13.3 One revision per investigation from its train days, registered 15 November before any score

The authors of BRE, CLD Flame, SLICE, the plastics law and SAME-R each type one damped step from the
train anatomy of their law (`research/phys_<slug>_d1.md`: numbers changed, old and new value, reason).
The engineer assembles each as a revision of the registered zero-shot release (`d1-fable`) without
choosing a number. Each revision is scored once on the same validation cases as 13.2, beside its
zero-shot parent and the same baselines. The authors and the coordinator have seen the aggregates of
13.2 above (not the validation cases); that exposure is the reason the test days stay unread and the
final word belongs to them. **Reading:** a revision *replaces* its parent if its score is lower than
the parent's on the target that lost (or, for SAME-R, on the particle diameter) and not higher than
the parent's by more than 5 % on the other scored target of the investigation; otherwise the parent
stays. SAME and DAFT have no data left to judge a revision and keep their zero-shot releases.

**Amendment to 13.3, plastics, before any score of the revision (15 November).** The forward has no
thickness feature, so the author's step uses the idle `nose_radius` of flat samples as a flag: flat
PMMA thicker than 0.5 mm is supplied 0.02, films stay at 0.01 (`research/phys_bass1_d1.md`). This is
an input convention computed from a condition known before the burn; the declaration is unchanged, and
the zero-shot parent must give identical forecasts under either value (checked on forecasts before the
revision is scored). If the check fails, the revision is not scored and the author's one-number
fallback of the same file is used instead.

**Outcome of 13.3 (15 November; each revision assembled verbatim from its committed dossier, run on
train, then given its one look; `feedback/<slug>_d1_result.md`).**

| Investigation | Target | W0 train | D1 train | W0 validation | D1 validation | Best baseline (validation) | Reading |
|---|---|---|---|---|---|---|---|
| CLD Flame | flame lost (the target that lost) | 0.1569 | 0.1253 | 0.5925 | 0.5139 | 0.3282 | **D1 replaces W0** |
| CLD Flame | radiometer | 0.1500 | 0.1234 | 0.1083 | 0.1003 | 0.1743 | |
| Plastics | quench speed (lost) | 1.3296 | 0.8987 | 0.5169 | 0.5067 | 0.1797 | **D1 replaces W0** |
| Plastics | kept burning | 0.0147 | 0.0147 | 0.0862 | 0.0862 | 0.1203 | |
| SLICE | lifted (lost) | 0.1559 | 0.1490 | 0.0886 | 0.0555 | 0.0563 | W0 stays |
| SLICE | blow-off | 0.1023 | 0.0601 | 0.1548 | 0.1631 | 0.1493 | |
| BRE | extinction (lost) | 0.2511 | 0.2174 | 0.2755 | 0.2866 | 0.2129 | W0 stays |
| BRE | operator termination | 0.0494 | 0.0430 | 0.1383 | 0.1403 | 0.0924 | |
| SAME-R | particle diameter (judged) | 0.2722 | 0.2158 | 0.1672 | 0.1725 | 0.3392 | W0 stays |
| SAME-R | detector at full scale | 0.1501 | 0.1005 | 0.2279 | 0.2436 | 0.2473 | |

Every step improved its train score; two of five also improved the validation target and replace their
parents. SLICE's step repaired lifting (0.0886 to 0.0555, level with the incidence) but raised blow-off
by 5.4 %, just past the 5 % allowance registered beforehand, so its parent stays. BRE's and SAME-R's
steps were worse on validation on both targets: their train gains did not carry to other days. The two
revisions that replace their parents still lose to the simple baseline on the target that lost (CLD
0.514 against 0.478; plastics 0.507 against 0.230): what they need is structural (a feature saying that
a test dilutes until the flame is lost; a thickness input and the rods), named by their authors. The
plastics slab flag left every forecast of the zero-shot parent identical (0 of 66 flat rows differ).
The validation days of these five investigations are now spent for selection; the test days of all of
them are unread.

### 13.4 What went wrong, and second laws: validation opened as study data, registered 15 November

The owner asked for the failures to be analysed and the models improved. The validation days of the
nine investigations of 13.2 have been used twice for selection (13.2, 13.3) and can judge nothing
more. From this commit they are study data beside the train days: their cases may be read, and every
number scored on them from now on is an in-sample development number and is labelled so. The only
judgement left for these models is one final run on their test days, which stay unread until a release
list is frozen for it (12.5). SAME and DAFT were scored whole and have no test days; their zero-shot
scores of 13.2 stand as their only held result, and any later revision of them is unjudged.

Method. (1) A failure analysis per target on train and validation cases together
(`feedback/gen7_failure_analysis.md`): where the law loses to the best baseline, by the strata of the
conditions and of the procedure (what the script or the crew was set to do: a dilution ramp, a planned
stop, a flow stepped until blow-off, a flow lowered until quench), which input known before or during
the test separates the cases, and how the validation conditions differ from train. (2) Each author
types a second law that takes the missing inputs; a declaration is frozen with its release, so a
second law is a new slug (`<slug>2`). (3) The engineer assembles it verbatim; it is scored on train and
validation together, by day, beside the first law and the baselines, with the score of each day
shown: a second law is kept for the final run if it is lower than the first law on the pooled
development cases and on at least two thirds of the days. Nothing is called validated.

### 13.5 F3 of the normal burner flames, registered 15 November before any validation score

Dossier `research/phys_sflame_f3.md` (committed in `53fb9b0`). Forward `acme_normal.wsp13` with member
specs `wsp14` to `wsp16` (new files `fuel2.rs`, `flame8.rs` to `flame10.rs`, `committee4.rs`; branch
`wsp-f3`, commit `de62db2`). A neutral carry of F2 on the new forward
(`sha256:f2e28a3e…2ba3`) gives exactly F2's tables on 372 s-Flame train prefixes, 150 s-Flame validation
input prefixes, 516 Flame Design prefixes and 210 CFI-G train cases (zero differences).

**Release, stage 1:** `sha256:d1660331c1f96091dad7f6f148c08f50db0cd03317b6c1dec458bd92696a45aa`: the
burner limit as an input of the test, the heating law with a flame-temperature factor (2650 K) and a
conductivity power (0.45), one step on the methane row (`fuel.s` 0.35 for the hinge members, -0.35 for
the growth members, weak-flame shift -350 K) and the ethylene dilution term (`zst0` 0.30, `gz` 1.0);
helium, hydrogen and blends refused. Data-informed from the train days of s-Flame, CFI-G and Flame
Design. On train: s-Flame 0.0736 / 0.0769 (sum, without and with the planned stop) against 0.0914 /
0.0891 for F2; CFI-G unchanged to the third decimal.

**The test.** (a) The s-Flame validation days of 13.1, the nitrogen tests, without and with the
planned stop, beside F2 and the two incidence baselines, once. The author and the coordinator have
seen the aggregates of 13.1 on these cases; that is why (b) and the unread test days matter.
(b) Consistency on data F2 was adopted on: CFI-G validation (shared records and the planned stop) and
the Flame Design validation days, F3 beside F2, once.

**Reading.** F3 stage 1 *replaces F2 on s-Flame* if its sum is lower than F2's on both paths of (a),
by the case mean; it *carries over to s-Flame* if it is also lower than the CFI-G train incidence on
both paths. It *replaces F2 as the normal-flame model* only if, in addition, on (b) its extinction and
termination scores are each not higher than F2's by more than 0.005 on CFI-G validation and on the
Flame Design validation days. Otherwise F2 stays the model of CFI-G and Flame Design and F3 is the
model reported for s-Flame alone.

**Stage 2 (helium, zero-shot Part A of the dossier), if assembled:** a further release on the same
forward with the helium lines only, scored once on the helium-diluted validation tests that carry an
oxygen fraction (8 ethylene tests; never scored by any model) and on the 9 methane-in-helium tests
answered as the registered mixture over 21 and 30 % oxygen, beside the two incidence baselines. It
*answers helium* if its sum is lower than the CFI-G train incidence on the cases it answers.

**Outcome of 13.5 so far (15 November; [feedback/sflame_f3_result.md](feedback/sflame_f3_result.md)).**
(a) s-Flame validation days, nitrogen tests, run once; F2 reproduced 13.1 on the same 46 cases before
any F3 number was read.

| Sum, extinction + termination, case mean (mean of day means) | F3 stage 1 | F2 | CFI-G train incidence |
|---|---|---|---|
| 46 cases, without the planned stop | **0.2048** (0.1425) | 0.2619 (0.1783) | 0.2793 (0.2579) |
| 46 cases, with the planned stop | **0.2051** (0.1427) | 0.3698 (0.2454) | 0.2793 (0.2579) |
| ethylene, 9 cases | 0.040 | 0.041 | 0.223 |
| methane, 37 cases, without / with the stop | 0.245 / 0.245 | 0.316 / 0.450 | 0.293 |
| 7 tests whose burner reaches 250 C before the planned stop | 0.445 / 0.446 | 0.539 / 0.798 | 0.307 |
| the other 9 tests | 0.036 / 0.036 | 0.067 / 0.068 | 0.260 |

By the rule: F3 stage 1 **replaces F2 on s-Flame** and **carries over to s-Flame** (lower than F2 and
than the CFI-G train incidence on both paths, by either mean). The planned stop no longer hurts. The 7
low-flow methane tests remain worse than the incidence baselines: the missing mechanism the author
named (lives rising with flow, which a temperature threshold cannot produce). Two days; the author had
seen the aggregates of 13.1.

(b) Flame Design validation days, 86 cases, run once: F3 0.0299 / 0.3737 without the planned stop
against F2 0.0317 / 0.3933; 0.0066 / 0.0732 with it against 0.0068 / 0.0724. No score is more than
0.005 above F2's (largest rise +0.0008): this half passes. The CFI-G validation half is running; until
it is in, F2 stays the model of CFI-G and Flame Design and F3 is the model of s-Flame.

**Outcome of 13.5 (b), CFI-G half (15 November, sealed chain, run once).** 84 cases of 28 runs on each
data root, no refusal. Shared records: extinction 0.1536 against 0.1537 for F2, termination 0.0956
against 0.0961. With the planned stop: 0.1472 against 0.1473 and 0.0795 against 0.0798. Paired
differences F3 minus F2 between -0.0004 and -0.0001, intervals at or below zero. With the Flame Design
half, no score is more than 0.005 above F2's on the data F2 was adopted on. **By the rule, F3 stage 1
(`sha256:d1660331…45aa`) replaces F2 as the model of the normal burner flames** (CFI-G, Flame Design
and s-Flame). The sealed test partition of CFI-G and the Flame Design test days were scored once with
F2 in the final run of 12.11 and are not scored again; F3's forecasts differ from F2's there only
through the heating law (CFI-G gives no limit input). The whole lineage rebuilt in a scratch copy: 206
releases, no id changed. The helium stage is being assembled.

**Outcome of 13.5, stage 2, helium (15 November, run once; appended to `feedback/sflame_f3_result.md`).**
Release `sha256:819ec0b61814cdfb6b1e3fde5b2bf8ef8a67a9ce455a58f5a46d878407cc9e51`: stage 1 plus the
helium lines of Part A, verbatim. The forward reproduces the dossier's own expectations except the
times to the burner limit, which Part A computed before Part B changed the heating law. On the 17
helium validation tests it answers 35 cases of 14 tests and refuses 7 cases of 3 tests (flow above
the declared range).

| Sum, extinction + termination, case mean | F3 stage 2 | CFI-G train incidence | s-Flame train incidence |
|---|---|---|---|
| all 35 cases (either path) | 0.999 | 0.330 | 0.324 |
| ethylene in helium, 21 cases | 1.512 (extinction 0.799) | 0.364 | 0.361 |
| methane in helium, mixture over 21 and 30 % oxygen, 14 cases | 0.230 | 0.279 | 0.270 |

**By the rule, stage 2 does not answer helium**, and helium stays refused: F3 stage 1 remains the
model. The zero-shot helium law holds ethylene flames until the burner limit, and they went out long
before; the methane half, which the law says flashes out at once, scores below both baselines on 14
cases. The author's own second-weakest assumption was this row ("no helium-diluted flame has been
seen, the critical flux is known to a factor of two"). The helium train days hold only hydrogen and
blends, so there is no helium train flame to revise the row from; these validation tests are now the
only helium data seen, and a revised helium row would have to be judged on the test days.

**Outcome of 13.4 so far (15 November; development numbers, in-sample: train and validation days
together; `feedback/<slug>_second_law_result.md`).**

| Second law | Target | Cases | Parent | Second law | Best baseline | Days lower / days that change | Reading |
|---|---|---|---|---|---|---|---|
| `cld2` (`sha256:870419c3…8721`) | flame lost | 256 | 0.317 (cld D1) | 0.143 | 0.228 | 5 / 8 | not kept |
| `cld2` | radiometer | 195 | 0.119 | 0.119 | | unchanged | |
| `efield2` (`sha256:cf5459b5…d8c5`) | ion current, µA | 3,235 levels | 0.659 (W0) | 0.721 | 0.426 | 7 / 15 | not kept |
| `efield2` | flame out | | 0.148 | 0.100 | 0.070 | 8 / 15 | |

`cld2` reproduces its parent on every case when both flags are 0 (0 of 290 differ). Pooled it halves
the parent's score and is below every baseline, but the two flags are 1 on 8 of 25 days only, and of
those it is lower on 5 and higher on 3. The rule was written to refuse a gain carried by few days; it
did not say how to count days with identical forecasts. Decision taken after seeing the count and
stated as such: those days are left out because they carry no information, and 5 of 8 is still short
of two thirds, so by either count **`cld2` is not kept and cld D1 stays the model**. `cld2` goes to
the final run as a reported comparator beside D1, not as a selected model. `efield2` is higher than
its parent on the ion current and is not kept. `slice2`, `bass2` and BRE's second rule are assembled
(`sha256:d586d0d3…0b61`, `sha256:e008cba3…a4c9`, `sha256:429f4743…8340`) and being scored.

### 13.6 A proper study of the development data, and how a typed candidate is checked (15 November)

The owner asked for the data to be analysed properly and the faults of the models fixed, without a
deadline. The steps of 13.3 and the second laws of 13.4 were typed from summaries, and their expected
effects were hand estimates; three of five steps and one second law then failed. From here each
author studies the development cases (train and validation days) of its investigations itself, case
by case, before typing anything, and may check a typed candidate on those cases under these rules.

- **Predict, then check.** Before a candidate is run, its author writes down the lines changed, the
  physical reason and the effect expected in each stratum. Then it is run once on the development
  cases with the logged what-if tool (`tools/wsp/ext/law_whatif.py`, which writes every evaluation,
  with the hash of its edits, to `runs/dev/gen7/whatif_log.jsonl`) and the realised effect is written
  beside the expectation. A candidate whose effect was not predicted in sign is dropped, not tuned.
- **Bounded.** At most five logged candidates per law in this round. No loop, no search, no number
  produced by a program; every number is typed by the author with its reason. The log makes the
  multiplicity visible.
- **By day.** A candidate is kept for the final run if it is lower than the current model pooled and
  on at least two thirds of the days whose forecasts change (13.4), for the target that loses, and not
  worse by more than 5 % pooled on the other targets. Where fewer than six days change, the candidate
  is carried as a reported comparator, not as the model.
- **Structure before numbers.** Where the fault is a missing input or a missing class of outcome, the
  author names it and the engineer adds it (a new slug); a number is not asked to do the work of a
  missing input.
- **The judge.** These are in-sample development numbers. The test days of every investigation stay
  unread; one final run on them, on a frozen release list, is the only test (12.5).

**13.6 also covers s-Flame.** Its validation days have judged three releases (13.1, 13.5 stage 1 and
stage 2) and are study data from this commit, beside its train days; the test days 20142, 20170 and
20176 and day 19245 stay unread and are the only judge of what follows (hydrogen, blends, a revised
helium row, the low-flow methane flames). The validation days of CFI-G and Flame Design keep their
role and are not opened.

**Outcome of 13.4, the last three second laws (development numbers, in-sample).** `bass2`
(`sha256:e008cba3…a4c9`): quench speed 0.4505 against 0.8135 for bass1 D1 and 0.7079 for the constant,
lower on 12 of 14 days: **kept**, and now the plastics model; rods 0.264 against 0.974 (published
correlation 0.286), slabs 1.781 against 2.371, films 0.061 against 0.163 (correlation 0.044); "kept
burning" identical on shared cases. `slice2` (`sha256:d586d0d3…0b61`): lifting 0.0333 against 0.1474,
lower on 8 of 8 days: **kept**, and now the SLICE model (the by-tube incidence scores 0.0293);
blow-off identical to its parent and far behind its baseline pooled (0.1089 against 0.0480). BRE: read
through its own `lasts` target, the registered W0 scores 0.1955 on extinction against 0.2576 through
the clock and 0.2269 for the planned-stop baseline; the pressure step on top (d2, 0.1800) helps train,
hurts validation and is lower on 9 of 14 days: **not kept**. The reading of W0 through `lasts` is a
candidate of its own and its by-day count is being taken.

**Outcome of 13.6 so far (15 November; development numbers, in-sample: train and validation days
together; each candidate's expectation was logged before its run; official scores through the Rust
forward and the lane scorers equal the what-if runs to round-off).**

| Investigation | Candidate | Target | Current model | Candidate | Best baseline | Days lower / days that change | Reading |
|---|---|---|---|---|---|---|---|
| CLD Flame | `cld` C1 `sha256:541e9b2a…9ab6` | flame lost | 0.3174 (D1) | 0.0431 | 0.2282 | 21 / 25 | kept: the model |
| E-FIELD | `efield` E1 `sha256:dbb4bd64…73b0` | ion current, µA | 0.6592 (W0) | 0.2530 | 0.4256 | 15 / 15 | kept: the model |
| E-FIELD | E1 | flame out | 0.1478 | 0.1478 | 0.0697 | identical | open (efield3 being built) |
| SPICE | `spice` P2 `sha256:f1fe69f0…258a` | smoke-point length, mm | 6.5136 (W0) | 2.1292 | 6.6258 | 9 / 9 | kept: the model |
| SPICE | P2 | flame length given the flow, mm | 3.0715 | 1.9370 | 1.8416 | 9 / 9 | lower, still above the Lin and Faeth law |
| SLICE | `slice2` S2 `sha256:0f5763f5…388a` | blow-off | 0.1089 (s1) | 0.0583 | 0.0480 | 5 / 8 | not kept: comparator |
| Plastics | `bass3` | quench speed (registered coding) | 0.3926 (bass2) | 0.1310 | 0.5786 | 20 / 22 | kept: the model |
| Plastics | `bass2` K1 | quench speed | 0.3926 | 0.3138 | 0.5786 | 20 / 22 | kept; comparator beside bass3 |
| BRE | C3 (to be assembled as d3) | extinction, read through `lasts` | 0.1955 (W0 through `lasts`) | 0.1121 | 0.2269 | 14 / 14 | proposed; assembly queued |
| s-Flame | F4 methane, helium, hydrogen | sum | see `feedback/sflame_f4_result.md` when assembled | | | fewer than six days change | comparators |

What the studies found, in one line each. CLD: one group decides the loss (coflow speed over the
burning speed squared); above its limit 24 of 27 flames are lost, below 0.86 of it none of 223.
E-FIELD: the faults were the saturation cap, the fall after the peak and above all the polarity of the
mesh. SPICE: the nozzle dependence is not a power law. Plastics: below about 4 cm/s the outcome is a
fan setting, not a speed; slabs are two outcomes decided by thickness, sides and starting oxygen.
BRE: pressure has no detectable effect; the fault was charging the burner's heat share at the
adiabatic temperature. SLICE: the crew log's tube cells for six tests of one day are copied down and
wrong; the documented tube is 2.1 mm (second case version), and the blow-off boundary has to be
re-read on it. Burner flames: methane needed no new mechanism; three numbers of the helium row were
contradicted; hydrogen is typed; blends stay refused.

**13.6, burner flames, F4 (15 November; development numbers, in-sample;
[feedback/sflame_f4_result.md](feedback/sflame_f4_result.md)).** Three further releases on the wsp13
forward, each transcribed from `research/phys_sflame_f4.md` and identical, tensor for tensor, to the
child of the author's own logged run: `f4-methane` `sha256:57fe4255…ebe2`, `f4-helium`
`sha256:370ed3cb…62c0`, `f4-hydrogen` `sha256:4cc2fa60…9436`. CFI-G train (0 of 210 cases) and the
Flame Design train days (0 of 444 tables) do not move under any stage; all 237 committed release ids
rebuild unchanged. Blends are refused by the lane's adapter, since no release carries a blend law.
Endpoints: version 1 as coded, version 2 by the general rule of `feedback/sflame_endpoints_v2.md`.

| Sum, without the planned stop | F4 (v1 / v2) | F3 stage 1 (v1 / v2) | F2 (v1 / v2) | CFI-G incidence (v1 / v2) |
|---|---|---|---|---|
| validation, nitrogen | 0.0815 / 0.0401 | 0.2048 / 0.1620 | 0.2619 / 0.2276 | 0.2793 / 0.2605 |
| validation, methane in nitrogen | 0.092 / 0.040 | 0.245 / 0.190 | 0.316 / 0.271 | 0.293 / 0.269 |
| validation, ethylene in helium | 0.180 | refused | | 0.364 |
| validation, methane in helium | 0.006 | refused | | 0.279 |
| train, nitrogen | 0.0730 | 0.0736 | 0.0914 | 0.2725 |
| train, hydrogen | 0.436 / 0.314 | refused | refused | 0.833 / 0.909 |

Each stage changes fewer than six days, so by 13.6 all three are comparators and F3 stage 1 stays the
model. **What the test days can and cannot judge** (inputs only; no endpoint read): days 20142, 20170
and 20176 hold 36 constant-flow tests, all helium-diluted: 26 ethylene tests with no oxygen fraction
per test (answered as the mixture over 21 and 30 %; 21 inside the declared flow range) and 10
hydrogen and methane blends (refused). So F3 stage 1 and `f4-methane` answer no test-day case: the
nitrogen flames of s-Flame have no held data beyond the validation days already used, and the methane
step remains an in-sample result. The test days judge the helium rows only: the zero-shot row of F3
stage 2, which failed its look, against the revised row of `f4-helium`, which was typed from the eight
ethylene flames of one validation day. Day 19245 is not a day of the experiment (eight misnamed files
of a train test). The s-Flame step of the final run is prepared and rehearsed
(`tools/wsp/final/run_final_sflame.py`; version 1 reproduces every registered number of 13.1 and 13.5)
and has not been run.

**13.6, further outcomes (15 November; development numbers, in-sample; official scores through the
Rust forward; result files `feedback/{bre_d3,efield3,cld3,bass4,slice_cases_v2,spice_p3,samer_s1,bass3_s1_q1}_result.md`).**

| Investigation | Candidate | Target | Before | After | Best baseline | Days lower / days that change | Reading |
|---|---|---|---|---|---|---|---|
| BRE | `bre` d3 `sha256:27e2af51…b955`, read through `lasts` | extinction | 0.2576 (W0 through the clock) | 0.1121 (validation 0.0833) | 0.2269 | 13 / 14 | kept: the model |
| BRE | d3 | operator termination | 0.0730 | 0.0398 | 0.0699 | | |
| BRE | d3m `sha256:4b7d155c…2a9f` (methane) | extinction | 0.1121 | 0.0980 | | 1 / 1 | comparator |
| CLD Flame | `cld3` `sha256:b9df5239…9e4b` | flame lost | 0.0431 (C1) | 0.0334 (validation 0.0610 against 0.0935) | 0.2282 | 7 / 9 | kept: the model |
| E-FIELD | `efield3` `sha256:8575550b…6f04` | ion current, µA | 0.2530 (E1) | 0.2103 | 0.4256 | 8 / 9 | kept: the model |
| E-FIELD | `efield3` | flame out | 0.1478 | 0.0630 | 0.0697 | 13 / 15 | |
| SPICE | P3 `sha256:000b88b8…98a0` | flame length given the flow, mm | 1.9370 (P2) | 1.2519 | 1.8416 | 8 / 9 | kept: the model |
| SAME-R | S1 `sha256:e33ee766…1b5f` | particle diameter | 0.2377 (W0) | 0.1717 | 0.2762 | 12 / 12 | kept: the model |
| SAME-R | S1 | detector at full scale | 0.1753 | 0.0562 | 0.1012 | 10 / 10 | |
| Plastics and fabrics | `bass4` `sha256:ceefc15d…ab86` | kept burning, 19 fabric burns | refused | 0.0529 | 0.3801 | | kept: the model (every PMMA forecast identical to bass3) |
| Plastics | bass3 Q1, bass3 S1 | quench speed; kept burning | 0.1522; 0.0467 (second coding) | 0.1348; 0.0247 | | 2 / 2; 1 / 3 | comparators (S1 is worse on the first coding) |
| SLICE | second case version (documented tubes), no new release | blow-off of `slice2` s1 | 0.1089 | 0.0456 | 0.0473 | | the fault was the copied log cells; S2 is worse on this version (0.0511) and is dropped |

Flags. `efield3` passes the rule pooled and by day but is worse than E1 on the validation days on both
targets (0.2455 against 0.2252; 0.0850 against 0.0753), with three validation days moving against the
author's written expectation; E1 goes to the final run beside it. BRE's d3 read through its clock and
through `lasts` disagrees by up to 0.95 in probability: only the `lasts` reading is the model, and the
clock target of d3 is not to be served. The SIBAL part of `bass4` was typed before the fabric cases
were opened (time-stamped in the study); the Nomex and Ultem floor is data-informed.

### 13.7 n-Heptane droplets: D3 of `flex_heptane.wsp2`, registered 15 November before any validation score

**Why.** The post-hoc comparison with Farouk and Dryer (2014) on FLEX-035
([RESULTS_POINT_METRICS.md](RESULTS_POINT_METRICS.md)) named three suspects in the adopted n-heptane
release D2 (`sha256:43f7871f…588d`): the end of the hot flame is spread over many seconds and comes
late, the droplet then burns too fast in the cool flame, and the point forecast is the median of a
mixture of scenarios. FLEX-035 (`PSI-69:181F001:2009-11-27`) is a validation run and its whole
trajectory was studied for that comparison. **It is recorded as studied, not used to set numbers**: the
question was taken to the train partition, and only what the train runs support was typed. Study, tables
and expectations: [RESULTS_HEPTANE_D3.md](RESULTS_HEPTANE_D3.md).

**What train supports, and what it does not.**

| Suspect | Train evidence | Decision |
|---|---|---|
| Cool-flame rate too high for large droplets and oxygen above 18 % | Twelve sustained cool flames in nine sessions burn at 0.31 to 0.36 mm²/s with no trend beyond 0.02 in oxygen (15 to 24 %), CO2 (0 to 25 %) or pressure (0.7 to 1 atm); D2's law gives 0.41 and 0.44 where 0.36 and 0.34 were measured, and 0.43 for a 3.85 mm droplet in air. Measured on 14 November by the fifth generation and its independent check, and again by the author on 15 November | **Changed**: the cool-rate law of `flex_heptane.wsp5` P0, typed on 14 November, copied digit for digit |
| Hot-flame end late and wide | The radiative clock of D2 is within 0.13 (in ln) on twelve of the thirteen nitrogen and CO2 runs at 1 atm (the thirteenth, a 1.8 mm droplet at 14 % oxygen, +0.23); no train run in air above 3.36 mm lost its flame to radiation (3.56 and 3.58 mm oscillated and burned on, both drifting fast); the near-critical runs that lost the flame and those that kept it show the same prefix rate ratios (0.91 to 1.03 against 0.88 to 1.10). What orders them, the drift of the droplet, is not a prefix observation | **Not changed.** A missing input, not a number. The widths stay: they are not set from the runs a law was typed from (sections 8 and 12.3) |
| Median of a mixture | Under the model's own distribution the median is the point with the least expected absolute error, and the 0.5 row must stay the median for the band to be a distribution. At the diagnosed condition the most probable path loses its hot flame at the clock median or later (10.3 s and more), not earlier, so following it would not move the error that was diagnosed | **Not changed.** No new forward |

**Candidate.** `flex_heptane.wsp2` D3, a literal revision of D2 on the frozen forward `droplet3.rs`
(`configs/models/flex/wsp2/heptane/d3-fable/`; edits sha256 `80382d64…0578`, five statements; release
`sha256:5feb609c3dd633b95b512696717beecd73a2b28e20a3be3c4ec6893b683046ad`). DATA-INFORMED. Exposure in its
`meta.json`; the two entries that matter: FLEX-035 studied and not used; the aggregate validation score
of P0, which holds this law among four changes (47.2 against 46.1 µm, section 10).

**Train, before the freeze** (`wsp-dev feedback --partition train`, 125 cases, 20 sessions). Rule of
section 8: median CRPS 21.7 against 21.8 µm, better or level in 17 of 20 sessions (3 better, 14 level,
3 worse): **passes**. Mean 44.2 against 46.2 µm, and the whole of that gain is the one run at 2 atm;
without it 43.4 against 43.1. At a horizon of 39 s the MAE of the median is 77.0 against 81.4 µm
(without the 2 atm run 75.5 against 76.7). Against the twelve measured rates the law is within 6 %
everywhere, where D2 was off by 11, 15 and 29 % on three runs; it is closer on seven runs, level on two
and farther on the three at 15 to 17 % oxygen, where D2 was within 2 %. The forecasts of the 1 atm
and 0.7 atm runs do not gain from it on train: where D2's hot
flame ends too early its fast cool flame was hiding the error (C09H501, worse by 10 to 17 µm), where
the hot flame ends on time the forecast improves (301F001, C02H101 at 10 s). So train does not promise
a gain on validation; the law is taken there because it is the measured one.

**The look, once.** `wsp-dev feedback --release D3 --release D2 --partition validation`, cutoffs 1, 3 and
5 s, at the horizon of 10 s and at 39 s (to the end of the records; 5 + 39 s is inside the forward's
45 s). The three cases of FLEX-035 are left out of both releases, because the run was studied. Only
the aggregates printed by `tools/wsp/analysis/partition_aggregate.py --skip PSI-69:181F001:2009-11-27`
are read: session-weighted means, case means, coverage, and counts of sessions lower, level or higher.
No per-run and no per-session value is read. The heptane validation days have judged about seven
aggregate looks before this one (dossier 7.2).

**Rule.** D3 replaces D2 as the carried n-heptane model if all three hold on those cases:

1. CRPS at the 10 s horizon, session-weighted, is not higher than D2's (compared to 0.1 µm). This is the
   standing veto of sections 8 and 12.4.
2. MAE of the median at the 39 s horizon, session-weighted, is lower than D2's by at least 1.0 µm. This
   is the claim under test: long forecasts.
3. At the 39 s horizon more sessions are lower than higher on the MAE of the median (a session counts
   when it moves by more than 1 µm).

Otherwise D2 stays and D3 is kept in the tables as a negative or undecided result. A near miss is a
refusal. No second heptane candidate is taken to these validation days for this question.

**Not part of the rule.** The score of D3 on FLEX-035 against the published curve is taken by the lane
that made that comparison and is labelled exposed. The unread n-heptane data that could judge a later
candidate fairly are listed in [RESULTS_HEPTANE_D3.md](RESULTS_HEPTANE_D3.md); none of them is scored
here.

**Outcome (15 November, the one look, taken after commit `e9f8137`).** 36 common cases in 6 sessions
(the three cases of FLEX-035 were the whole of the seventh session); no refusal differs between the
releases.

| Criterion | D3 | D2 | Holds? |
|---|---|---|---|
| 1. CRPS at 10 s, session-weighted | 51.3 µm | 49.9 µm | no (+1.4) |
| 2. MAE of the median at 39 s, session-weighted, lower by 1.0 µm | 101.6 µm | 98.6 µm | no (+3.0) |
| 3. Sessions at 39 s, MAE of the median: lower against higher | 0 lower, 3 level, 3 higher | | no |

Beside the rule: MAE of the median at 10 s 69.2 against 67.4 µm; CRPS at 39 s 69.5 against 67.5 µm; by
session the CRPS is lower in none, level in 4 and higher in 2 at both horizons; coverage of the 50 %
and 90 % bands 0.74 and 0.97 for both at 10 s. Case means: CRPS 53.2 against 52.2 at 10 s, 75.8 against 74.3 at 39 s.

**D3 is not adopted; `flex_heptane.wsp2` D2 stays the carried n-heptane model.** All three criteria
fail, and none nearly. The cool-flame rate law that fits the twelve measured train rates better does
not forecast the validation droplets better, at either horizon, as it did not on train outside one
run; it is the second time this law goes to these days and comes back slightly worse (P0, section 10,
with three other changes). What it shows: on these runs the steady cool-flame rate is not the weak
link, and D2's too fast cool flame is offset by its other errors (the time of the loss, the sag of the
rate around it). What it does not show: that D2's law is the right physics, or anything about a 3.85 mm
droplet in air, which no train run resembles. D3 stays in the tables as a negative
result. By the rule no second candidate goes to these days for this question; the next judge has to be
data nobody has read ([RESULTS_HEPTANE_D3.md](RESULTS_HEPTANE_D3.md), section 6).

### 13.7 The final run of the seventh generation, registered 15 November before any test-day outcome is read

The owner asked for the work to be closed. The one final run of 12.5 for the investigations of 13.2
and for s-Flame, on their unread test days, once.

**Release lists, frozen.** `tools/wsp/final/releases_final_gen7.json`, sha256
`f35560716ed8b9aa4606a8db42feff97520e2998dd77cf1fece7e80f911c70e2` (7 models and 27 comparators; every
id checked against the registry); `tools/wsp/final/releases_final_sflame.json`, sha256
`ba34f8f74f2497a893a70ec65348f69f96f8c8388a7eca115593819751f4e9c9`.

| Investigation | Model | Comparators | Test days |
|---|---|---|---|
| BRE | `bre` d3, read through `lasts` | W0 through the clock (the zero-shot reading of 13.2), W0 through `lasts`, d1, d2, d3m | 3 days, 56 tests |
| CLD Flame | `cld3` | `cld` W0, D1, C1; `cld2` | 7 days, 67 test points |
| E-FIELD | `efield3` | `efield` W0, E1; `efield2` | 4 days, 41 tests |
| SLICE (second case version) | `slice2` s1 | `slice` W0, D1; `slice2` S2 | 3 days |
| SPICE | `spice` P3 | P2, W0 (variant B), the published-fit candidate | 2 days, 25 flames |
| SAME-R | `samer` S1 | W0, D1 | 3 days, 34 rows |
| Plastics and fabrics (second coding) | `bass4` | `bass3` and its two comparators, `bass2` and K1, `bass1` W0 and D1 | 13 days, 48 burn rows |
| s-Flame (second endpoint version) | F3 stage 1 (answers no test-day case: they are all helium) | F2, F3 stage 2 (zero-shot helium), F4 methane, F4 helium, F4 hydrogen | 3 days, 36 constant-flow tests |

SAME and DAFT were scored whole in 13.2 and have no test days. FLEX-2 and CFI droplet candidates are
not in this run (their held droplets were spent in 12.11 and the third batch did not finish).

**Tools.** `tools/wsp/final/run_final_gen7.py final --slug <s> --releases <list> --sha256 <sha> --execute`
and `tools/wsp/final/run_final_sflame.py final`. Their rehearsals on the validation days rebuilt the
cases from raw and reproduced every registered number of 13.1, 13.2, 13.3 and 13.5 and every
validation-partition number of the newer releases (466 rows, largest difference 0). A marker per
investigation is written before anything is read, so no step can run twice; reveals are logged.
Aggregates only go to `RESULTS_FINAL_GEN7.md` and `feedback/`.

**What each result will be taken to mean**, per scored target, on the test-day cases the model answers:
the model *holds* if its score is lower than every baseline's; it *holds against the simple baseline
only* if it is lower than the constant or incidence but not than the best baseline; it *does not hold*
otherwise. The improvement of this generation *holds* if the model is also lower than its zero-shot
parent of 13.2 on the same cases. For s-Flame, the revised helium row *answers helium* if the sum of
F4 helium is lower than the CFI-G train incidence on the cases it answers; F3 stage 2 is reported
beside it. Counts and refusals are reported with every number; with two to thirteen days no interval
is called stable and nothing is called validated.

**Known risks, stated beforehand.** The SLICE test pages have never been parsed; plastics, SPICE,
SAME-R and SLICE read inputs and outcomes from one record, so their inputs could not be counted in
advance. If a step stops with an error before it has written any aggregate, the fault is fixed and the
step is run; if it has written aggregates, that is the run, whatever they show. `efield3` was worse
than E1 on the validation days. The s-Flame helium row was typed from eight flames of one day. No
number of any release is changed after this run in this generation.

**Outcome of 13.7 (15 November; run once on the frozen lists; tables in [RESULTS_FINAL_GEN7.md](RESULTS_FINAL_GEN7.md)
and [RESULTS_FINAL_SFLAME.md](RESULTS_FINAL_SFLAME.md)). Test days only; lower is better.**

| Investigation | Target | Cases | Model | Zero-shot parent | Simple baseline | Best baseline | Reading |
|---|---|---|---|---|---|---|---|
| BRE | extinction | 95 | 0.1389 (d3) | 0.2448 | 0.2220 | 0.2220 | holds; the improvement holds |
| BRE | operator termination | 95 | 0.0535 | 0.1070 | 0.1031 | 0.0999 | holds |
| E-FIELD | ion current, µA | 788 levels | 0.3190 (efield3) | 0.7039 | 0.9864 | 0.5746 | holds; E1 0.3238 |
| E-FIELD | flame out, ends in window | 39 | 0.0547 | 0.1072 | 0.1331 | 0.1331 | holds |
| E-FIELD | flame out, integrated Brier | 39 | 0.0348 | 0.0282 | 0.0394 | 0.0394 | holds against the baselines; not better than the zero-shot parent |
| SPICE | smoke-point length, mm | 6 | 4.18 (P3) | 6.94 | 10.58 | 10.33 | holds (the non-blind published-fit candidate 3.16) |
| SPICE | flame length given the flow, mm | 6 | 1.18 | 2.98 | 10.58 | 2.56 | holds |
| SAME-R | particle diameter | 33 | 0.1672 (S1) | 0.2595 | 0.3609 | 0.2559 | holds |
| SAME-R | detector at full scale | 33 | 0.0144 | 0.0807 | 0.2997 | 0.0189 | holds |
| CLD Flame | lost before fuel-off, Brier | 63 | 0.0175 (cld3) | 0.0545 | 0.1022 | 0.0568 | holds |
| CLD Flame | flame lost, integrated Brier | 63 | 0.1239 (cld3) | 0.0786 | 0.1020 | 0.1020 | **does not hold**: D1 0.0726 and W0 are better |
| CLD Flame | radiometer | 63 | 0.1145 | 0.1411 | 0.3145 | 0.1439 | holds |
| Plastics and fabrics | kept burning (case mean) | 42 | 0.0477 (bass4) | 0.0588 (34 cases) | 0.0707 | 0.0707 | holds |
| Plastics | quench speed, mean of day means | 38 | 2.9103 (bass4) | 1.6069 (32 cases) | 2.7049 | 2.7049 | **does not hold** (case mean 1.54 against 2.11; bass1 D1 0.98 on 32 cases) |
| SLICE | blow-off | 31 lines | 0.0021 (slice2) | 0.0021 | 0.0028 | 0.0000 | no event among the answered lines: says little |
| SLICE | lifting | 31 lines | 0.0041 | 0.0466 | 0.0008 | 0.0000 | no event; the false-alarm level fell tenfold |
| s-Flame, helium | sum | 49 of 60 | 0.708 (F4 helium) | 1.057 (F3 stage 2) | 0.481 | 0.416 | **does not answer helium** |

Readings by the rule written beforehand. Held on every scored target: BRE, SPICE, SAME-R, E-FIELD
against its baselines (its newest law equals E1 on the current and is better on flame-out in the
window), the plastics "kept burning" answer with fabrics. Did not hold: the time course of flame loss
of the newest CLD law (it forecasts whether a flame is lost better than anything else, 0.0175, and when
worse than a constant; the first revision D1 beats every baseline on all three scores); the quench
speed of plastics by the registered day mean (the two-class slab law typed from the development burns
did not carry; the test days score ten times higher than development for every release); the revised
helium row of the burner flames (typed from eight flames of one day). SLICE's test lines hold no
blow-off or lifting event, so its test says little. The s-Flame step first stopped on a tooling error
before any aggregate was written (the tool's default binary predates the wsp13 forward); it was run
once with the merged binary after its rehearsal reproduced the registered numbers, as 13.7 allows; the
marker of the stopped attempt is kept.

**What is served after the run, and what was chosen after seeing it.** BRE d3 (through `lasts`),
`efield3`, SPICE P3, SAME-R S1, `slice2` s1, `bass4` for "kept burning", F3 stage 1 for the normal
burner flames in nitrogen (helium, hydrogen and blends refused). Two choices are made AFTER the test
and are labelled so wherever they are shown: CLD Flame is served by D1, and the plastics quench speed
by bass1 D1; their test-day numbers are no longer an unbiased estimate for those two choices. The
generation is closed: no number of any release is changed from this run.

### 13.8 Revisions after the final run, by the owner's decision (15 November)

The final run of 13.7 is the record of the releases it scored and is not rerun or altered. The owner,
told that the test days are spent and that any further change can only be judged on data it has
seen, decided: modify the values nonetheless, for the answers that stayed weak. From this commit the
test days of the plastics investigations, of s-Flame and of the BASS-II fabric burns are study data
like the others. Three revisions, one each, no further rounds: the quench speed of plastics, the
helium row (and the hydrogen row) of the burner flames, the quench boundary of the fabric flame.

**How they are labelled, everywhere they appear** (result files, cards, the served list, the table of
the 19): "typed after the final run with every partition seen; in-sample on all data; no held-out
score exists". Their scores are given on train, validation and test days separately beside the
baselines, as descriptions of fit, never as test results. A revision is served only if it is lower
than the constant or incidence baseline on each of the three partitions for the target it repairs
and changes no other served answer of its investigation by more than rounding; otherwise the refusal
of 13.7 stands. The releases scored in 13.7 keep their ids and their results.

**Outcome of 13.8, plastics (typed after the final run with every partition seen; in-sample on all
data; no held-out score exists; [feedback/bass5_post_final_result.md](feedback/bass5_post_final_result.md)).**
Two burns carried 87 % of the test loss of `bass4`. The larger, O13a, was a coding fault: in the
workbook row the pot settings are in the fan cell and the fan displays in the air cell, so the
displays were read as speeds; it is a floor quench at pot 0.31, which the law forecast. With it a
general fault of the second coding was found (a low-flow quench was coded without checking that the
flame went out at a setting lower than the one held before) and corrected in a third coding
(`feedback/bass1_coding_v2.md`, addendum; rules R11 to R15; four burns change, none on the validation
days). The other burn is a slab in the wrong class, as one on the train days: width separates them
(sideways conduction into the metal frame), and `bass5` (`sha256:050a4ceb…b82d`) adds a width input
and one number in five places; every "kept burning" forecast is identical to `bass4`.

| Quench speed, cm/s, mean of day means, third coding | `bass4` | `bass5` | constant |
|---|---|---|---|
| train | 0.1611 | 0.1143 | 0.6709 |
| validation | 0.1082 | 0.1082 | 0.5466 |
| test | 0.5633 | 0.1255 | 0.9157 |

`bass5` is lower than the constant on each partition: by the rule it is served and the harness answers
the quench speed again. On the corrected coding `bass4`, the model the final run scored, is itself
below the constant on the test days (0.5633 against 0.9157): the failure recorded in 13.7 came from
the two faults above. Both statements rest on a coding corrected after the test outcomes were seen.

**Outcome of 13.8, s-Flame helium (after the final run with every partition seen; in-sample;
[feedback/sflame_post_final_result.md](feedback/sflame_post_final_result.md)).** The author proposed
no change to the helium row: the fault on the test days was in the cases. Eleven of the 26 test-day
ethylene tests ramp their flow after ignition (`ignitionToTestRampTimeSec` in the script; the
constant-flow rule read the key and did not use it); eight of them were scored and carried 78 % of the
loss. A third case version excludes flow-ramped tests from the constant-flow set (no train or
validation test is affected) and, beside it, takes the oxygen fraction of a two-candidate day from the
pressure block (30 % before the vent and refill, 21 % after), where the blocks and the named fractions
match in number.

| Helium cases, sum, without the planned stop | cases | F3 stage 2 (zero-shot) | `f4-hydrogen` | CFI-G incidence |
|---|---|---|---|---|
| validation days | 35 | 0.9990 | 0.1105 | 0.3299 |
| test days, as built for the final run | 49 | 1.0566 | 0.7080 | 0.4812 |
| test days, flow-ramped tests excluded | 31 | 1.0226 | 0.2419 | 0.4143 |
| test days, and oxygen by pressure block | 31 | 1.1535 | 0.1664 | 0.4143 |

Hydrogen on the train days 0.3140 against 0.9087; CFI-G (0 of 210 cases) and Flame Design (0 of 444
tables) unmoved; nitrogen ethylene identical to F3 stage 1. By the rule `f4-hydrogen`
(`sha256:4cc2fa60…9436`: F3 stage 1 plus the methane step, the helium row and the hydrogen row) is
below the incidence on the helium cases of the validation days and of the test days once the
flow-ramped tests are excluded: it is served, and s-Flame answers helium and hydrogen; blends and
flow-ramped tests are refused. The result of the final run on the cases as built (0.708 against 0.481)
stands as recorded.

**Outcome of 13.8, fabric quench boundary ([research/phys_quench_post_final.md](research/phys_quench_post_final.md)).**
One number was typed (the fabric oxygen floor 14.8 % to 14.1 %) and checked once; every direction came
out as predicted, and it does not pass: the ignition outcome is 0.1612 against 0.1333 for the constant
on the validation burns and 0.2912 against 0 on the two test burns, which both lit. The burns bracket
the boundary without placing it (each quenched sample has a survivor at the same reading), and passing
would take a boundary threaded between two burns, which the author declined to type. **Not served: the
refusal of the quench question for fabric stands**; "does the fabric keep burning" is answered by the
solids model.

### 13.9 n-Heptane droplets: D4 of `flex_heptane.wsp10`, registered 15 November before any non-train score

**Why.** After 13.7 two things were left open for the large droplets near the critical diameter
([RESULTS_HEPTANE_D3.md](RESULTS_HEPTANE_D3.md), [RESULTS_POINT_METRICS.md](RESULTS_POINT_METRICS.md)):
the time the hot flame is lost, and the stage between that loss and the steady cool flame, which
needs a forward. By 13.7 no further candidate goes to the validation days for this question; the
judge has to be data nobody has read. FLEX-035 (`PSI-69:181F001:2009-11-27`, validation) is
**studied, not used to set numbers**, as in 13.7; the author did not score it.

**Candidate.** DATA-INFORMED. `flex_heptane.wsp10` D4, release
`sha256:f3ccc122172b9d680cd17af5183bc640bf643d1175ed576b684ec9b6f324b164`, a literal revision of the
baseline `sha256:47fb0df64160972d2272974f9a35232eab8d64dd5ebfb5913127a86cfb0b6fb7` of the new spec
`flex_heptane-wsp-v10` (key `flex_heptane10`). Forward `crates/ignis-wsp/src/droplet10.rs`: a copy of
the frozen `droplet3.rs` with one stage added, off when `trans.depth = 0`. Files:
`configs/models/flex/wsp10/heptane/` (`spec.json`, `binding.json`, `baseline-d2/`, `d4-fable/`).
Exposure is in `d4-fable/meta.json`.

| Part | What is typed | Source |
|---|---|---|
| Everything of D2 but its cool-flame rate | copied digit for digit (76 statements) | `wsp2/heptane/d2-fable/edits.wsp` |
| Cool-flame rate | the five statements of D3: 0.325 mm²/s at 2 mm, size exponent 0.15, no oxygen, CO2 or pressure term to 1 atm, helium +0.2 per 20 %, +0.5 per e-fold above 1 atm | twelve train cool flames (13.7) |
| Transition stage | a cool flame that follows a hot flame lost in the forecast burns at K_c (1 − a·exp(−Δt/τ)); `trans.depth` a = 0.25 (starts at three quarters of the steady rate: 0.26 mm²/s for a 3.4 mm droplet, between the unlit droplet in the same gas, 0.21, and the steady flame, 0.35) | the train sag, 0.25 to 0.33 mm²/s before 0.35 in five train sessions (RESULTS_HEPTANE_D3 2.4); the one new number |
| | `trans.tau_s = 3.7`, `trans.tau_exp = 0.2`: the relaxation time of the gas a flame heated | copied from D2's `post.tau0_s`, `post.tau_exp` (thirteen train runs) |
| Radiative clock, critical diameter, widths | **unchanged from D2** | see below |

**The radiative clock: derived, failed on train, not typed.** The balance asked for was written out
(section 4b of the edits file): optically thin emission of the CO2 and H2O inside the flame sphere
against the heat release (π/4)ρ_l K d_s q_c, the flame dying at the loss fraction
(T_ad − T_x)/(T_ad − T_∞) with T_x = 1200 K, and the flame diameter from the transient field of a
point source of fuel (final standoff 12 in air, gas diffusivity 1.6 cm²/s). With those constants and
no train number it gives a flame of 7 to 8 d0 at the loss, a loss fraction that peaks and falls,
hence a critical diameter, and a peak at 8.0 s for 194F001 (lost at 7.9 s) and 6.6 s for C01H302
(8.0 s). As a clock it fails: the first crossing comes at 3.6 and 2.9 s for those two runs, and at
1.4 s for a droplet 30 % above its critical diameter, where the train runs lose the flame near 5.5 s.
The flame sphere fills within two seconds; what the train runs show is a loss that keeps growing for
seconds after that. A cold far field was then tried (five pairs of the two growth constants); one
pair came within 0.3 of D2's clock. Taking it would have been a choice by agreement, so it was
dropped and set nothing. A balance with an accumulating radiating volume gives a time independent
of d0, against six train pairs that fall as 1/d0. **So the candidate does not move the hot-flame
end; the 10.3 s median of D2 at 3.85 mm in air stands in D4.** This part of the task is unfinished.

**Train, before the freeze** (`wsp-dev feedback --partition train`, 125 cases, 20 sessions; one logged
candidate, nothing retyped; expectations in `runs/dev/hept_d4/expectations_20261115.txt`, local).

| | D4 | D2 | D3 |
|---|---|---|---|
| CRPS at 10 s, median / mean | 19.5 / 44.6 µm | 21.8 / 46.2 | 21.7 / 44.2 |
| Rule of section 8 against D2 | **passes**: 5 sessions better, 11 level, 4 worse | | |
| MAE of the median at 10 s, session-weighted | 53.6 µm | 59.4 | 53.9 |
| MAE of the median at 39 s, case mean / session-weighted | 77.5 / 73.1 µm | 81.4 / 82.1 | 77.0 / 73.4 |

Against D2 the candidate is better; against D3 it is level. The transition stage moves single runs
both ways at 39 s: lower on N04H101 (−12 µm against D3), N02H101 (−27), C04H101 (−16) and C08H101
(−4); higher on C02H101 (+29), C09H501 (+18), C03H601 (+14) and C02H201 (+14). I expected it to gain
1 to 4 µm on D3 and it did not; I expected C02H101 lower and it came out higher. Where D2 loses the
hot flame too early, a slower start of the cool flame adds to that error. So train supports the
stage as a mechanism (five sessions show the sag) and does not show that it improves forecasts.

**The fair judge.** A blind test on n-heptane tests nobody has read, measured in parallel by another
agent under `runs/dev/heptane_fresh/`; the author read nothing there. Releases: D4 against D2
(`sha256:43f7871f…588d`), same cases, cutoffs and times for both. **Rule:** D4 is better than D2 on
those tests if both hold: (1) the session-weighted MAE of the median at the 10 s horizon is lower
than D2's; (2) the session-weighted nRMSE of the median over the available burn (from each cutoff to
the end of the measured series inside the forward's 45 s) is lower than D2's. Otherwise D2 stays the
carried n-heptane model and D4 is kept as a negative or undecided result. A near miss is a refusal.
One look. The score of D4 on FLEX-035 is taken by the lane that made that comparison, after this
freeze, and is labelled exposed; it cannot adopt D4.

**Reproduce** (the binaries of `target-merged/` predate this forward; use `target-d4/release/`):

```bash
CARGO_TARGET_DIR=target-d4 cargo build --release -p ignis-cli -p ignis-wsp-dev
```

```bash
target-d4/release/ignis.exe wsp compile --family flex_heptane10 --edits configs/models/flex/wsp10/heptane/d4-fable/edits.wsp --meta configs/models/flex/wsp10/heptane/d4-fable/meta.json --out configs/models/flex/wsp10/heptane/d4-fable/patch.json
```

```bash
target-d4/release/ignis.exe wsp assemble --family flex_heptane10 --patch configs/models/flex/wsp10/heptane/d4-fable/patch.json --binding configs/models/flex/wsp10/heptane/binding.json
```

```bash
target-d4/release/wsp-dev.exe feedback --release sha256:f3ccc122172b9d680cd17af5183bc640bf643d1175ed576b684ec9b6f324b164 --release sha256:43f7871f778040550291789ca97e319729675a9dd0df9c9529525c1805ad588d --partition train --horizon-s 39 --out runs/dev/hept_d4/fb_train_d4_d2_h39.json
```

`target-d4/release/ignis.exe infer forecast --release <id> --run <train run> --cutoff 3` was run once on a train run and is admitted. Not run: `cargo test -p ignis-wsp`, `tools/wsp/reassemble.py`
(the line to add is `("flex_heptane10", "configs/models/flex/wsp10/heptane", "binding.json", ["baseline-d2", "d4-fable"])`).

**Amendment, 15 November, before any score of D4 or D2 on the fresh batch exists.** The fresh batch
arrives in parts (the 2013-03-28 day of FLEX-2 is measured; four PSI-69 days of 2013 are still being
fetched). The one look of the rule above is split into two stages, each taken once, with the same
releases, metric definitions and cases rule: **stage A** on the tests of 2013-03-28 that are usable and
marked visual PASS when this amendment is committed (E087C01, C03, C04, C06, C07); **stage B** on every
other usable, visual-PASS test of the batch, once its measurement is finished. Cutoffs 1, 3 and 5 s
after the time origin of the series; a (test, cutoff) case enters only if the series has at least
five valid points after the cutoff; both releases are scored on exactly the same cases. D4 is better
than D2 only if **both** criteria hold in stage A **and** in stage B. Stage A alone is reported as
"first day of the blind test" and adopts nothing. The session of stage A is one day, so "session-
weighted" there equals the case mean over its tests; the conditions of that day are an assumption
(air carried 14 days, `runs/dev/heptane_fresh/README.md`), which is stated with every stage-A number.

**Stage A outcome, 15 November** (`tools/wsp/final/fresh_heptane_score.py`, scorer and prefix builder
committed before the look; the prefix builder gives forecasts identical to `infer forecast` on a train
run; 9 cases on 5 tests of 2013-03-28, none refused; per-case rows in
`runs/dev/heptane_fresh/stageA_score.json`, local):

| | D4 | D2 | Holds |
|---|---|---|---|
| (1) MAE of the median, 10 s horizon | 18.9 µm | 20.7 µm | yes |
| (2) nRMSE of the median over the available burn | 3.84 % | 3.68 % | **no** |

**D4 is not better than D2: criterion 2 fails in stage A, so stage B cannot change the outcome. D2
stays the carried n-heptane model; D4 is kept as a negative result.** Where it moves: the large
droplet E087C07 (3.84 mm, air assumed, 1 atm, measured to the planned stop at 40.6 s) is D4's gain at
10 s (MAE 5.3 to 8.4 µm against 7.2 to 19.8) and its loss over the burn (nRMSE 6.7 to 7.5 % against
6.2 to 7.3 %); the four smaller droplets leave the view or vanish within 13 s and score alike. Stage B
may still be scored as information when the rest of the batch is measured.
