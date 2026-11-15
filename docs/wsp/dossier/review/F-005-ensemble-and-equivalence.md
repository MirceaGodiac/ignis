# F-005: Ensemble algebra and equivalence language

- **About:** C-27, C-45
- **Reviewer:** Astra, 14 November 2026
- **Verdict:** weakened
- **Severity:** record
- **Partitions touched while reviewing:** train only (read no run-level outcomes or derived outcomes); validation inputs and the existing aggregate claims as printed in the dossier; no validation outcome was newly opened

## What the dossier says

C-27 says averaging two forecasts gains one quarter of their mean squared disagreement over the mean of their scores, and that blending helps only when members differ in kind. C-45 says the droplet laws rebuilt from mechanism cannot be told apart from the train-informed laws on unseen days, citing paired intervals that include zero.

## What I did

Read C-27 and C-45 in `05_claims.md`, the committee row in `10_extension_map.md`, and the task-assigned README, handoff and review instructions. I derived the binary Brier / squared-error identity and the CRPS CDF-mixture identity directly. I read Gneiting and Raftery's 2007 paper, *Strictly Proper Scoring Rules, Prediction, and Estimation*: the abstract and definitions of propriety (p. 360), quadratic/Brier score example (p. 363), and CRPS definition and expectation representation (pp. 367–368). I also read Lakens (2017), *Equivalence Tests: A Practical Primer*: abstract and TOST/equivalence-bound discussion. I did not read case-level results or rerun any score.

Sources: [Gneiting & Raftery (2007), DOI](https://doi.org/10.1198/016214506000001437), [accessible paper PDF](https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/Gneiting07.pdf); [Lakens (2017), equivalence tests](https://doi.org/10.1177/1948550617697177).

## Evidence

For scalar forecasts `a,b` of outcome `y`, squared error obeys the exact per-case identity

`((a-y)^2 + (b-y)^2)/2 - (((a+b)/2-y)^2) = (a-b)^2/4`.

The identical identity applies to Brier loss for two binary-event probability forecasts `p1,p2` and observed label `z`, since Brier loss is `(p-z)^2`. It also applies to an equal-weight *mixture distribution* under CRPS. In CDF form, write `H_y(t)=1{y≤t}` and `CRPS(F,y)=∫(F(t)-H_y(t))²dt`. Then

`(CRPS(F,y)+CRPS(G,y))/2 - CRPS((F+G)/2,y) = (1/4)∫(F(t)-G(t))²dt ≥ 0`.

Thus C-27's first identity is sound when disagreement is defined in the score's geometry: scalar/probability difference for squared-error/Brier, integrated squared CDF difference for CRPS mixtures. It need not be the squared difference between member means. The reduction is relative to the average member loss on the same case, not necessarily relative to the better member. Nor does this pointwise identity prove lower expected future risk under a shifted or adaptively selected evaluation distribution.

“Only when members differ in kind” does not follow. For Brier/squared error and CRPS mixture scoring, any score-relevant disagreement gives a nonnegative same-case reduction relative to the mean member loss, whether disagreement is about mechanism, calibration, or arbitrary noise. “Differ in kind” can be a scientific reason to preserve complementary scenarios, but it is neither a necessary nor sufficient condition for the algebra. The reported normal/inverse member differences and score changes in C-27 are empirical summaries, not evidence for that causal rule.

C-45's paired intervals containing zero mean the comparison did not establish a nonzero difference under the interval procedure used. They do not establish equivalence or that the models “cannot be told apart.” Equivalence requires a scientifically justified margin fixed before examining the comparison and an interval/test showing the plausible difference lies wholly inside that margin (for example, two one-sided tests). The dossier gives intervals of roughly ±5 to ±10 µm but no equivalence margin, and notes only nine validation days on a partition used repeatedly. The only warranted reading is “this look did not resolve which model is better”; it supplies no equivalence guarantee and no coverage claim for new days.

Competing event labels and administrative censoring also matter to any future comparison: a known planned stop is an observed competing termination if the target is *termination cause*, while a run that simply ends observation before the target horizon is censored. Those are different labels. A binary Brier calculation cannot silently turn a censored case into a negative outcome. This is a design limit, not a new claim that the current score implementation mishandles it.

## What should change

Keep the first identity, define its disagreement term for the score used (including integrated squared CDF difference for CRPS mixtures), state that it compares the mixture to mean member score on the same case, and remove “only when members differ in kind.” Treat an equal-weight committee as a transparent scenario statement, not as automatically superior to its best member or proven to improve future performance. Rewrite C-45 as: “This small, repeatedly used validation look did not resolve a score difference; equivalence and future-day performance are undetermined.” Do not add a validation look to settle this wording. For any future event score, state competing events, censoring, and forecast horizon explicitly and use a scoring method appropriate to the observed status.

## Response

(The author answers here.)
