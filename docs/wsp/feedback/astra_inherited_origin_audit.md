# Inherited literal origin audit

Author: **GPT-6 Astra (`gpt-6-astra`), explicitly selected by the user**. Scope: documentary audit of the five Fable predecessor configs, 2026-11-14. No weights authored or changed, releases materialized, fitting performed, dataset registry opened, or protected partition read. This report changes no existing card or release.

## Conclusion and counting rule

**The comments do not support an exact full-model count of inherited literals directly read from train. They do establish nonzero examples.** A zero in Astra's *new-position* ledger cannot be extended to the inherited model. The scientifically defensible full-model entry is **unknown, with the lower bounds below**, alongside a separate exact count for Astra's newly authored positions.

Here, a *position* is one named scalar tensor coordinate, counted once in a predecessor's final stored parameter layout. Repeated committee copies count as different positions but share one scientific origin. A *direct measurement-backed* position has a comment explicitly tying its value to an observed train value, an empirical summary, or a simple difference/sign convention between such values. Choosing to deploy that observation as a predictive constant remains an author decision; this category does not assert absence of judgment.

The conservative lower bound excludes coefficients merely supported by train residuals, manually chosen approximations to a law, and ambiguous mixtures of empirical summaries with uncertainty-policy choices. It is deliberately incomplete. If “directly read” is defined as **raw transcription only**, even medians and time differences must be reported separately; the table provides that distinction rather than hiding it.

| Fable predecessor | Direct raw-value transcription established | Additional explicit measurement-summary/difference positions | Conservative measurement-backed lower bound | Full inherited count |
|---|---:|---:|---:|---|
| FLEX methanol `wsp5/methanol/carry-d3` | Not established in this audit | Not included: mixed-origin floor discussed below | No positive bound asserted | **Unknown, not zero** |
| FLEX heptane `wsp5/heptane/carry-d2` | Not established in this audit | Not included: mixed-origin floor discussed below | No positive bound asserted | **Unknown, not zero** |
| ACME normal `wsp5/normal/p1-fable` | 4 positions carrying 98 s | 1 script-offset median | **At least 5** | Unknown |
| ACME inverse `wsp5/inverse/p1-fable` | 8 positions carrying 66 or 90 s | 1 script-offset median | **At least 9** | Unknown |
| BASS `front2/carry-d1` | None asserted | 1 difference of observed event times | **At least 1** | Unknown |

These are bounds for the **specified predecessor configs**, not automatically for a later Astra candidate. For a child, intersect the listed coordinates with its actually retained positions; replacement or removal can change the bound. No cross-model total is a count of independent observations: the same 0.4 s summary occurs in both ACME models, for example.

## Reproducible positive evidence

Paths below are repository-relative. All coordinate multiplicities were checked in the predecessor `edits.wsp` text; this is not a tensor-hash replay of immutable releases.

| Predecessor coordinates | Count | Literal and documentary origin | Evidence locator |
|---|---:|---|---|
| Normal `script.onset_s` | 1 | 0.4 s, sign convention for the stated median difference between the planned stage sum and recorded stop across 29 TRAIN tests | Normal P1 `header.wsp` lines 24–27; same statement at destination line 27 |
| Inverse `b0.ft.atom_t_s[3]`, `b1.ft.atom_t_s[3]`, `b2.ft.atom_t_s[3]`, `b3.ft.atom_t_s[3]` | 4 | 66 s: the ancestor explicitly says most observed stops in the 63–67 s window occurred at 66 s | `configs/models/acme/wsp3/inverse/d1-fable/edits.wsp` lines 102–107. Destination lines 76, 129, 178, 225 |
| Inverse `b0.ft.atom_t_s[5]`, `b1.ft.atom_t_s[5]`, `b2.ft.atom_t_s[5]`, `b3.ft.atom_t_s[5]` | 4 | 90 s: ancestor explicitly says the fifth session stopped two flames at 90 s | Same ancestor lines 105–107. Destination lines 78, 131, 180, 227 |
| Inverse `script.onset_s` | 1 | Same 0.4 s empirical script-offset median | Inverse P1 `header.wsp` lines 27–30; destination line 30 |
| BASS `end.linger_s` | 1 | 5 s = 99.5 s minus 94.5 s; comment gives the observed T5 flame-disappearance and tip-at-end times | `configs/models/bass/front2/carry-d1/edits.wsp` lines 79–80 |

The inverse lower bound does not count every integer in the 63–67 s support grid. Its comments combine an observed window with an authored probability law; 66 and 90 have especially explicit observed-value provenance. Likewise, the normal bound excludes 61.5, 126.5 and 275.5 s: they are representative nodes tied to observed time ranges, but the comments do not establish a unique direct-transcription or arithmetic rule for all three. This conservatism must not be mistaken for evidence that the excluded nodes were independent of train.

The no-stops variants disable atom probabilities and lower the background stop rate; they do **not** remove the stored atom times. Thus copied timestamps in `a1/b1` (normal) and `b2/b3` (inverse) remain historical train-derived positions even when their stop atoms are disabled. A count of influential parameters on a particular forecast would be a different, input-dependent quantity.

## Empirical summaries mixed with author judgment

These examples explain why an exact binary direct/non-direct classification would fabricate precision.

