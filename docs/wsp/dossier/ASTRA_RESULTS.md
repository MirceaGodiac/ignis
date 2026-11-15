# Astra's first extension: results and retained limits

14 November 2026. Main author and independent research analysts: **GPT-6 Astra**.
All ten new candidates are **data-informed, not zero-shot**, `not_trained`, and
`experimental`. This checkpoint completes the first instruction's five-model extension;
the subsequent `ASTRA_PROMPT_ALL_NASA.md` expands the continuing work to 19 investigations.
It does not turn this checkpoint into completion of that larger objective.

## Decision

Keep Fable's carried models unchanged. Preserve every Astra R1 and equal-weight joint
candidate as an experimental alternative and negative-result record. The joint BASS
forecast improves train tip CRPS slightly; none of the other new forecasts improves its
principal train score. No new validation look or final test was run. No coefficient was
changed after these scores. No unseen-session improvement is established.

The [design](ASTRA_DESIGN.md), [pre-weight error budget](ASTRA_ERROR_BUDGET.md),
[property ledger](ASTRA_PROPERTIES.md), [research](../research/astra_research_analysis.md),
and [ten findings](review/README.md) explain the scientific choices and their limits.
The numerical budget followed the first implementation prototypes; it preceded the new
weights. That chronology is disclosed rather than presented as blind preregistration.

## What was built

| New model names | New mechanism | Joint statement |
|---|---|---|
| `flex_methanol.wsp6`, `flex_heptane.wsp6` | Coupled liquid mass/energy integration with reference vapor pressure, latent heat and molar Stefan diffusion; limited to future hot-radiative endings without He/CO2 | Equal complete diameter distributions, including re-inverted quantiles; independent predecessor encoding retained |
| `acme_normal.wsp6`, `acme_inverse.wsp6` | Five persistent flow/pressure scenarios; composition, supplied Tad and Zst remain fixed | Half the central Fable forecast plus half Astra's five-node distribution |
| `bass_front.wsp3` | Steady oxygen-supply speed from gas transport and fuel loading | Equal position distributions; both speed members use the new coherent first-event interpretation |

The BASS joint's events are **not** the mean of the old artifact's event rows: F-007
identified a nonmonotone inherited first-event interpretation. The new public coordinates
retain `p_quenched = Fq` and `p_reached_end = Fe/(1-Fq)`, so existing total-ended scoring
remains meaningful. The neutral path still reproduces the predecessor exactly.

New releases use explicit complete-literal rewriting with real parent ancestry across
specifications, then ordinary same-spec revisions for the joints. They are not root W0
releases with exposure erased. [The inventory](../feedback/astra_literal_inventory.json)
checks exact inherited transcription and current forward hashes for all ten releases.

| Family | R1 card | Joint card | New R1 positions | New directly train-read positions |
|---|---|---|---:|---:|
| Methanol | [R1](../../../configs/models/flex/wsp6/methanol/r1-astra/CARD.md) | [joint](../../../configs/models/flex/wsp6/methanol/joint-astra/CARD.md) | 50 | 0 |
| Heptane | [R1](../../../configs/models/flex/wsp6/heptane/r1-astra/CARD.md) | [joint](../../../configs/models/flex/wsp6/heptane/joint-astra/CARD.md) | 50 | 0 |
| Normal ACME | [R1](../../../configs/models/acme/wsp6/normal/r1-astra/CARD.md) | [joint](../../../configs/models/acme/wsp6/normal/joint-astra/CARD.md) | 15 | 0 |
| Inverse ACME | [R1](../../../configs/models/acme/wsp6/inverse/r1-astra/CARD.md) | [joint](../../../configs/models/acme/wsp6/inverse/joint-astra/CARD.md) | 15 | 0 |
| BASS | [R1](../../../configs/models/bass/front3/r1-astra/CARD.md) | [joint](../../../configs/models/bass/front3/joint-astra/CARD.md) | 12 | 1 |

