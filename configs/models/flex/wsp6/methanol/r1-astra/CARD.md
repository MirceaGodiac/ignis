# flex_methanol.wsp6 — Astra r1

**Data-informed, not zero-shot. Experimental; not trained; no new validation look.**

Release: `sha256:60cf1f53e09f72e79f9993e21e00bd61c4088a54d2b511b6cb47b98392a56217`. Parent: `sha256:a8e957efdb5df6c3cdb2732f31c02a6ee6de7d5711759e9202a35a8313aa91a5`.

Main author: `gpt-6-astra`, effort `max`; Fable is credited for exact inherited literals.

Only future hot-radiative endings without He/CO2 use coupled cooling. Water activity, camera view loss, already-unlit thermal history and diffusive endings retain predecessor limits. Pure-liquid, room-environment and 330 K starting-temperature assumptions are uncertain; extrapolations are disclosed in the trace.

| Inventory | Count / status |
|---|---|
| Authored tensor capacity (including declared structural zeros) | 456 |
| Nonzero authored positions | 120 |
| Fixed spec constants | 20 |
| Explicit literal positions copied unchanged from Fable | 111 |
| Explicit positions in full R1 patch (including authored zeros) | 161 |
| New positions written by Astra in this patch | 50 |
| Cumulative new N / A / L / S / U positions | 20 / 1 / 5 / 22 / 2 |
| New directly train-read positions, cumulative | 0 |
| Full-model measurement-backed train count | unknown; not zero (one explicitly identified mixed-origin train-sd floor) |

N = source property; A = apparatus/environment (may be assumed); L = closure; S = structural/numerical; U = uncertainty judgment. Inactive carrier/branch placeholders count as S. Counts are expanded positions, not unique numbers or effective degrees of freedom.

The inherited block has mixed origins and is conservatively marked **inherited, train-informed**. Its exact directly train-read subset was not recounted, so the full model's count is not zero merely because a new block has zero direct-train entries. Reasons for new values are beside every assignment in `addition.wsp` (R1) or `edits.wsp` (joint).

The subsequent documentary audit `docs/wsp/feedback/astra_inherited_origin_audit.md` establishes the conservative inherited bounds above; exact inherited transcription proves those coordinates survive here. Measurement-backed includes explicitly described empirical summaries/differences, with raw transcriptions and mixed-origin judgments separated in that audit. No bound is an exact census or an effective-freedom count.

R1 uses the explicit new-spec full-literal rewrite route; the parent records ancestry without array inheritance. The joint is an ordinary same-spec literal revision. `inventory.json` verifies exact inherited transcription and the executing forward hash. See dossier `ASTRA_PROPERTIES.md`, `ASTRA_ERROR_BUDGET.md`, and `ASTRA_RESULTS.md` for source limits, priorities, and actual train results. No adoption or generalisation claim follows from a train score.
