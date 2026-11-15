# Independent discrepancy checker B

This method is frozen before opening numerical NASA payloads. The input authority
is source/question protocol SHA256
`944d37266906bb2e6e20c85a8570e7f92e913fa9fa6a8310ea5b62daa2ebbe99`.
No first analyst code, derived outputs, endpoint implementation, model, feedback,
handoff, holdout, validation series or final-test artifact is an input.

## Frozen method B1

1. Verify the protocol hash, unique TRAIN identities, family/test/run consistency,
   source pathname identity and hash bindings before opening any numerical
   payload. Open only explicitly bound raw source roles and tables. Table rows
   are selected by identity before any outcome cell is parsed. Private records
   retain source hashes, physical row/line locators, original tokens and units.
2. Independently parse exact Test Point headers and literal `testpointName`
   assignments. Compare each with the routed identity. A mismatch is unresolved;
   no outcome similarity establishes identity. TP clock aliases require a unique
   matching calendar Script start. Otherwise the TP join remains unresolved.
3. Independently parse literal `stabilizationTime1Sec`, and the explicit
   `Flame stabilization #1` and `Flame stabilization #2` timestamps. Compare the
   elapsed observed interval with the literal planned interval only when identity,
   clock origin, uniqueness and ordering are established. Preserve missing,
   ambiguous, interrupted and unqualified-clock cases. Difference alone does not
   establish a table error or a cause.
4. Compare MFCA1/2/3 command and corresponding readback voltages only at exactly
   equal, unique acquisition timestamps, with common origin and documented units.
   Use decimal arithmetic on original tokens; reject duplicate timestamps rather
   than choosing one. No interpolation, time shift, smoothing, transient exclusion
   or consecutive-fill differencing. Retain all sample comparisons privately.
   No conversion from voltage to flow without a documentary instrument/channel
   calibration and its applicable conditions.
5. Independently check original table identity uniqueness and numeric support.
   A scalar or explicit interval remains that kind of support. Do not interpret
   ambiguous prose as a scalar. Table endpoint comparisons remain unresolved:
   PMT dark is not physical extinction, and B1 has no independent documentary
   dark baseline or qualified physical endpoint observable. Do not import v2.
6. Approved scalar fractions can be range-checked independently. Sum only an
   explicitly complete component set for one identity, with all values resolved
   on the same basis; never impute missing components. Unit conversion is checked
   only against a separately supplied documentary qualification. If the binding
   does not expose original source, units and locator, keep it unresolved.

For each question, preserve these ordered cause checks: own independent reading
and control agreement; independent third NASA source (as-run script / TP execution
log when independently identity-bound); physical consistency with explicit
assumptions (nonnegative elapsed time, fraction domain, completeness for closure);
patterns as descriptive clues only; same observable and definition. Third-source
existence does not prove independent measurement. Unknown uncertainty remains
unknown. Quantization is not instrument accuracy. No normalized residual or
scientific superiority claim is produced.

All source values and comparisons stay in private `work/discrepancy_checker_b`.
Console and coordination output contain only operational counts, hashes and
artifact paths. Synthetic tests exercise isolation and parsing invariants, and
never qualify NASA measurements or a model. Execution requires `--execute`.
