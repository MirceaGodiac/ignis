# slice2: second law (PROTOCOL 13.4), development, in-sample

- Dossier: `docs/wsp/research/phys_slice2.md`, sha256 `333af88eb8ebed9988827578829fc7f74b53f71e1fe0807ffa01e36e1e343906`.
- Release: `sha256:d586d0d3d09c50d1d61335fe422ccb15ee1a8ab48e105d79dd6bf8663c820b61` (slug `slice2`, `s1-fable`); parent: slice W0 `sha256:d10c94bc6ea0bcb957f89734393b0cbabbd44fdf0ac9c2e440f051a33cd3006e`. Edits: the W0 block with every `lifted.logit.*` line written as `past_lift.logit.*`, then the new lines, verbatim.
- Scoring-time rule (dossier): the log's `lifted` label is scored as p_lift_noted * (p_past_lift - p_past_blow) / (1 - p_past_blow); `blow_off` from its own target.
- Neutral check (forecasts only): `blow_off` identical to the parent on every line (differing: train 0, validation 0).
- Cabin oxygen supplied to validation lines without one (PROTOCOL 13.2): the median of the 275 train lines that record it, x_o2 = 0.219122. (as in 13.2; the 41 ethylene lines stay refusals).

Development, in-sample. Brier, mean of day means (pooled line mean in brackets), on the lines both answer; lower is better.

| target | partition | lines | parent (W0) | slice2 | incidence pooled | incidence by tube | never |
|---|---|---|---|---|---|---|---|
| lifted | pooled | 302 | 0.1474 (0.1695) | 0.0333 (0.0368) | 0.0297 | 0.0293 | 0.0306 |
| lifted | train | 268 | 0.1559 (0.1798) | 0.0305 (0.0346) | 0.0258 | 0.0249 | 0.0265 |
| lifted | validation | 34 | 0.0886 (0.0886) | 0.0536 (0.0536) | 0.0563 | 0.0596 | 0.0588 |
| blow_off | pooled | 302 | 0.1089 (0.1151) | 0.1089 (0.1151) | 0.0509 | 0.0480 | 0.0538 |
| blow_off | train | 268 | 0.1023 (0.1101) | 0.1023 (0.1101) | 0.0352 | 0.0335 | 0.0362 |
| blow_off | validation | 34 | 0.1548 (0.1548) | 0.1548 (0.1548) | 0.1605 | 0.1493 | 0.1765 |

lifted by day:

| day | parent | slice2 |
|---|---|---|
| 040 | 0.0199 | 0.0017 |
| 048 | 0.0037 | 0.0003 |
| 058 | 0.2002 | 0.0703 |
| 061 | 0.7116 | 0.1087 |
| 065 | 0.0886 | 0.0536 |
| 069 | 0.1090 | 0.0095 |
| 072 | 0.0385 | 0.0219 |
| 079 | 0.0080 | 0.0007 |

Days lower (lifted): **8 of 8**; blow_off identical.

**Reading (13.4):** lifted pooled 0.0333 against the parent 0.1474 (best baseline: incidence by tube 0.0293), lower on 8 of 8 days; blow_off unchanged: **kept**. slice2 still scores above the best baseline on lifted, and blow_off stays the W0 forecast.
