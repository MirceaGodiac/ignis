# F-007: fabric endpoint rows are not generally first-event cumulative incidences

- **About:** new — event semantics of `bass_front.wsp2` and the initially copied `front3` path
- **Reviewer:** `gpt-6-astra`, 2026-11-14
- **Verdict:** refuted for the interpretation as competing first-event cumulative incidence;
  confirmed as an inherited construction in `front2.rs`
- **Severity:** model
- **Partitions touched while reviewing:** none. Only source and review instructions were read.
  No outcome, model weight or configuration was read or written for this finding.

## What the source says

`crates/ignis-wsp/src/front2.rs` notes, lines 323–326, define quenched/reached probabilities and rows
`active=(1-quenched)(1-reached)`, `extinction=quenched`, `reached_end=(1-quenched)reached`.
The artifact stores these in `event_cdf` (line 1439 at the inspected snapshot).
The new `front3.rs` initially retained this construction.

This finding does not refute the fact that the three values are nonnegative and sum to one.
It refutes their general interpretation as probabilities of mutually exclusive **first terminal
events by time t**, which requires each cause's cumulative incidence to be nondecreasing.

## What I did

1. Inspected the new `front3.rs` event construction during the requested physics review, without
   reading any experiment or forecast outcome.
2. Notified the parent and implementing worker before any candidate weights were authored.
3. At the parent's explicit follow-up request, read only the matching `front2.rs` source excerpts:
   lines 71–97, 316–342, 979–1005, 1164–1190 and 1405–1440, together with search matches locating
   the event fields. No predecessor source was modified.
4. Derived the counterexample below algebraically. No script, score or dynamic test was required
   or run for the counterexample. Synthetic probabilities below are not model parameters.

## Evidence

At inspected `front2.rs` lines 986–1004, the code computes

```text
q(t) = 1 - exp[-integrated quench hazard from cutoff]
e(t) = monotone envelope of P(uncut tip at/after end at t-linger).
```

At lines 1411–1414, the artifact assigns `F_end(t)=(1-q(t))e(t)`.
For a period after `e(t)=1`, this becomes `F_end(t)=1-q(t)` and decreases under any positive quench
hazard. For example, `q(t1)=0.1`, `q(t2)=0.2`, `e(t1)=e(t2)=1` gives `F_end(t1)=0.9` and
`F_end(t2)=0.8`. Probability that already reached the end has moved to extinction.

More generally,

```text
dF_end/dt = (1-q) de/dt - e dq/dt,
```

whose sign is not constrained by monotonicity of `q` and `e` individually. The running maximum of
`e` at lines 999–1004 does not fix it. Flow mixing at lines 1168–1184 preserves weighted means of
these rows; it does not repair member semantics. The same formulas were present in the initial
new `front3` snapshot (`p_q/p_end` around lines 933–951, artifact around 1402–1405).

An instantaneous occupancy model with precedence given to quenching could legitimately use these
probabilities, but it would not have the same meaning as a first-event CIF. The historical source's
words “quenched by t” and “reached the end by t” do not establish coherent first-event competition.

## What should change

1. **Preserve historical identity.** Do not edit `front2.rs`, existing releases, labels, partitions
   or their recorded results. Record the semantic limitation for historical event outputs.
2. **Repair the active new route.** Construct joint survival and monotone cause-specific increments
   on a fixed integration grid. Mass assigned to extinction or reaching the end is absorbing.
   Mix complete path-level CIFs over persistent law/flow scenarios.
3. **Keep the neutral compatibility route exact.** `front3` with the new mechanism disabled can
   reproduce `front2` exactly, while being explicitly identified as the compatibility behavior
   carrying this known limitation. Equality is not evidence that the event semantics are correct.
4. **Handle endpoint atoms explicitly.** The proposed repair can convert a monotone envelope of
   front-location probabilities into an assumed first-passage law. This is a new modeling closure,
   not a derived hitting-time distribution. A step to probability one is a right-end atom;
   finite quench hazard must act before the remaining mass is consumed at that atom.
5. **Do not mix incompatible meanings in a new joint CIF.** Averaging an exact old occupancy row
   with a corrected physical-member CIF need not be monotone and has no single first-event meaning.
   Preferred active-joint design: apply the coherent event mechanism to both the legacy-speed
   scenario and the oxygen-speed scenario, and document that the former's event interpretation is
   new. The zero-member-weight compatibility branch can remain exact. An alternative, only if the
   existing output contract permits it, is a positions-only committee that withholds joint event
   CIFs. A warning does not turn a nonmonotone row into a valid new CIF.

Checks for the implementation: monotone per-cause incidence, survival plus cause incidences equal
one, no event-mass reassignment after reaching the end, finite-hazard mass before an endpoint atom,
mixture identity, and query-horizon independence on the fixed internal grid. Neutral predecessor
identity is a separate check. These need synthetic cases only; no validation look is justified.

The author/worker owns implementation and candidate authoring. This reviewer supplies no weights.
The thermal conditional-floor concern remains in the separate
[`astra_forward_physics_review.md`](../../research/astra_forward_physics_review.md) note and is not
bundled into this finding.

## Response

Pending author response. The implementing worker has acknowledged the issue and proposed a fixed-grid
first-event construction. This finding records neither a completed repair nor a passing test; append
the actual changes and observed verification after review.