| Positions or block | What the comments establish | Honest classification |
|---|---|---|
| Methanol `unc.s_floor = 0.06` | Carry file lines 166–173 report robust train run-median sd 0.06 and then use 0.06 as a component no prefix removes, alongside a larger pre-prefix spread | Empirical numerical anchor **and** author judgment about uncertainty on new sessions; one identifiable mixed-origin position, excluded from the conservative bound |
| Heptane `unc.s_floor = 0.06` | Carry file lines 182–187 report realized run-level rate sd 0.06, then prescribe 0.10 before a prefix and 0.06 remaining after one | Same mixed origin; one identifiable position. Equality to train sd does not establish an irreducible noise floor |
| Normal `b0.bt.limit_w[0:2]`, `b1.bt.limit_w[0:2]` | `acme/wsp3/normal/d2-fable/edits.wsp` explicitly bases 0.15/0.85 on one of seven versus six of seven train sessions | Four stored positions from two complementary, author-coarsened cohort shares. Empirical-summary-based, but not literal 1/7 and 6/7, and not four independent choices. Separate from the strict lower bound |
| Inverse `b0.ft.log_base`, `b1.ft.log_base` | Ancestor D1 cites nine stops in 2300 flame-seconds next to -5.5 | Explicit empirical rate anchor with log transformation/rounding and event-definition judgment; not a directly read raw rate. No-stops slots replace it with -9.0, which expresses an assumed rare-event allowance |
| ACME `bt.tau_s = 30` | Normal growth-clock ancestor says 30 s was “read from” a sequence of burner temperatures and times | A response-law timescale inferred from multiple observations, not a directly observed 30 s event duration; count as train-informed closure unless a stricter historical derivation establishes more |
| Methanol radiative clock, heptane post-extinction law | Comments cite train medians, size trends, and window rates while choosing slopes, pivots and decay times | Train-informed author laws, not transcribed measurements. Good residual agreement does not change their origin |
| BASS speed, overhang and burnout laws | Comments use train speed groups and tip/burnout behavior to choose or revise closure coefficients | Train-informed judgment. The identical 0.325 in two overhang positions is a repeated representation of one chosen coefficient, not two measured constants |
| Inverse committee ratio 2:1 | Header says one third is the author's probability that a proposed law carries to unobserved conditions | Subjective scenario weight informed by train, **not** the empirical frequency of two observed surviving flames |

Under a broader explicitly declared “measurement-anchored, including mixed-origin” convention, the two FLEX floors each add one identified position, and the normal cohort weights add four. That is a **different classification**, not a correction to the strict lower bounds and not an exact full-model census. An N/A/L/S/U ledger should retain a mixed-origin note or an explicit precedence rule rather than silently call these wholly data-free or wholly measured.

## Structural values, copying and fixed constants

- Author-written zeros/ones that switch off newly added carry blocks are structural entries. They do not become observations because they were written after seeing train. Conversely, selecting which mechanism to disable is an adaptive modeling choice even if the literal itself is structural.
- A zero can also encode an empirical log-reference or a chosen probability; value alone cannot classify its provenance. Nor does an unchanged inherited number become structural merely because Astra copied it.
- ACME P1 is a delta against a baseline. Its header and `tools/wsp/committee_edits.py` explain that only changed member values are written. Counting assignment lines is therefore not counting the full stored model. Unwritten positions, silent baseline slots, and ancestor fills remain outside this partial attribution audit.
- Spec-fixed constants are a separate denominator from author-written literals. `crates/ignis-core/src/contracts/wsp.rs` defines `TensorFill::Fixed` / `TensorRole::Constant` and forbids patch writes to them. Do not include fixed feature scales, quadrature constants or unit conventions in a claim about author-written train constants merely because their numerals occur in the forward/spec. This audit does not assert a total number of spec-fixed positions.
- Repeated slots count repeatedly for storage-position totals and once per documented scientific source for source-origin totals. Neither total is an effective-degrees-of-freedom or generalization bound.

## Ledger wording and limits

Suggested card language:

> The new Astra block has the separately reported exact provenance count. The inherited Fable portion is data-informed; its full direct-measurement count is unknown. A source-comment audit establishes the predecessor-specific lower bound in `astra_inherited_origin_audit.md`, plus identified mixed-origin positions. Copying preserves historical provenance. Counts describe positions, not independent scientific choices or effective degrees of freedom.

Record any adjustments to a lower bound when a child overwrites the listed coordinates. Keep historical feedback exposure alongside the origin ledger: “not trained” and “literal” do not imply “no data/feedback used.” Some source comments describe earlier feedback exposure; this audit did not follow those references to outcome artifacts or use that historical performance to classify a value as direct train measurement.

Reading extent: five requested predecessor `meta.json` files; the three FLEX/BASS carry `edits.wsp` files; both ACME P1 headers and relevant destination coordinates; normal WSP2 D1/D2/D3 and D3-no-stops, normal WSP3 D1/D2 and D2-no-stops; inverse WSP3 D1/D5 and their no-stops edits; copying-helper source; contract definitions for fixed constants. This checks what the author **documented**, not the underlying NASA measurements, original extraction procedure, full release hashes, or the completeness of every historical baseline's attribution. No original measurement or historical performance claim was independently revalidated.

Reproduction requires only those text sources: expand the coordinate sets in the positive-evidence table, check their final literal statements, follow the named ancestor comments, and count each `(predecessor, tensor, index)` once. Do not run the copying helper, inspect dataset registries, or reopen evaluation artifacts to reproduce these bounds. A complete census would require a provenance annotation for each surviving historical scalar and an explicit mixed-origin policy; the existing prose does not supply one.
