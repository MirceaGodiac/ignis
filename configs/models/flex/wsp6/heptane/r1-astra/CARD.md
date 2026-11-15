# flex_heptane.wsp6 — Astra r1

**Data-informed, not zero-shot. Experimental; not trained; no new validation look.**

Release: `sha256:45e753e02ac19bd613fa8c31c169eb5c2439c19a90adaa7d8a9770349bdc2475`. Parent: `sha256:b6bc5a9a502f8b8559d05ce03a8e62bb195c53ee159a735ae4ab3bc99fb20f2c`.

Main author: `gpt-6-astra`, effort `max`; Fable is credited for exact inherited literals.

Only future hot-radiative endings without He/CO2 use coupled cooling. This does not solve the dominant helium, high-CO2 or cool-flame discrepancies. The 360 K initial state, constant liquid properties and no-cloud-heat assumption are not observations; source extrapolations are explicit.

| Inventory | Count / status |
|---|---|
| Authored tensor capacity (including declared structural zeros) | 653 |
| Nonzero authored positions | 145 |
| Fixed spec constants | 20 |
| Explicit literal positions copied unchanged from Fable | 141 |
| Explicit positions in full R1 patch (including authored zeros) | 191 |
| New positions written by Astra in this patch | 50 |
| Cumulative new N / A / L / S / U positions | 20 / 1 / 5 / 22 / 2 |
| New directly train-read positions, cumulative | 0 |
| Full-model measurement-backed train count | unknown; not zero (one explicitly identified mixed-origin train-sd floor) |

N = source property; A = apparatus/environment (may be assumed); L = closure; S = structural/numerical; U = uncertainty judgment. Inactive carrier/branch placeholders count as S. Counts are expanded positions, not unique numbers or effective degrees of freedom.

The inherited block has mixed origins and is conservatively marked **inherited, train-informed**. Its exact directly train-read subset was not recounted, so the full model's count is not zero merely because a new block has zero direct-train entries. Reasons for new values are beside every assignment in `addition.wsp` (R1) or `edits.wsp` (joint).

The subsequent documentary audit `docs/wsp/feedback/astra_inherited_origin_audit.md` establishes the conservative inherited bounds above; exact inherited transcription proves those coordinates survive here. Measurement-backed includes explicitly described empirical summaries/differences, with raw transcriptions and mixed-origin judgments separated in that audit. No bound is an exact census or an effective-freedom count.

R1 uses the explicit new-spec full-literal rewrite route; the parent records ancestry without array inheritance. The joint is an ordinary same-spec literal revision. `inventory.json` verifies exact inherited transcription and the executing forward hash. See dossier `ASTRA_PROPERTIES.md`, `ASTRA_ERROR_BUDGET.md`, and `ASTRA_RESULTS.md` for source limits, priorities, and actual train results. No adoption or generalisation claim follows from a train score.
