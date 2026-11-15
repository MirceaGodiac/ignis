# Blind measurement design for NASA combustion archives

## 1. Reading boundary and the immediate gate

No local experimental tables, `runs/dev/data_all/`, old model implementations/releases/results,
old scientific dossiers, media, per-run validation, test, or holdout were opened. Neither the
NASA TRAIN references mentioned below nor the split files were opened. The assignment states
that the new-investigation split files are not yet present; this document does not claim a
fresh filesystem verification of that state.

**Boundary incident:** a methods-targeted web search returned unsolicited result-bearing
snippets associated with *NASA/TM-20210011385, BASS-II final report* (2021), including an
unidentified per-test passage in a search result. The report was not opened. Its canonical
source is [NASA/TM-20210011385](https://ntrs.nasa.gov/api/citations/20210011385/downloads/TM-20210011385.pdf).
The unsolicited per-test search excerpt appeared on the corresponding
[ResearchGate report entry](https://www.researchgate.net/publication/355545976_TM-20210011385_BASS-II_final_report);
that entry was also not opened.
No result, run identity, or snippet is reproduced here or used to choose a method, threshold,
or uncertainty. The parent coordinator was informed. This designer is therefore not eligible
to act as a clean-room model author or a fresh blind measurer. Subsequent research was
restricted to generic instrumentation, uncertainty, and image-analysis methods. The source
ledger below distinguishes excerpts from complete reads; it is not a claim to have read all
NASA papers.

Before any new-investigation outcome reaches a model author, an independent custodian must verify
and pin `docs/wsp/atlas/splits/PSI-<id>.json`, its digest, and its test-id/test-day-only provenance,
and prove that every released per-run outcome belongs to TRAIN. Missing, ambiguous, changed,
or outcome-derived membership blocks **author release**, not independently authorized acquisition
or protected work by fresh non-modelling agents. Such agents may acquire, measure or label into
the designated protected paths without reporting outcomes to an author or the main coordinator;
unresolved group membership remains pending until the custodian joins the frozen split. They
must not invent groups, qualify results against an unresolved partition or grant themselves
access to an existing test partition. No result may be used to repair a group definition.
Media duplicates, alternate camera views,
reprocessed copies, and a run's sensor streams must resolve to the same frozen test/group.
Calibration shared across groups is tracked as shared uncertainty; it does not justify
regrouping after outcomes. Existing protected partitions remain protected.

## 2. Roles and the two identity layers

**Owner requirements:** separate modelling from measurement/labelling; use fresh contexts with
strictly scoped input manifests; keep private routing and protected outputs in designated
folders that authors never open; send no protected outcome back to the main coordinator or an
author. The authorized operational arrangement uses fresh `fork_turns="none"` non-modelling
agents on the shared filesystem. Their instructions and input/output allowlists enforce role
discipline: no inherited model context, no repository-wide discovery, no browsing outside the
approved material, no reads of routing/peer outputs, and no outcome-bearing final messages or
tool logs returned to authors. The custodian selects explicit paths rather than asking an
author to inspect protected directories. Completion receipts contain only authorized workflow
state and artifact identities that do not reveal outcomes; even aggregate QC stays protected.

This is **discipline-enforced isolation, not OS-enforced security**. A directory named `sealed`
does not make shared files technically inaccessible. Record that limitation plainly. It does
not block the explicitly authorized acquisition or isolated labelling/measurement workflow.
No new OS account, evaluator service or storage service is required before that work can begin.
Host-derived `ExecutionScope` remains mandatory wherever the existing services use it; a prompt
or request body cannot grant a partition privilege.

**Stronger recommended enforcement:** an evaluator-controlled account/process/store with actual
filesystem/service access checks, no author-readable indices, and dependency-aware artifact
permissions would reduce accidental disclosure. Adopt it when available as hardening, not as an
invented owner prerequisite. Do not implement another agent loop to obtain separation. In the
rest of this document, "protected store", "private" and "custodian-only" refer to the designated
operational paths/roles under the active arrangement unless OS enforcement is explicitly stated.

| Role | May see | Must not see |
|---|---|---|
| Routing custodian | Source catalogue, exact source IDs/paths, frozen groups/partitions, clock and calibration provenance; minimum bytes needed to redact inputs | Model predictions or author feedback used to select cases |
| Instrument custodian | Calibration records, channel manuals, acquisition settings, blank/reference-target images | Model hypotheses, expected experimental outputs, reference comparison residuals |
| Blind measurer A | Opaque package, measurement instructions, instrument-only metadata, approved pixels/traces | Original condition filenames, run/day/group/partition, NASA outcome reference, model, hypothesis, expected answer |
| Blind measurer B | Separately aliased equivalent package and same method version | A's annotation, errors, confidence, selection reason, all other prohibited context |
| Comparator/evaluator | Sealed A/B outputs, identity map, reference series for permitted TRAIN groups, pinned method and comparison protocol | Permission to edit measurements to match the reference |
| Scientific model author | Released TRAIN measurement artifacts only; permitted registered validation aggregates later | Protected per-run outputs/media, method-debug residuals from protected groups, unregistered aggregates |

**Routing identity** stores the NASA accession, exact original object URL/path/member name,
test id, source checksum, frozen group id, partition, calibration provenance and dependency
graph. It never enters a measurer prompt, filename, tool response, log, thumbnail or error.
Use unpredictable opaque aliases, not a public hash of an enumerable run id. Keep the alias
mapping and any HMAC key private to the custodian. Distinct aliases prevent B from recognizing
A's package; the custodian alone joins them afterward.

**Blinded input** exposes only what defines a reproducible observation:

- Channel modality (backlit silhouette, visible, filtered emission, PMT, radiometer, aerosol
  counter), channel axis/polarity, pixel format/bit depth, exposure, gain, binning, clipping codes,
  pixel aspect ratio, lens/window corrections, native sampling timestamps and clock quality.
- Instrument geometry needed to measure: coordinate axes, specimen plane/outline, camera view,
  physical fiducial lengths and uncertainty, nozzle or support masks, calibration transforms,
  spectral band, detector transfer function and validity range, integration/response time,
  sample flow/dilution and transport delay when relevant. Supply immutable calibration artifacts
  under opaque IDs and a shared-calibration identifier without revealing run dates.
- The observable definition, frozen preprocessing recipe, valid field of view, fixed failure
  rules, time-origin definition, and objective blank/reference acquisition windows. Supply
  instrument/event markers such as shutter state or igniter power only when required to exclude
  a known optical contaminant; identify them as instrument states, not outcome labels.
- Raw acquisition-relative time is retained. Absolute date, file basename, fuel/material name,
  oxygen fraction, intended burn duration, nominal expected size and experimental partition are
  withheld unless a particular item is indispensable to an instrument correction. For such an
  exception, the instrument custodian supplies the minimal verified correction with provenance
  and uncertainty, without an expected result. If that cannot be done honestly, mark the
  observable unsupported; do not invent a universal correction.

Remove container titles, embedded subtitles, commentary audio, spreadsheet captions, archive
member names, and condition/outcome overlays before release to the measurer. Retain a protected
redaction map and original digest. A time overlay may be transcribed by the custodian into an
acquisition clock, retaining its precision; do not erase scientifically necessary time. An
overlay covering the measuring region makes that region unavailable rather than permission to
inpaint it. Never apply image-generation, super-resolution, or content-aware fill to evidence.

Blinding cannot remove the physics visible in an image, nor guarantee that a person/LLM has no
prior knowledge. Each measurer declares prior familiarity with these runs/publications; known
exposure disqualifies it for that package. Use a fresh context, no inherited chat, repository
search, browsing, condition tables or access to the routing store. Archive this declaration,
prompt hash, tool allowlist and role identity. This entire design document contains model-task
context and is **not** the prompt sent to the eventual measurer; generate a minimal instrument
and method-only instruction packet from its approved method card.

## 3. Freeze a measurement method before experimental reference comparison

A method card has a version and content digest and contains the following independently
reviewable items:

1. Precisely named observable, units, geometric definition and sensor bandwidth; input channel
   and acquisition requirements; instrument-calibration versions and their validity criteria.
2. Exact pixel/signal transforms, masks, boundary rules, time handling, constants with their
   instrument/noise rationale, manual annotation rules and allowed interventions. No unspecified
   "choose a sensible threshold" step is executable.
3. Noise/blank-based detection criterion, false-positive allocation, persistence duration tied
   to response/sampling, censoring rules, handling of dropped/clipped/occluded samples, and
   segmentation ambiguity. Values still awaiting an instrument manual are explicitly unresolved
   requirements; that card cannot process real measurements until filled and frozen.
4. Calibration and per-sample uncertainty model, covariance factors, reproducible derived
   quantities, evidence selection rule, and relevant failures/out-of-domain conditions.
5. Tool/code identity, dependency versions, decoder settings, prompts, approved fixtures,
   predeclared reference comparison and A/B disagreement criteria, qualification state.

Use dimensional targets, blank/dark traces and synthetic fixtures for development. Synthetic
fixtures remain `synthetic_fixture`; passing them establishes mechanics, not NASA accuracy.
Any instrument calibration estimation must use independent calibration artifacts and belong
to the measurement pipeline, never a physical-model forward. Prefer published/certified
calibration coefficients. No fitting or searching a tracker against experimental NASA truth,
no optimized burn-time threshold, and no fitted physical trajectory substitute for pixels.

Adaptive operations are allowed only as a frozen instrument rule, such as estimating a dark
offset from a specified shutter-closed window. They cannot choose parameters from the observed
burn duration, apparent extinction, a model prediction, or agreement with the NASA series.
Do not normalize each fading frame by its own maximum. A method change produces a new immutable
version and a remeasurement lineage; it never overwrites an unsuccessful version.

## 4. Concrete extraction methods

These are proposed procedures, not measurements. NASA's Spotlight manual supplies established
image-analysis primitives: explicit areas of interest, manual point tracking, threshold-edge
tracking, center tracking and contour recording. It supports saving a processing sequence.
Use these ideas with a pinned modern decoder/implementation; this protocol does not require
installing the historical GUI. The procedure choices below are design proposals, not claims
that NASA used this exact algorithm. [Spotlight-8, printed pp. 3–7](https://ntrs.nasa.gov/api/citations/20060011194/downloads/20060011194.pdf)

### 4.1 Common image and clock handling

Decode original frames without rescaling, display interpolation or gamma changes to the numeric
array. Retain stream index, frame index, native PTS/timebase, exposure duration and duplicate/
dropped-frame flags. Use acquisition timestamps or a documented clock map; file modification
time and nominal frame rate alone do not establish experimental time. Clock reversals, unknown
gaps or a lost origin yield explicit invalid segments. Container PTS denotes presentation time,
so verify how it maps to exposure rather than presuming equality.

Prefer calibration through the same optical path, focal configuration, binning and object
plane. Use source-provided lens correction and a planar mapping only for a documented plane.
A single scale bar does not establish full perspective correction; a solid's planar mapping
does not calibrate the three-dimensional flame above it. Retain apparent projected quantities
when depth is unobservable. Calibration uncertainty includes target dimensions, point placement,
lens/window effects, pose/depth and changes of focus/zoom. Multiview planar calibration and
radial distortion have a standard basis, but its required calibration images cannot be
invented from a flame. [Zhang, camera-calibration paper](https://robots.stanford.edu/cs223b04/JeanYvesCalib/papers/zhan99.pdf)

If no dimensional calibration exists, emit pixel-only observations with no fabricated SI
distance. If no trustworthy clock exists, emit frame-index observations with no fabricated
seconds. Such observations are useful evidence but cannot qualify a metric forecast.

### 4.2 Droplet and flame diameters, drift and soot-shell geometry

NASA's FLEX apparatus description distinguishes backlit liquid imaging from filtered emission
imaging of the flame; the channels must not be interchanged. Only the diagnostic description,
not the report's experimental results, informs this distinction.
[FLEX apparatus description, NASA/TP-2015-216046](https://ntrs.nasa.gov/api/citations/20150023456/downloads/20150023456.pdf)

1. On the backlit channel, apply the approved dark/flat correction, mask supports/needles using
   instrument geometry, and locate the liquid silhouette in the prescribed field. Record all
   ambiguous candidates rather than selecting the one closest to a predicted size/position.
2. Obtain a boundary from the frozen contrast/edge rule. Record the subpixel rule if used;
   fractional coordinates do not by themselves imply subpixel accuracy. Preserve the contour,
   local edge-width information, excluded arcs and a raw-frame overlay for audit.
3. Report projected area-equivalent diameter `D_A = 2 sqrt(A/pi)` and horizontal/vertical
   extents separately. If reporting a circle-based diameter, identify the estimator and its
   shape assumption as a distinct observable. Do not force spherical symmetry when the outline
   deforms; missing occluded perimeter is not silently reconstructed. A partial silhouette can
   give a bound or an explicitly incomplete contour, not a complete area.
4. Record centroid and its displacement in the calibrated view. Preserve camera motion and
   fiducial registration uncertainty; do not label projected centroid motion a 3-D velocity.
5. For a luminous/OH-band flame, record a channel-specific emission-contour diameter, with
   inner/outer contour definition, fixed absolute/noise-referenced detection criterion and
   saturation flags. It is an optical observable, not automatically the reaction-sheet radius.
   Visible, OH-band and infrared radii are separate quantities. An annulus may require reporting
   inner and outer area/extent; do not collapse both into an unnamed "flame diameter".
6. Soot-shell projected position/thickness is a separate contrast observable. Brightness does
   not become soot volume fraction without the appropriate calibrated extinction/emission
   measurement and its assumptions. Near disappearance, emit a detection bound or
   `below_detection`, never a zero-sized droplet/flame by default.

Carry pixel boundary ambiguity, shared scale, optical blur, saturation, depth/shape assumptions
and registration uncertainty. Derived `D²` is a separate computation artifact with propagated
uncertainty; do not smooth/fill it to satisfy a burning law. Multi-droplet overlaps and breakup
retain component identity and an ambiguity state; no model-based association through gaps.

### 4.3 Flame fronts and material fronts in solids

Define the specimen coordinate axis and its origin from geometry, with its direction fixed
before watching combustion. Track **different** observables: leading luminous tip, luminous
base, material regression/char/pyrolysis boundary when distinguishable, and specimen edge.
Flame length is tip minus base along the stated projection; it is not automatically burn
length. Label front definitions and channels; retain the transverse contour when a single
position would hide curvature or multiple fronts.

For each native frame, apply the frozen detection rule in the geometrically valid field and
record contour/point plus local ambiguity. A manual operator places the defined boundary using
the raw/approved display, without a model curve or prior operator trace. Do not force monotonic
motion. Wrinkling, glowing material, reflections, igniter light and detached flame fragments
are recorded as alternative components/flags, not resolved by expected spread behavior.

Convert positions with the valid specimen-plane calibration. Report flame positions as
projected when height above the plane causes unresolved parallax. Calculate interval spread
speed as `(x(t_b)-x(t_a))/(t_b-t_a)`, with the interval chosen by a frozen acquisition/time-grid
rule and published alongside the result. Do not select the visually straightest interval or
fit a spread rate to a model. Derivative estimates require their own resolution and uncertainty;
positions remain primary. Preserve gaps and separate intervals across instrument changes.

Quench/blow-off are event classifications only under frozen operational definitions. Give
last-present/first-absent brackets, unresolved/sub-threshold cases and reacquisition events.
Loss of the image or flame leaving the field is censored, not evidence of extinction. Saffire's
long videos use the same rules and a continuous clock across chunks; BASS-specific material
calibration is not transferred merely because both experiments burn solids.

### 4.4 PMT/radiometer optical-flame end and onset

Manufacturer documentation identifies dark current, gain/temperature dependence, saturation
and electronics-dependent response as measurement concerns; it does not identify the particular
NASA sensor or certify its calibration. Obtain the actual channel manual and acquisition
configuration before activating a card. [Hamamatsu PMT technical guide](https://www.hamamatsu.com/content/dam/hamamatsu-photonics/sites/documents/99_SALES_LIBRARY/etd/Highsensitivity_PMTModules_TPMO1120E.pdf)

1. Parse raw sample values with channel identity, ADC clipping codes, hardware integration time,
   gain, timestamps, trigger map and instrument event markers. Keep raw voltages/counts even
   when a calibrated radiometric quantity can be computed. Do not convert volts to temperature
   or total radiant power without the required spectral/geometric calibration.
2. Determine offset/noise only from specified independent dark/blank acquisitions or a
   predeclared shutter/igniter-off baseline window. If baseline contamination cannot be excluded,
   fail the event measurement. Correct documented drift only by the frozen rule.
3. Define detection relative to the blank distribution and ADC/response limits. The card must
   state its false-alarm criterion, noise dependence/autocorrelation treatment, hysteresis if
   used and persistence interval. Choose these from instrument response and intended observable
   resolution, not the NASA burn-time table. Missing numeric instrument parameters block the
   card; there is no universal threshold supplied here.
4. Identify each detected interval and store threshold crossings. End is bracketed between the
   final qualifying present sample and the first sample of a sustained below-threshold interval;
   retain the later confirmation time separately. Onset uses the corresponding rising rule.
   Reignition produces separate intervals. No confirmed decline before acquisition ends means
   right censoring. Gap crossings and recovery from saturation are unresolved unless the
   method's documented instrument behavior supports a bound.
5. Subtract calibrated clock/channel delays with uncertainty, without aligning signals to get
   the desired burn time. PMT, radiometer and visible-camera ends remain separate. A radiometer
   may include hot surfaces/soot/igniter radiation after a flame channel disappears. Disagreement
   is an observation, not permission to choose whichever matches NASA.

The measured endpoint is **end of detectable emission in a stated channel**, not universal
chemical extinction. Instrument response, exposure/integration, dark drift, threshold
uncertainty and synchronization contribute to timing uncertainty. If the response delay is
unknown, retain an interval or unknown systematic term rather than a precise corrected time.
Only an independently defined comparable NASA endpoint may be used for method qualification.

### 4.5 Smoke/aerosol particle-size moments

The SAME/SAME-R method descriptions distinguish number, diameter and mass concentration and
make distribution reconstruction conditional on assumptions. A manufacturer P-Trak manual
reports number concentration and instrument averaging; it is not a full size-distribution
measurement. Preserve the actual flight instrument variant and calibration, rather than
substituting a current product specification. [NASA SAME-R methods summary](https://www.nasa.gov/wp-content/uploads/2025/07/smoke-aerosol-measurement-experiment-same.pdf?emrc=b828be),
[TSI P-Trak manual, Operation p. 21](https://tsi.com/getmedia/d1f26f38-5f1d-4851-80a2-b99d95c7c9c8/Model-8525-P-Trak-Op_Svc_Mnl_1980380?ext=.pdf)

Use unnormalized number-distribution moments `M_k = integral d^k n(d) dd`. With diameter in
metres and concentration per cubic metre, `M_k` has units `m^(k-3)`. Keep normalized averages
as distinct quantities: number-mean diameter `M1/M0`, and volume-mean particle diameter
`(M3/M0)^(1/3)`. The first moment itself is not the mean diameter.

For each channel, retain raw readings and implement only its documented calibration: counting
efficiency/size acceptance, flow and dilution, zero correction, coincidence/saturation,
integration time, tubing losses and transport delay. An ionization or scattering channel is
not a moment merely because its label suggests one; the instrument response and assumptions
must justify the conversion. Instrument-reported mass is stored as reported mass. Converting
mass concentration to `M3` requires justified particle density/shape (`mass = rho*pi*M3/6`
only for the applicable spherical/density interpretation). Unknown density or optical response
leaves the derived moment unavailable, not filled from a model.

Combine moments only over matched sample volumes and timing windows after independently
calibrated transport-delay correction. Retain sensor bandwidth/size-window differences. Where
NASA actually reports a calibrated moment, ingest it as an instrument-reported observation
with its calibration/source locator; do not reverse-engineer unprovided moments from a plot.

Optional lognormal reconstruction is a **derived assumption-bearing artifact**, never a direct
measurement. With compatible positive moments,
`sigma_log² = ln(M3*M0²/M1³)/3` and
`D_geometric = (M1/M0)*exp(-sigma_log²/2)`.
A negative inferred variance, inconsistent channels, detection limits, unknown calibration or
non-spherical aggregates means the reconstruction is unsupported; do not clip it into a valid
distribution. Three moments do not establish that the distribution is lognormal. Retain
ratios/covariance, dilution and common-volume calibration correlations. DAFT/DAFT-2 does not
inherit a SAME moment set unless its own channel provenance supports it.

## 5. Uncertainty belongs to every value and to shared calibrations

Report standard uncertainty, uncertainty components and assumptions; expanded intervals must
give coverage factor and intended coverage. The covariance form of propagation is
`Cov(y) = J Cov(z) J^T` for a suitable local linearization. Include calibration parameters as
inputs, not only per-frame edge noise. Where linearization is unsuitable (threshold crossing,
censoring, missing contour), retain bounded/set-valued results or a justified propagation
method instead of an invented Gaussian standard deviation.
[NIST TN 1297, Appendix A](https://www.nist.gov/pml/nist-technical-note-1297/nist-tn-1297-appendix-law-propagation-uncertainty)

The implementation should represent covariance sparsely: sample-local terms plus shared
parameter blocks and each observation's sensitivity vector. This supports long series without
storing a dense all-frame covariance matrix. For `D_i = s*p_i`, a shared scale contributes
`Cov(D_i,D_j) = p_i*p_j*u(s)^2`; common timing offset contributes to absolute event times but
can cancel in a duration. Shared calibration can correlate different tests and different
instruments. Do not reduce those terms by `sqrt(number_of_frames)` or count every frame as an
independent replicate. Preserve edge/blur temporal correlation and shared reference uncertainty.

Per-value uncertainty must reflect that frame's contrast, visibility, clipping and calibration
applicability. A single measured method RMSE copied onto every value is inadequate. The
independent comparison stage may establish a method-domain discrepancy component, with its
sampling uncertainty and limitations, but must avoid counting the same calibration uncertainty
twice. A/B agreement does not identify a shared calibration bias. No available estimate is
reported as `unknown`/`provisional`; it is never silently zero. Non-observed values carry a
reason and bounds where justified. Measurement uncertainty stays separate from physical-model
predictive uncertainty and from variability among experiments.

## 6. Independent qualification on NASA TRAIN reference series

The routing custodian, not the measurer/designer, resolves the following candidate references
against frozen TRAIN membership. Their existence/content is **not verified by this document**:

| Proposed method | Candidate reference identified by the assignment | Required compatibility check |
|---|---|---|
| Liquid diameter | FLEX processed TRAIN diameter series; CFI Igor series only after split freeze and TRAIN filtering | Same liquid diameter definition, units, clock origin, channel and documented uncertainty |
| Solid front | TRAIN tracks under `tools/wsp/bass_optical/tracks/train/` | Same luminous/material boundary, projection, calibration and timestamp convention |
| Optical flame end | CFI-G TRAIN table burn times | Table start/end definitions genuinely match the channel endpoint; administrative cutoffs distinguished |
| Aerosol moments | Authorized TRAIN instrument-reported moments plus independent calibration/reference measurements, where available | Same moment convention, size window, dilution, density/shape assumptions and temporal averaging |

NASA values are references with their own uncertainty, not infallible truth. If a reference was
computed from the same image/calibration, agreement checks extraction consistency rather than
independent absolute accuracy. Record shared-error covariance and this limit. If no compatible
reference exists, the method remains unqualified for that endpoint/domain; do not borrow an
accuracy number from another apparatus.

Operational sequence:

1. Custodian pins a reference manifest and reserves method-development and qualification subsets
   **within TRAIN**, by frozen group/source identity before consulting outcome values. This does
   not alter the scientific train/validation/test split. All grouping uses pre-outcome metadata.
2. Freeze the method, eligibility rules, comparator code, matching/time-grid rule, exclusions,
   metrics and adequacy tolerance before reference access. Tolerances come from an instrument
   requirement or intended measurement resolution documented in advance; an unavailable
   justification yields a descriptive error report, not a pass/fail qualification claim.
3. Fresh measurers submit sealed outputs and evidence. Only then may the independent comparator
   join NASA reference values. Compare directly at compatible timestamps, or use a specified
   interpolation rule inside observed overlap with its uncertainty. No dynamic time warping,
   optimized lag/scale, selective interval picking, or extrapolation into gaps.
4. Report signed bias, absolute error distribution, RMSE where defined, interval coverage,
   event-bracket compatibility, missing/failed fraction and reason, per-run weighting and
   uncertainty of those statistics. Compute at run/group level to respect temporal dependence.
   Do not discard difficult runs or choose the best metric after seeing the comparison.
5. A method developer may receive registered TRAIN comparison feedback only after outputs and
   the card are frozen. The measurer is retired from blindness for any truth revealed. A revised
   method gets a new version; already revealed qualification cases become development evidence,
   and its next qualification uses still-unrevealed reserved TRAIN cases. If those are exhausted,
   report the limitation; never replace them with validation/test or relabel old cases as fresh.
6. The evaluator signs a qualification artifact with its domain, measured errors, limitations,
   all denominators and references. At present no such artifact exists from this task.

Method qualification on TRAIN is separate from model evaluation. Before any protected aggregate
is visible to an author, the releases/endpoints/cohort/metric/baseline/interpretation must be
preregistered through the repository's evaluation process. That includes seemingly benign
aggregate durations, sample counts by outcome, tracker failures and uncertainty summaries that
could guide a model. Protected per-run QC stays with the evaluator. A test-scoring reveal is
not authorized by this measurement protocol; the final test remains the one frozen evaluator
run with its reveal log. Neither a designer nor an author reads protected aggregates to tune a
method, decide coverage or choose the next model family.

## 7. Deterministic independent repeat of one in ten tests

Pin the complete eligible test inventory from source IDs before outcome inspection. Within an
investigation, the custodian deterministically orders tests by keyed hash of the opaque routing
identity with a committed selection-key digest; ties use the routing identity privately.
Select positions 1, 11, 21, ... for B. This is one per complete ten-test block, plus one in a
final incomplete block; thus small inventories also receive a repeat and the rounding is
explicit. The selection algorithm/version and complete routing-level selection manifest are
sealed. Later discoveries create a new manifest extension with its own fixed blocks; they do
not reorder prior assignments or remove difficult repeats. Selection never depends on quality,
measured value, condition filename, partition, model residual or A's confidence.

B gets an independently aliased package, identical input bytes/calibration and method version,
but no A marks, operator notes, evidence selection, confidence or statement that the case is
a repeat. B performs a fresh measurement; simply rerunning a deterministic script with A's
manual inputs is a reproducibility check, not an independent measurement. The comparator alone
joins A/B results. Do not treat common calibration as independent when assessing differences.

Preserve A, B and their uncertainty. Report differences in observables, interval overlap and
classification disagreement using predeclared criteria. A third blinded adjudication, if
needed, is another immutable record; any curator decision cites all versions and its rule.
Never average away disagreements or silently replace A. Additional objectively flagged reviews
are allowed under a frozen quality rule, but are labelled separately from the one-in-ten
sample and do not replace it in repeatability statistics.

## 8. Typed record shape for implementation review

This is a design-level field inventory, **not** a new executable Rust/TypeScript/JSON schema.
Integrate through existing `ArtifactRef`, `EvidenceRef`, `DatasetBundle`, `VisualObservationBundle`
and related contracts after their owners review the concrete producer/consumer changes. Any
cross-lane type must be defined in `ignis-core` with fixtures, validation and generated outputs;
this task changes none of them. Use closed enums, typed IDs, finite numeric values, explicit
units and `deny_unknown_fields` when implemented.

| Record | Typed contents and invariants |
|---|---|
| Protected routing record | `SourceId`, `TestId`, `GroupId`, split digest, protected partition, source URL/member locator, source SHA-256, byte length, calibration dependency IDs, alias mapping, access ledger reference. Group/partition never copied into blind input. |
| Blind input manifest | Opaque `MeasurementJobId`, alias, input artifact references/digests, channel/instrument enums, calibration artifact references, acquisition clock/time-origin descriptor, frozen method digest, redaction transform reference and approved geometry. No arbitrary free text from source filenames or experiment tables. |
| Blind measurement submission | Job alias, opaque measurer identity/exposure declaration, method/tool/prompt/config digests, input digest, timestamp in RFC 3339 UTC, series/contour/evidence artifact references, explicit completion state and coverage intervals, annotations/interventions. No source-derived test/group ID. |
| Routed per-test measurement | Custodian joins the submission to `TestId`, `GroupId`, split digest and protected visibility; retains each A/B version, acquisition source locators and immutable provenance. One per-test manifest references the full numeric series, without exposing protected contents to authors. |
| Numeric sample | Typed observable/channel, original value/unit (or absence reason), SI value only if calibrated, native sample/frame locator, acquisition time with origin/uncertainty, quality status, uncertainty descriptor, shared-factor sensitivities, evidence reference. All finite; bounds ordered; no zero substitution. |
| Event observation | Observable definition, onset/end/interval index, last-present/first-absent times and confirmation time, detection rule digest, `observed / left_censored / right_censored / interval_censored / ambiguous / unavailable`, uncertainty and evidence references. A censoring bound is not an exact event. |
| Uncertainty descriptor | `known / provisional / unknown`; local standard uncertainty or justified bounds, unit, component sources, distribution/coverage if used, covariance-block IDs and sensitivities, calibration-domain status. Zero requires an actual mathematical reason, not missing metadata. |
| Calibration block | Opaque calibration ID, instrument/configuration validity, coefficients and units, source artifact and locator, covariance or explicit missing terms, correction/transform version, common-factor identifiers; original identity stays protected. |
| Evidence item | Content digest, raw-frame/sample locator, decoded-plane digest, PTS, ROI/mask/contour annotation, approved display-transform digest and legend, original source mapping in protected store. Rendered overlay and unaltered evidence pixels are distinct artifacts. |
| Qualification / A-B comparison | Method version, sealed eligibility/reference manifests, comparison protocol digest, group-aware statistics, mismatch records, uncertainties, all exclusions and failures, evaluator identity and release authorization. No hidden measurement mutation. |
| Fetch/deletion receipt | Source digest/URL, actual bytes, allocation and budget reservation, UTC timestamps, download verification, kept artifact references/digests, both measurement completion receipts where selected, protected-routing authorization, exact disposable path and deletion verification. |

Suggested sample quality states are `observed`, `pixel_only`, `below_detection`, `saturated`,
`occluded`, `out_of_field`, `missing_samples`, `ambiguous`, `calibration_invalid`, and
`not_applicable`; simultaneous flags may supplement a primary state. Do not conflate missing
with not applicable or censoring with zero. Lists/series stream to referenced Parquet or
appropriate binary contour artifacts; manifests carry references, not whole media payloads.
The access label follows every derived artifact, thumbnail, cache, log and comparison.

## 9. Sequential acquisition under a strict storage budget

The coordinator reported about 119 GiB free on D:; that is contextual, not a measurement made
here and not a reservation. Keep the **entire task's simultaneous allocated footprint below
40,000,000,000 bytes**, including retained outputs/calibration/documents, partial downloads,
compressed archive, extracted members, decoder caches, evidence, logs and copies on any volume.
Using decimal GB deliberately avoids confusing 40 GB with 40 GiB. Available D: space is checked
again before and during each job because other workers share the disk.

Use one acquisition/measurement job at a time. For each planned object, reserve
`retained_bytes + download_peak + extraction_peak + decode_peak + outputs_peak + safety_margin`
strictly below both the task cap and currently allocatable free space after the host's reserve
and other jobs' reservations. Estimate from actual metadata/decoder behavior, then enforce a
live byte budget; unknown expansion is not zero. The margin is tied to measured buffers/files
and host requirements, not a new arbitrary sample ceiling.

1. **Plan.** Custodian resolves the exact source and members, split/visibility and calibration.
   Dry-run the repository fetch command with the pinned selection; record advertised size.
   Network acquisition uses the explicit `--execute` path only when the job is authorized.
   The measurer never receives a condition-bearing URL or fetch log.
2. **Download and verify.** Stream to an isolated task-specific temporary path, meter actual
   bytes and free space, hash the bytes, verify expected length/digest where available, and log
   missing publisher checksum honestly. Resume only with a verified object identity. A changed
   remote object is a new source version, never a silent continuation.
3. **Decode incrementally.** Inspect archive headers without expanding everything. Reject path
   traversal, absolute paths, symlinks/reparse entries and unexpected expansion. Stream a member
   or extract only the current required member, retaining frame order and a bounded decoder
   buffer. Do not create a whole-video PNG directory. For nonseekable archives, count all bytes
   transferred even if only selected members are kept.
4. **Measure and double.** A submits the complete series and method/evidence artifacts. When
   selected, B measures independently before deletion; if secure isolation requires a later
   refetch, pin and verify exactly the same source digest. Long videos are checkpointed at
   acquisition-time boundaries with overlap only as required by the instrument response; remove
   duplicate overlap samples deterministically. No frame skip is justified solely by disk size.
5. **Verify retention.** Validate records, units, time coverage/gaps, checksums, evidence links,
   calibration dependencies and A/B completion. Replay selected checkpoints from source pixels
   with the pinned decoder; distinguish this from accuracy qualification. Keep a compact raw
   evidence set chosen by the frozen rule: calibration/blank, representative acquisition-time
   locations, event brackets and ambiguous/failing cases. Evidence selection may use observed
   events after measurement, but cannot determine inclusion or what the author may see. Any
   incomplete evidence preview is `partial = true` and links to the full series/source locator.
6. **Delete disposable source copies.** Only after durable verified measurement/evidence receipts,
   required B work, and an exact source-redownload route exist, delete the fetched archive and
   temporary extraction owned by this task. Unique calibration/evidence/measurement files are
   not disposable. Verify absolute resolved paths stay inside the designated disposable root,
   reject reparse-point traversal, close readers, use native PowerShell literal-path deletion,
   verify disappearance and remeasure allocation. Never delete original shared data, model
   artifacts, sibling work or a computed unchecked directory. Log failure rather than claiming
   space was reclaimed.
7. **Advance.** Only after the receipt/checkpoint is durable does the next archive start. Failed
   or oversize jobs keep a reason and resume plan; they do not disappear from the denominator.

If a single archive plus working set cannot fit, use verified member/range streaming where the
source/format supports it, or an already authorized controlled remote execution/storage path
with the same blinding and lineage. If neither exists, queue the asset as `resource_blocked`.
Do not exceed the cap or claim its tests have been measured. If retained outputs eventually
exhaust the budget, pause acquisition until they can be durably transferred to an authorized
store and verified; do not delete the measurements. There is no fixed maximum tests, frames,
archives or science coverage: the queue aims at every eligible test and exposes actual resource
and method limitations.

## 10. Build order and acceptance evidence

These are tasks for the later implementation, not reports that they have run:

1. Establish custodian-only identity mapping, scoped manifests and protected-output routing;
   exercise the operational boundaries with synthetic canaries:
   condition names, partition labels and expected values in filenames/metadata/error strings
   must not reach a blind package. Host derives scope; tool/request input cannot grant it.
2. Add a versioned instrument-card parser, time/calibration validation and immutable manifests;
   prove pixel-only and unknown-time records cannot become metric outputs implicitly.
3. Implement one method per actual available instrument configuration; use synthetic dropped
   frames, clipping, occlusion, clock discontinuity and known geometric targets to verify failure
   semantics. Verify checkpointed versus contiguous processing is equivalent at boundaries.
4. Build comparator fixtures that catch unit/time-origin mismatches, shared-calibration covariance,
   incompatible endpoint definitions, protected reference access and selective failure omission.
   Verify the comparator cannot mutate submissions or reveal references to either measurer.
5. Exercise deterministic one-in-ten assignment, separate aliasing, independent annotations,
   disagreement preservation, and append-only versioning. Test a small inventory and a later
   inventory extension without reassigning previous repeats.
6. Exercise storage accounting, interrupted download, expansion overrun, unsafe archive paths,
   retention failure, re-fetch identity change and deletion confined to disposable paths. Test
   a job larger than budget returns `resource_blocked`, not partial success.
7. After frozen TRAIN membership, role-routing and method-card gates, perform the prescribed
   independent NASA TRAIN qualification;
   publish actual errors and domain limits, or a candid unqualified state. Only then scale the
   sequential queue. Report eligible, acquired, measured, repeated, reference-comparable,
   qualified, ambiguous, unavailable and resource-blocked coverage with reasons; protected
   counts remain sealed where they convey outcomes.

The R5/data owner and evaluator review the shared-contract mapping before implementation. No
shared source, generated schema, model forward or registry is changed by this design task.
No live NASA download, image measurement, final-test evaluation, commit, push or Rust formatter
was run to produce this document.

## 11. Page/test labels, rule-based clusters and evidence links

This section implements the **architecture** requested in the new section 2b; it generates no
actual label, page card, cluster membership, edge or interpretation. Fable owns
`docs/wsp/atlas/SCHEMA.md` and all controlled lists. It is now available and was read in full:
the current schema identifier is `ignis.atlas.test/1`. Use its existing fields/tokens and rules
exactly where they apply. Unsupported record kinds or label meanings wait for Fable's extension;
this does not block acquisition, evidence retention, measurements or already-supported labelling.
Do not create substitute enums or a parallel schema in this document. The quality states
proposed for measurement review above are not atlas phenomenon labels.

Schema-document SHA-256 at this read:
`b63b28e9763d1ad92892fa7424c093cbeae725b97571092c092a05eac1aeb1fc`.
This pins the text inspected here, not unseen atlas rows or the generated JSON schema.

#### Exact schema support and extension gaps after the schema-only read

| Requested capability | Existing exact schema vocabulary/rule | Gap or implementation consequence |
|---|---|---|
| Identity and grouping | `atlas_id`, `test_id`, `aliases`, `level`, `parent_test_id`, `group_id`, `group_rule`; levels `test`, `test_point`, `test_day`, `matrix_row` | A `test_day` or `matrix_row` is not an observed test. Preserve the level; parent-linked points must not be counted as independent ignitions. Split membership still comes from Fable's pinned split. |
| Partition handling | `project_partition` is `train`, `validation`, or `outside_partitions` | `outside_partitions` is not an authorization to release outcomes. This schema field does not encode the future all-NASA frozen train/validation/test assignment; do not add a `test` token or treat `group_id` alone as a frozen split. Custodian keeps authoritative split routing separately until Fable specifies integration. |
| Fuel, apparatus, geometry | `fuel.phase`, `hardware.rack`, `hardware.insert`, `geometry.configuration`, `geometry.flame_orientation`, `geometry.sizes.dimension` have closed lists in section 4 | Use only documented tokens. The configuration list covers the listed burner/droplet/flat-sample/heated-wire/aerosol-bag forms, but contains no generic arbitrary-geometry token. A source-supported shape outside the list requires Fable's proposal process, not mapping it to a convenient existing shape. |
| What actually happened | `labels.flame_type` is derived from configuration by `L1`; `labels.studied` is derived from investigation-title matches by `L2` | Neither field is an observed-event label. `extinction`, `cool_flame`, `liftoff`, `smoke_point`, `electric_field`, `burning_rate`, `aerosol` are existing `labels.studied` tokens, not proof those events occurred in a test. A closed observed-phenomenon/event vocabulary and its evidence/uncertainty semantics are absent; request an extension without assigning new tokens. |
| Fire-safety need | `labels.fire_safety_need` tokens `prevention`, `detection`, `suppression`, assigned by keyword rule `L3` | Apply `L3` only with its recorded matched source. An empty list means no match, not lack of relevance. A richer evidence-based page/test assignment needs Fable's rule/meaning extension; do not silently reinterpret these keyword outputs. |
| Available measurement modality | `measurements.items.kind`: `color_imaging`, `backlit_imaging`, `uv_imaging`, `intensified_imaging`, `still_images`, `radiometer`, `photomultiplier`, `thermocouple`, `pressure`, `gas_flow`, `gas_composition`, `air_velocity`, `particle_instrument`, `electrical`, `test_log`; status `not_filled` or `filled` | These are modality/catalogue entries, not a closed list of measured physical observables or qualification states. Liquid/flame diameter definitions, front position, channel-specific flame end and moment order need their own approved measurement mapping; do not insert them as new `kind` aliases. |
| Flow and condition clusters | `flow.direction`: `quiescent`, `concurrent`, `opposed`, `coflow`, `forced`, `burner_injection`; existing value-object status/basis/unit rules | Direction is not a Reynolds-number/transport regime. No closed flow-regime list is specified here. Rule clusters can use the supported direction/condition fields now, without inventing regime labels. |
| Crew atmospheres | `CREW`; `map.crew_atmospheres` and `crew_atmospheres_o2_only`: `earth_iss`, `moon_base_rover`, `spacesuit` | Reuse the exact section-7 bands, inclusivity and ambient oxygen/pressure requirements. Injected oxidizer is excluded. Oxygen-only entries are candidates, not members; no substitute habitat names or newly tuned thresholds. |
| Page cards and source evidence | Value-object `source` can hold `file`, `sheet`, `column`, `row`, `page`, `field`, `note` | This test schema does not define a page-card record, page-kind closed list, outcome-reveal/test-link field, word-span/figure-region locator shape, or actual assigning-model identity. The prose page kinds in section 11.1 are requirements, not approved enum tokens. Preserve source evidence pending Fable's record extension. |
| Clusters, edges, findings | Existing rule IDs describe test-value derivation, including `CREW`, `L1`, `L2`, `L3` | There is no cluster-record/rule-expression schema, typed edge-relation list or interpretation-record schema in this document. The requested output filenames do not constitute those missing schemas. Request the definitions rather than writing arbitrary JSON fields or relation aliases. |
| Assignment/audit provenance | Existing labels keep `value`, `rule`, `matched`, `source` | The test schema does not specify model provider/runtime ID, prompt/config digests, assignment version, blind repeat/adjudication records or per-kind audit units. These remain explicit Fable-owned integration gaps; do not discard the provenance while awaiting the approved representation. |

The schema's `status`, `basis`, `scope`, unit and flag lists remain unchanged. In particular,
`withheld_outcome` cannot be promoted to author-readable data merely because a split file
appears: the custodian must additionally establish permitted TRAIN membership and provenance.
`outcome_fields_available` names source-table columns uniformly; per-cell availability,
emptiness, extracted values and derived failure patterns remain outcome information. This
schema read is not evidence that its described generators/checkers were run in this task.

### 11.1 Label in the same authorized reading/measurement pass

At ingest, give every physical document page a stable page identity, document digest, page
number and extraction/rendering identity. A protected document labeller reads each page once
and emits the atlas-defined card from that reading, retaining its evidence. Include blank,
drawing-only, unreadable and unclassifiable pages with the schema's supported states; a
failure record preserves coverage where a card cannot be formed. The independent audit below
is the intentional second read, not a second full-corpus labelling pass. No arbitrary page or
label ceiling is imposed.

The card carries the schema's page-kind classification, investigation/test/condition mentions,
reported quantities and units, phenomenon and fire-safety-need labels, and explicit identification
of which tests' outcomes are revealed. Every assertion points to document/page and a word span,
table cell or figure region. A result table/figure needs a locator to its relevant entry, not
merely its caption. Units and raw quantity names stay as evidence; they do not silently become
new vocabulary terms. Test identity resolution is a custodian step using the catalogue, with
ambiguous aliases retained as unresolved. Never guess the test from the result value.

The measuring pass emits observational evidence and, through a separate vocabulary-only
labelling step, proposed test-phenomenon labels supported by that evidence. Its labeller sees
approved observations/pixels/traces, the closed definitions and instrument context, without
model predictions, expected phenomenon or partition. The custodian joins opaque aliases to
catalogue test IDs afterward. A document's claimed phenomenon and a measured phenomenon are
separate evidence-backed assertions until their agreement is established; neither overwrites
the other. Conditions supplied by the catalogue are distinguished from observed outcomes.

Do not use this design agent as a fresh labeller. A labeller records the **actual runtime model
identifier** supplied by trusted execution metadata (provider/model and version when exposed),
not merely the friendly name "Astra". Also retain prompt/tool/config digests, assignment ID,
schema/vocabulary versions and timestamp. If exact model-version provenance is unavailable,
record that limitation rather than guessing. Rule-produced records identify the deterministic
producer and the originating human/model assertions; they do not pretend a model assigned
them directly.

If a needed label is absent from the closed list, emit no invented replacement. Queue a proposal
with its evidence locator and justification for Fable's vocabulary review. The eventual authorized
producer records proposals in `docs/wsp/measure/VOCABULARY_PROPOSALS.md`; this task owns and
edits only the present file, so creates no such file. A versioned accepted definition triggers
documented re-labelling of affected records. A rejected proposal remains auditable.

### 11.2 Protected store and author-readable projections

The requested filenames are logical outputs: `pages.jsonl`, `test_labels.jsonl`, `clusters.json`,
`edges.jsonl`, and `findings.jsonl`, ultimately described in `docs/wsp/measure/LABELS.md` by the
implementation owner. JSONL rows follow Fable's schema; `clusters.json` is the specified JSON
rule catalogue. No file is created here. The all-NASA prompt's
`runs/dev/data_astra/work/labels/` may contain the protected outputs under the authorized
folder-discipline arrangement, provided authors never open it and their tools/searches exclude
it. The custodian supplies separate explicit paths for approved public/TRAIN projections;
authors do not derive those projections by opening the full files themselves. Record the
actual protected/private/projection paths in the scoped manifests before dispatch. No new OS
account or service is a prerequisite; a filename convention is not technical access control.

Derive visibility from all source/evidence dependencies. An author-readable projection may
contain approved public methods/apparatus information and resolved TRAIN observations only.
Public availability on the internet does **not** make a per-test result author-readable.
A page revealing mixed-visibility tests, any validation/test result, or an unresolved test
identity fails closed for authors. Do not expose its title, thumbnail, excerpt, quantity list,
phenomenon labels, outcome-revealing edge or searchable terms as a workaround. Unknown
outcome-reveal status also fails closed. A wholly resolved TRAIN-only page still needs the
document custodian's classification before release.

Page labels and evidence can themselves reveal outcomes even if the page bytes are hidden.
Protect all derived assertions, contradiction flags, cluster counts, missingness/QC patterns,
agreement reports and findings accordingly. A condition-only grouping is not permission to
publish its protected outcome distribution. Projection filters execute before summaries,
aggregation, caching and link traversal, not after a mixed result is rendered. The author's
public/TRAIN projection has a digest and the dependency/access-policy version that created it.

### 11.3 Clusters are explicit rules over versioned labels

Use only supported atlas field names, code lists and rules for crew-atmosphere categories,
flow direction, fuel phase and measurement modalities. Reuse `CREW` as written. Observed
phenomena, flow regimes, document topics and any additional material classes await the exact
extensions identified above; do not encode them as free strings or repurpose `labels.studied`.
All condition clusters use source-supported conditions, never flame behavior inferred as a
substitute for missing atmosphere/flow metadata.

Each cluster rule must specify its ID/version, input schema version, exact predicate over
typed labels/condition values, SI unit conversions and interval inclusivity, missing/conflicting
value handling, dependency digests and whether overlapping membership is allowed. Missing
condition data cannot be replaced by inferred outcomes. Store the rule and materialized
membership separately so all memberships can be rebuilt and diffed. An explicit predicate
over a versioned phenomenon label is legitimate grouping, but its members and summaries inherit
outcome protection. Embedding/similarity suggestions remain proposals until an approved rule
exists; they are not scientific clusters or evaluation groups. Clusters never redefine the
frozen train/validation/test groups.

### 11.4 Edge tables and interpretations

Use tables of typed edges from Fable's schema, not a graph database or a new ontology. The
requested relations cover test–investigation, test–catalogued file, test–page, page–quantity,
investigation–hardware/fire-safety need/crew atmosphere, parameter–justifying source,
model–scored test/score artifact, and conflicting table–sensor values. Retain source and target
IDs, relation version, evidence locator, producer identity, uncertainty/qualification where
relevant and visibility dependencies. No dangling inferred join is silently accepted.

Parameter/source and scored-model edges are imported later from authorized modelling/evaluation
artifacts by their respective owners. They never enter a blind measurer or labeller package.
A contradiction edge retains both original assertions and the unit/time/observable matching
evidence; it does not declare one true from numerical disagreement alone. Unresolved mismatches
remain unresolved, without selecting whichever supports a model.

Findings are short records **of kind interpretation**, per investigation and per rule-defined
cluster, with evidence links, scope/domain, disagreements and unknowns. Each claim must resolve
to openable evidence for its intended audience; a reference to hidden data is not permission
to summarize it to an author. Protected full-corpus findings stay protected. An authorized
public/TRAIN finding states that narrower coverage explicitly and does not imply all-group
behavior. The model identity/prompt provenance and version of every assertion are retained.

### 11.5 Blind random one-in-twenty label audit, minimum 100 per kind

Keep this audit separate from the one-in-ten **measurement** repeat. Before looking at label
values, freeze the source-unit/label-kind universe under the published schema, including units
where the first labeller assigned no label or reported uncertainty. Sampling only positive
assignments cannot detect systematic omissions. The audit unit and denominator for each kind
must be explicit; for multi-label fields, compare the complete assigned set for that source
unit, not a selectively counted collection of labels.

For each kind with `N` eligible units, use a logged random seed/private keyed permutation of
stable unit IDs, independent of A's labels or confidence, and select
`min(N, max(100, ceil(N/20)))` units without replacement. This implements the requested random
twentieth with the 100-per-kind minimum. When `N < 100`, audit all and explicitly disclose the
insufficient population; do not duplicate units to reach 100. Record inclusion probability and
the frozen sampling manifest. Protected selections stay protected; do not balance the sample
by outcomes or replace difficult/unreadable sampled units.

The independent labeller B sees the same source content/instrument evidence, the same frozen
definitions, and an independently aliased packet, but no A labels, annotations, uncertainty,
expected class or partition. A document labeller may need text that identifies a source/test;
it is a protected labelling role, distinct from the strictly blinded media measurer. It still
receives no model/hypothesis, NASA truth used as an expected label, or original label assignment.
The custodian performs all catalogue/partition joins afterward.

The comparator reports agreement **by kind**, exact-match fraction (set equality for multi-label
fields), confusion/disagreement counts including missing/uncertain assignments, effective sample
size and uncertainty clustered by originating page/test/document where dependence matters.
Review class-specific disagreements so a prevalent "none" label cannot conceal failure on a
rare phenomenon. Agreement is reproducibility, not ground-truth accuracy. Unreadable or failed
audit units remain in the report and do not vanish from the denominator.

Where agreement is below 0.90, repair the definition with Fable, version the vocabulary, and
relabel the **whole affected kind/domain**, preserving the old version. Do not repair only the
sample or resample until it passes. Rerun the frozen audit to expose the effect, and use a newly
preselected independent audit where unexposed units exist; if the population was exhausted,
say that the repeat audit is exposed rather than claiming fresh verification. Retain every
audit round and adjudication. Neither blind labeller sees A/B disagreement feedback before
its assignments are sealed; protected agreement summaries are not sent to model authors.

### 11.6 Additional implementation gates

Verify schema/vocabulary pinning, actual model-identity capture, page completeness, exact
evidence locators, unresolved/mixed-page access denial, and dependency-closed projection on
synthetic fixtures before real labelling. Verify the audit sizes for a population smaller than
100, between 100 and the one-twentieth crossover, and larger; verify negative/absent assignments
can be sampled and that vocabulary repair reprocesses all affected units. Confirm clusters
rebuild exactly from rules, no protected counts leak through edges/findings/caches, and every
interpretation is typed separately from measurements. These are proposed tests, not executed
checks. Query-time search, ranking, a fast judge and the application's map implementation are
explicitly outside this task.

## 12. Primary-source reading ledger

All external reads below occurred on 2026-11-14. Search dates/crawl ages are not publication
dates. No instrument specification below is assumed to be the actual flight configuration.

| Source and exact route | Portion actually returned/read | Use and limit |
|---|---|---|
| [NIST TN 1297 Appendix A](https://www.nist.gov/pml/nist-technical-note-1297/nist-tn-1297-appendix-law-propagation-uncertainty) | Full Appendix A web content, including covariance propagation and Type A/B examples; not the entire technical note | Uncertainty/correlation basis; the practical sparse representation is this protocol's design |
| [Klimek and Wright, Spotlight-8, NASA/TM-2006-214084 (2006)](https://ntrs.nasa.gov/api/citations/20060011194/downloads/20060011194.pdf) | Search method excerpts and returned PDF text through early Results-files heading (printed pp. 1–8 start; physical PDF pp. 1–12 start); not all 60 PDF pages | Image-tracking primitives, areas of interest and saved process sequence; not evidence of accuracy on any assigned experiment |
| [Zhang, Flexible Camera Calibration by Viewing a Plane from Unknown Orientations (1999)](https://robots.stanford.edu/cs223b04/JeanYvesCalib/papers/zhan99.pdf) | Search-returned abstract/introduction excerpt | Planar target and radial-distortion basis; full paper and an actual calibration implementation were not audited |
| [NASA/TP-2015-216046 FLEX report](https://ntrs.nasa.gov/api/citations/20150023456/downloads/20150023456.pdf) | Search-returned apparatus/diagnostics introductory excerpt only | Backlit versus visible/filtered emission channels; report results and tables not opened |
| [NASA SAME-R information sheet](https://www.nasa.gov/wp-content/uploads/2025/07/smoke-aerosol-measurement-experiment-same.pdf?emrc=b828be) | Search-returned methods summary | Moment identities and conditional distribution reconstruction; no test measurements extracted |
| [SAME moment-method paper, NTRS 20080012612](https://ntrs.nasa.gov/api/citations/20080012612/downloads/20080012612.pdf) | Search-returned Moment Method excerpt and number-distribution definition | Distinguish unnormalized moment from mean; no experimental tables opened |
| [TSI P-Trak Model 8525 operation/service manual](https://tsi.com/getmedia/d1f26f38-5f1d-4851-80a2-b99d95c7c9c8/Model-8525-P-Trak-Op_Svc_Mnl_1980380?ext=.pdf) | Search-returned Operation p. 21 excerpt | Number concentration and averaging provenance; no inference that catalogue limits apply unchanged to flight hardware |
| [Hamamatsu PMT handbook](https://www.hamamatsu.com/content/dam/hamamatsu-photonics/sites/documents/99_SALES_LIBRARY/etd/PMT_handbook_v4E.pdf) | Search-returned section 4.3.2 linearity excerpt | Saturation/nonlinearity concern; no specific NASA tube identified |
| [Hamamatsu PMT modules technical guide](https://www.hamamatsu.com/content/dam/hamamatsu-photonics/sites/documents/99_SALES_LIBRARY/etd/Highsensitivity_PMTModules_TPMO1120E.pdf) | Search-returned gain/dark-current/spatial-uniformity/temperature/drift/time-response sections | Instrument uncertainty checklist; not a source of NASA channel correction coefficients |

Search also surfaced investigation report/results metadata and unrelated secondary pages;
they were not adopted as evidence or opened. The BASS-II incident is recorded in section 1
without reproducing the result. Further apparatus-specific research must be done by a
custodian that can screen/redact result-bearing documents before methods reach a measurer or
author. The missing instrument-specific settings and independent TRAIN comparison are explicit
prerequisites to qualified measurements, not completed work. Protected acquisition and scoped
non-modelling labelling/measurement may proceed under the authorized operational separation;
stronger OS/service enforcement is a recommendation, not a prerequisite. Author outcome access
still requires the frozen split, proven TRAIN membership and an explicitly released projection.
