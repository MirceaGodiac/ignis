# Model card — `acme_normal.wsp` (spec `acme_normal-wsp-v1`)

**State: experimental.** Hand-written weights, not trained. On validation none of its candidates
beats a constant-hazard baseline on the extinction row. Uncertainty is assumed, not calibrated.

## What it predicts

For a burner-stabilised flame in the **normal** configuration (PSI-159: fuel — ethane, propane or
butane, usually diluted — issues from the porous burner into an O₂/N₂ chamber), given the
conditions and that the hot flame is established at the cutoff: the probability over time of
spontaneous extinction, of termination by the operator, of still burning, and the mass left at the
model horizon. An operator stop is its own cause and is never counted as an extinction.

It does not predict the counterfactual lifetime without the operator, cool-flame phases, or the
inverse configuration.

## Releases

| | Release id | How written | Nonzero / authored |
|---|---|---|---|
| W0 | `sha256:7ef0c7ab236bf8b495e6659822839bc5d9d5d9955096cc1ff96f3863aecc66f5` | One shot, before any ACME endpoint was read | 13 / 284 |
| W1 | `sha256:a75ba790658108fe8218677326376fedbc34c0d1422ef34c67aff00e619f251c` | Revision from the train endpoint table | 30 / 284 |
| W2 | `sha256:fa1ab73e498a9d1a5a7f19236bf9d14c28b5ec4e5d832e6d2b688408a209eb10` | Flatter hazard shape after W1's aggregate validation score | 30 / 284 |

Spec `sha256:4b575d7af8f8b05a18cb7ec9264df155191632c27d040d9cc8f5c03323e97a53` · forward
`sha256:d66abd56024b49f502c12e1bdad496b78b5dbfe0ab8f7cc7ba90f7a484c9191a` · author `claude-fable-5-2`.
Files: `configs/models/acme/wsp/normal-{w0,w1,w2}-fable/`.

## Computation

```text
x = (c_logp, c_logm, c_o2, c_xf, c_tad, c_zst, f_ethane, f_butane, x_t),  x_t = ln(1 + t/10 s)
ln(λ_ext·1 s) = N_ext(x)      ln(λ_op·1 s) = N_op(x)      12 hidden units each
```

`c_logm` is the **fuel** mass flow over 1 mg/s, `c_o2` the **ambient** oxygen fraction,
`c_xf` the **injected** fuel fraction. The two hazards compete from the cutoff on; within a step the
lost mass is split in proportion to the hazards, so the four rows sum to one exactly.

| Unit (W2) | Reads | Computes |
|---|---|---|
| `ext.skip`, `ext.b0` | c_logp, c_o2, c_tad, x_t | Level of the hazard for a median life L = 3.4 + 1.2·c_o2 + 0.7·c_tad − 0.3·c_logp (ln seconds), growing as (1 + t/10 s)^1.5 |
| ext 0–1 | the same combination L | The level a long median life needs (knots at L = 2.0 and 3.3) |
| ext 2 | c_tad | Below about 2100 K the flame is gone within a second |
| ext 3–4 | c_logm, c_o2 | Flow shortens the life only in enriched ambient (gate on c_o2) |
| `op.*` | x_t | About 0.004 per second once the flame is 20 s old, no dependence on conditions |

**W0** assumed a radiative-loss index (flow^1.6, pressure^0.5, T_ad) around a median life of 20 s
and an operator hazard peaking near 45 s. The train table showed two regimes instead: 5–20 s in
air-like ambient, 30–300 s in enriched ambient with a life falling as about flow^−1.8.

## Results (development)

Integrated Brier score over 6–120 s, lower is better. Validation: 84 cases from **2 sessions**.

| | Validation extinction | Validation operator | Train extinction | Train operator |
|---|---|---|---|---|
| W0 (zero-shot) | 0.219 | 0.125 | 0.159 | 0.065 |
| W1 | 0.261 | 0.097 | 0.122 | 0.063 |
| W2 | 0.244 | 0.098 | 0.119 | 0.062 |
| constant baseline (fitted) | 0.214 | 0.099 | 0.225 | 0.072 |
| linear baseline (fitted) | 0.185 | 0.093 | 0.108 | 0.039 |
| acme_normal.hazard (fitted) | 0.187 | 0.093 | 0.111 | 0.044 |

The revisions fit train and do not carry to the two validation sessions on extinction. The
validation runs were not opened to find out why, so the cause is not known.

## Checks and limits

- Build checks passed; `tests/acme_forward.rs`: closed-form competing risks to 1e-12, role-aware
  feature columns, refusal of inverse prefixes, a known stop lands on the operator row.
- Latency 0.33 ms per forecast (p50, one thread); 2.4 kB of parameters.
- The stored time origin mixes two clocks for runs with a TP log; a future "MFCs off" is not in
  the prefix. Tudor's evaluator on `feat/research-product-t2` would count an event artifact
  (no quantity) as failed.
- Dilute-fuel flames in enriched ambient (four train runs) contradict the flow gate.
