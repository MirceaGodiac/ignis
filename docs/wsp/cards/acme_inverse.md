# Model card — `acme_inverse.wsp` (spec `acme_inverse-wsp-v1`)

**State: experimental.** Hand-written weights, not trained. W1 reaches the fitted linear baseline on
validation and stays behind the full fitted model. Uncertainty is assumed, not calibrated.

## What it predicts

For a burner-stabilised flame in the **inverse** configuration (PSI-159: oxidiser — O₂/N₂ — issues
from the porous burner into a fuel/N₂ chamber), given the conditions and that the hot flame is
established at the cutoff: the probability over time of spontaneous extinction, of termination by
the operator, of still burning, and the mass left at the model horizon.

This is a separate model from the normal configuration: its own spec, reference scales, patch and
release. The code is shared; no coefficient is. A release of one refuses a prefix of the other.

## Releases

| | Release id | How written | Nonzero / authored |
|---|---|---|---|
| W0 | `sha256:843eef56a0746d9f2b6db323627d94aeb45674bcc326ff6ddbbda215a644316a` | One shot, before any ACME endpoint was read | 13 / 284 |
| W1 | `sha256:785b6b9d33bd3dd2cd8ee67b29548d8418a6d80ca978f8013c83954d2bcac452` | Revision from the train endpoint table | 23 / 284 |

Spec `sha256:f8afec5917a7e7424c5a310344e466c486014340677d38bdbb30be8ab127bc21` · forward
`sha256:d66abd56024b49f502c12e1bdad496b78b5dbfe0ab8f7cc7ba90f7a484c9191a` · author `claude-fable-5-2`.
Files: `configs/models/acme/wsp/inverse-{w0,w1}-fable/`.

## Computation

Same form as the normal model, with the opposite roles: `c_logm` is the **oxidiser** mass flow over
5 mg/s, `c_o2 = (injected x_O2 − 0.50)/0.25`, `c_xf = ln(ambient fuel fraction / 0.10)`.

| Unit (W1) | Reads | Computes |
|---|---|---|
| `ext.skip`, `ext.b0` | c_xf, x_t | A hazard growing as (1 + t/10 s)³ whose level falls with the ambient fuel fraction: median life about 10 s at 8 % fuel |
| ext 1 | c_xf | From 9 % fuel upward the level falls faster: about 75 s at 14 %, minutes at 20 % |
| `op.skip`, `op.b0`, op 0 | x_t | About 0.008 per second from 10 s on |
| op 1–4 | x_t | A bump of e⁴ between 62 s and 67.5 s: many runs are stopped at 65.5–66.5 s, a protocol time the prefix does not carry |

**W0** carried the normal model's flame-size reasoning over to the oxidiser flow (flow^1.6,
pressure^0.5, T_ad) with a median life of 17 s. The train table did not support any of those
terms: the ambient fuel fraction decides, and W1 removes them.

## Results (development)

Integrated Brier score over 6–120 s, lower is better. Validation: 49 cases from **2 sessions**.

| | Validation extinction | Validation operator | Train extinction | Train operator |
|---|---|---|---|---|
| W0 (zero-shot) | 0.302 | 0.135 | 0.326 | 0.202 |
| **W1** | 0.177 | 0.124 | 0.069 | 0.085 |
| constant baseline (fitted) | 0.252 | 0.153 | 0.260 | 0.162 |
| piecewise baseline (fitted) | 0.251 | 0.127 | 0.242 | 0.144 |
| linear baseline (fitted) | 0.173 | 0.131 | 0.064 | 0.074 |
| acme_inverse.hazard (fitted) | 0.143 | 0.118 | 0.068 | 0.076 |

W1 was written from train endpoints only; of validation only W0's aggregate scores had been seen.
The fitted baselines refuse 5 validation cases that the literal model forecasts.

## Checks and limits

- Build checks passed; `tests/acme_forward.rs` covers both configurations.
- Latency 0.33 ms per forecast (p50, one thread); 2.4 kB of parameters.
- The 66 s bump is protocol knowledge from three train sessions; a campaign with another protocol
  would make it wrong.
- With injected air (21 % O₂) at low ambient fuel the train runs split between immediate
  extinction and operator stops; the model does not separate them.
