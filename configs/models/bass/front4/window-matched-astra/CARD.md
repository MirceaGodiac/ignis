# BASS matched-window observation operator

**Data-informed, not zero-shot. Experimental; not trained.** Release identity:
`release.json`. Parent: joint Astra
`sha256:79d27b0203a7935f1bdaa275e2f508f24f0d2a52edd67ebe38fd327647c5a1e0`.

This is a new forward (`bass_front.wsp4`) with **all numerical weights frozen**
to the parent. Fable and Astra retain credit for their original literal values.
The new edits file is an exact literal transcription, not a new parameter proposal.
Its compiled patch and construction manifest retain the actual ancestry and exposure.

| Inventory | Count |
|---|---:|
| Inherited authored scalar capacity, including structural zeros | 180 |
| Inherited nonzero authored positions | 48 |
| Inherited fixed spec constants | 11 |
| Newly chosen or changed scalar positions | 0 |

Every tensor name, shape and value is unchanged. The full-model directly TRAIN-read
count remains the parent's incompletely classified count, not zero. The inherited
inventory documents at least two measurement-backed positions; it does not claim
an exact census or effective degrees of freedom.

The corrected operator compares each window's observed mean with the modeled
mean at the same timestamps. It applies to tip, delayed base, forecast anchor
and the increments used by the inherited uncertainty law. Each scenario still
has its own deterministic prefix encoding. Mean-time gain/lead clocks, base-delay
selection, physical laws, events, mixture weights and integration step are fixed.

An empirical window mean of a nonlinear modeled path differs from that path at
the mean time. Correcting that identity does not establish the modeled path's
physical accuracy. In particular the quench limitation remains; the uncertainty
law still lacks a covariance model for averaged optical noise. The forecasts
remain conditional on supplied future fan history.

Preregistration and exposure: `docs/wsp/dossier/ASTRA_BASS_OPERATOR_20261115.md`.
Exact transcription audit and frozen identities:
`docs/wsp/feedback/astra-bass-operator-20261115/`.
Five synthetic integration tests and an independent GPT-6 Astra high-effort
symbolic/source check precede the frozen TRAIN comparison. Synthetic results
are not NASA qualification evidence. TRAIN has three scored burns from one
day; a lower mean cannot establish generalization and causes no automatic adoption.

Reproduce construction using the committed literal files (no parameter search):

```powershell
$env:CARGO_TARGET_DIR='runs/tooling/astra-train-comparison-target'
cargo build -j 2 -p ignis-cli -p ignis-wsp-dev
& "$env:CARGO_TARGET_DIR/debug/ignis.exe" wsp compile --family bass_front4 --edits configs/models/bass/front4/window-matched-astra/edits.wsp --meta configs/models/bass/front4/window-matched-astra/meta.json --out runs/tooling/front4-patch.json
& "$env:CARGO_TARGET_DIR/debug/ignis.exe" wsp rewrite --family bass_front4 --patch runs/tooling/front4-patch.json --binding configs/models/bass/front4/binding.json --root artifacts
```

The one registered scoring invocation is
`python tools/wsp/astra/bass_operator_train.py --execute`, after its protocol
commit. It refuses to overwrite an existing result directory.