The main author typed 142 new R1 scalar positions and 13 joint-patch positions, with
reasons beside the assignments. These are position counts, not distinct scientific
choices. The copied Fable block remains data-informed. Its exact direct-train count is
unknown; a [separate origin audit](../feedback/astra_inherited_origin_audit.md) establishes
measurement-backed lower bounds of five normal-ACME, nine inverse-ACME and one BASS
inherited positions, plus separately documented mixed-origin numbers. Zero new direct
train readings does not mean a data-free full model. Fixed spec positions, explicit
literal positions and authored tensor capacity are separately counted.

## Train comparison

All generic comparisons use the shared `data` root, cutoffs 1/3/5 s, and the existing
scorer. FLEX predicts the next 10 s. ACME uses fixed horizons
6/8/10/12/15/20/25/30/40/50/60/80/100/120 s. These ACME numbers use the shared-record
fallback, not the unresolved prospective script-plan input. BASS uses only the four
train-track files and its existing scoring protocol.

| Metric, lower is better | Fable reference | Astra R1 | Equal joint | Scored support |
|---|---:|---:|---:|---|
| Methanol diameter CRPS, case mean, µm | 29.8681 | 29.9267 | 29.8979 | 182 cases, 26 sessions |
| Heptane diameter CRPS, case mean, µm | 46.1562 | 46.3022 | 46.2205 | 125 cases, 20 sessions |
| Normal ACME extinction IBS contribution, case mean | 0.051752 | 0.052725 | 0.052232 | 210 cases, 7 sessions |
| Normal ACME operator-termination IBS contribution | 0.027399 | 0.027804 | 0.027596 | same |
| Inverse ACME extinction IBS contribution, case mean | 0.027091 | 0.028194 | 0.027624 | 204 cases, 7 sessions |
| Inverse ACME operator-termination IBS contribution | 0.060816 | 0.061072 | 0.060939 | same |
| BASS tip CRPS, mean over runs, sample widths | 0.076101 | 0.080217 | 0.073439 | 12 cases, 3 spreading runs on one day |
| BASS track-ended Brier, mean over runs | 0.003392 | 0.003605 | 0.003206 | 59 observable pairs, only one ended pair |

Full values, release IDs, source-file hashes, exact executed commands, all session
differences and largest case changes are in the
[machine summary](../feedback/astra_train_summary.json). Session weighting differs from
case weighting; the table does not combine those estimands. The reference droplets/BASS
are Fable's neutral carry releases, whose current forward hashes were checked.

All three versions share three methanol domain refusals (a zero-pressure row), four
additional admitted but unscorable methanol cases, and nine heptane domain refusals
(unresolved gas-composition conflicts). ACME and BASS have no forecast refusals here.
The scorer's admitted-case accounting omits domain-refused forecasts, so the separate
refusal arrays are included in the summary and are not described as zero refusals.

### Session and mechanism limits

- **Methanol:** R1's equal-session CRPS difference is +0.0301 µm, joint +0.0156 µm.
  Only 10 of 26 sessions change beyond numerical noise: two improve, eight worsen.
  R1's largest case increase is +5.894 µm at `N06M101@5`. The 77 cases whose inputs admit
  a nitrogen-carrier closure worsen by +0.1385 µm for R1 and +0.0703 µm for the joint.
- **Heptane:** equal-session differences are +0.1274/+0.0559 µm. Four sessions improve,
  six worsen and ten are unchanged. `N04H101@1` worsens by +18.580 µm for R1. The 52
  nitrogen-eligible cases worsen by +0.3511/+0.1547 µm. These eligibility subsets do not
  identify which latent hot-radiative transition actually occurred; they are not a
  measurement of loss attributable solely to post-flame cooling.
- **Normal ACME:** extinction contribution worsens in all seven sessions for both new
  candidates. Small symmetric condition uncertainty did not improve this train target.
  **Inverse ACME:** four sessions improve and three worsen on extinction, but the net
  score worsens; `21228C2@5` adds +0.06013 to R1's extinction contribution. Neither
  pattern establishes sensor uncertainty, regime truth or transferable calibration.
- **BASS:** R1 improves T6 and worsens T5/T16; the joint improves T6/T16 and worsens T5.
  Three runs from one day cannot establish session transfer. The tip law improves at
  some leads and worsens at others; a steady balance has not replaced transient front
  dynamics. The very small event sample cannot establish an improved quench model.

The historical D-40 tolerance labels 25/26 and 19/20 sessions “no worse within 1 µm”
for the two R1 droplets despite their worse means. That tolerance is descriptive here,
not an adoption decision or evidence of equivalence.

## Outcome-free checks

The combined Rust suite exercises neutral identity, full-mixture behavior, event mass
and temporal coherence, physical limits, literal completeness/ancestry, and train-domain
selection using synthetic fixtures. Those fixtures never qualify a NASA model.

A separate [release-backed timestep receipt](../feedback/astra_thermal_convergence.json)
evaluates the authored 0.01 s step against one independently prescribed 0.005 s step:
both fuels; 2/3 mm initial diameter; 0.7/1/2 atm; 5/20/45 s after a hot-radiative ending.
All 36 cases succeed with no support-floor hit. Maximum diameter difference is
0.000842 µm and maximum temperature difference 0.000203 K. These are differences on
this fixed grid, not error bounds or empirical accuracy. No step was selected by a search.

Property extrapolation matters far more than that numerical difference. Along this grid,
methanol reaches about 264.95 K and heptane 283.00 K; permitted vapor/latent extensions
are used in many cases. The 0.7 atm starts are pressure-capped. Initial liquid temperature,
ambient-only heat input, constant liquid properties and absence of absorbed water remain
assumptions. The [independent property check](../feedback/astra_property_crosscheck.json)
was evaluated after the constants were typed; no model coefficient was replaced by its
output.

## Audit corrections and reproduction

F-008 motivated train-ID selection before domain deserialization. Mixed JSONL lines are
still streamed for identity: this is not filesystem isolation. F-009's colliding paired
summary keys are bypassed by pairing full release IDs. F-010's missing event case metrics
were fixed in the development exporter. Initial score files are preserved; a replay into
`runs/dev/astra/session_receipts/` has **exactly identical aggregate objects** for every
release. The replay verifies the export correction, not another candidate selection.

The first assembly attempt failed before patch creation because new exposure SourceIds
contained whitespace. The syntax was normalized without removing exposure or changing
a scientific number; the rejected metadata is retained locally under `preassembly/`.
The earlier formatter incident was corrected before release construction. Later Git
status noise was LF/CRLF conversion; committed old-source content remained unchanged.

Commands are in [09_reproduce.md](09_reproduce.md), section 9.9. Immutable source literals
and forwards were committed in `f061759`, after the initial research/design commit
`8e1e66f`. Assembly records, cards and results form the next checkpoint. No push or PR.

## What each priority became

| Pre-weight priority | Outcome |
|---|---|
| Prospective control plan and trustworthy readbacks | Deferred pending provenance/ingestion acceptance; no new shared record or input admitted |
| Persistent ACME condition uncertainty | Implemented once; negative train result, kept unchanged |
| Scoped coupled droplet cooling | Implemented once; stable fixed-step arithmetic, narrow scope and weakly negative train scores |
| Steady oxygen-supply spread | Implemented once; standalone negative, joint modestly positive on one train day |

No clean-room candidate or new outside-campaign score was performed in this checkpoint.
It therefore supplies no fresh generalisation estimate. The all-NASA extension now
requires clean-room first candidates, frozen groups before author outcome access,
separate blind measurement, and prospective registrations. Those are continuing work,
not accomplishments inferred from this train table. The unresolved X-23 access-identity
uncertainty and X-24 collector abstract exposure remain disclosed in page 07.
