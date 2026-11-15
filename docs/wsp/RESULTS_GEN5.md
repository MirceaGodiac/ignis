# Development results of the fifth generation (14 November 2026)

The one registered validation look of the fifth generation: candidates, rules and train numbers were
committed in `00635b2` ([PROTOCOL.md](PROTOCOL.md) section 10) before any number on this page existed.
Validation was read as the aggregate tables below and nothing else. **The test partition was not read.**
Every release here is data-informed; the validation partitions have been looked at several times before
(`dossier/07_limits_and_disclosures.md`, 7.2), so these are development numbers.

How the tables were made: `tools/wsp/dev_eval.py` (sealed forecasts, `ignis eval compare --intersect`, cutoffs
1, 3 and 5 s) for droplets and burner flames, `wsp-dev front-feedback` for the fabric flame. Three things differ
from the earlier results page and are stated in the protocol:

- **Droplets** were scored on the private copy of the data root (`runs/dev/data_ext/`), whose prefixes carry the
  flame-camera series, for every release in the table, the carry included. The evaluator's shared vault holds the
  continuations of the shared records and refused the private copy's (write-once, different content), so the
  private copy's validation continuations were deposited into a vault of their own (`IGNIS_HOLDOUT=
  runs/dev/holdout_ext`) by the evaluator's own command. Neither vault was opened by the author.
- **Burner flames** were sealed twice by the same releases: on the private copy (prefixes carry the script's
  planned stop) and on the shared records (they do not). The evaluator keys a sealed forecast by run and prefix,
  so its comparison now holds both; `tools/wsp/analysis/cmp_split.py` splits it into the two tables and reproduces
  the evaluator's own numbers for the half the evaluator had scored alone (largest difference 6e-17).
- **The fabric flame** was scored once on the sealed validation tracks, which the author has not opened.

## Outcome by the rules written beforehand

| Family | Candidate | Validation | Carried model on the same cases | Rule | Outcome |
|---|---|---|---|---|---|
| FLEX methanol | `flex_methanol.wsp5` P1 | CRPS 42.5 µm | 39.5 µm | CRPS not higher | **not adopted**; `flex_methanol.wsp2` D3 stays |
| FLEX n-heptane | `flex_heptane.wsp5` P0 | CRPS 47.2 µm | 46.1 µm | CRPS not higher | **not adopted**; `flex_heptane.wsp2` D2 stays |
|  | `flex_heptane.wsp5` P1 | CRPS 48.4 µm | 46.1 µm | CRPS not higher | **not adopted** |
| ACME normal, planned stop known | `acme_normal.wsp5` P1 | extinction 0.1452, flow termination 0.0747 | 0.1553, 0.0984 | extinction not worse | **adopted** for prefixes that carry the planned stop |
| ACME normal, shared records | the same release | extinction 0.1552, flow termination 0.0968 | 0.1553, 0.0984 | extinction not worse | **adopted**; level with the carried model (a difference of 0.0001) |
| ACME inverse, planned stop known | `acme_inverse.wsp5` P1 | extinction 0.1348, flow termination 0.0478 | 0.1606, 0.0939 | extinction not worse | **adopted** for prefixes that carry the planned stop |
| ACME inverse, shared records | the same release | extinction 0.1288, flow termination 0.0880 | 0.1606, 0.0939 | extinction not worse | **adopted** |
| BASS | `bass_front.wsp2` P1 | tip CRPS 0.114 sample widths; track ended 0.198 | 0.117; 0.178 | tip not higher and track-ended not worse | **not adopted** (track-ended worse); `bass_front.wsp` D1 stays |

What the table does not say by itself:

- The three droplet candidates were better than the carried models on train (28.7 against 29.9 µm, 43.9 and
  44.6 against 46.2) and are worse on validation by 1 to 3 µm. Every paired interval contains zero (methanol:
  carry minus P1, −3.1 [−12.1, +4.6] µm over nine days), so validation does not show that the physical laws
  are worse; it shows no gain, and by the rule that is a refusal. The 90 % band of the heptane camera
  candidate covered 0.88.
- The normal-flame candidate without the planned stop is the carried model with one corrected pair of weights,
  and scores as the carried model does. Its gain comes from knowing the planned stop.
- The inverse-flame candidate gains at extinction from its second scenario (0.129 against 0.161 without the
  planned stop). Knowing the planned stop halves its flow-termination score (0.048 against 0.088) and costs
  0.006 at extinction.
- Both burner-flame validation partitions hold two independent clusters (sessions). The paired intervals that a
  cluster bootstrap gives from two clusters are not informative, and the adoptions rest on point estimates.
- The planned stop is an input of the private copy of the data root. It is not in the shared records or
  contracts; whether it may be an input is argued in `dossier/02_data_classification.md` and is for the data
  and evaluation lanes to accept or refuse. The rows "shared records" are what the adopted releases do
  without it.
- The fabric candidate differs from D1 only for the narrow sample. Its tip score is 0.003 lower with an interval
  that contains zero ([−0.010, +0.003]); its track-ended score is worse, and the rule refuses it.

## Droplets (private data root; each release reads the same prefixes)

### FLEX methanol, fifth generation (PSI-69)

Cohort `sha256:9e31b1aa6b56b2086aac85776d27778bb1680a83fcdfc543dad62e0385da637e` · split `sha256:660b7d9ede629b3009b4b761ea9e5875560719566e3dea21112f607cddd49894`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| flex_methanol.wsp5 P1 — data-informed (train); physics-first laws, flame cameras read when the prefix carries them | `sha256:7884e61c9995d40b7f12f140a68b782aa02a811e774143cd128f6e71e25c141a` | `sha256:c4924921…` | 92 / 406 |
| flex_methanol.wsp5 carry of D3 — the carried numbers on the sixth droplet forward | `sha256:2181b643430a31aefa8e774e9b9523f6f09b343957f1bdcae04ea435512545be` | `sha256:c4924921…` | 77 / 406 |
| flex_methanol.wsp5 D2-law — hand-written reference | `sha256:575b0f63d5939811a2a7083eb302443d51eb486b394e67c3dd8d54d864e47a2d` | `sha256:c4924921…` | 37 / 406 |

**Validation** — 65 common cases; CRPS and MAE in µm.

| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |
|---|---|---|---|---|---|
| flex_methanol.wsp5 carry of D3 (the carried numbers on the sixth droplet forward) | 39.5 | 52.8 | 0.71 | 0.90 | 8.64 |
| flex_methanol.wsp5 D2-law (hand-written reference) | 91.1 | 113.6 | 0.55 | 0.76 | 7.81 |
| flex_methanol.wsp5 P1 (data-informed (train); physics-first laws, flame cameras read when the prefix carries them) | 42.5 | 59.5 | 0.67 | 0.90 | 8.61 |

Paired difference to flex_methanol.wsp5 P1 (data-informed (train); physics-first laws, flame cameras read when the prefix carries them) (other − reference, µm, 95 % CI):

- flex_methanol.wsp5 carry of D3 (the carried numbers on the sixth droplet forward): CRPS -3.1 [-12.1, +4.6] (59 cases, 9 clusters)
- flex_methanol.wsp5 D2-law (hand-written reference): CRPS +48.5 [+18.7, +82.0] (59 cases, 9 clusters)

**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet leaving and re-entering the view); the median and the mean without the worst 5 % of cases are given beside it.

| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |
|---|---|---|---|---|---|---|---|---|
| flex_methanol.wsp5 P1 (data-informed (train); physics-first laws, flame cameras read when the prefix carries them) | 182 | 5 | 28.7 | 17.5 | 19.0 | 36.0 | 0.78 | 0.95 |
| flex_methanol.wsp5 carry of D3 (the carried numbers on the sixth droplet forward) | 182 | 5 | 29.9 | 17.7 | 19.2 | 37.8 | 0.77 | 0.94 |
| flex_methanol.wsp5 D2-law (hand-written reference) | 182 | 5 | 87.2 | 37.9 | 59.7 | 112.9 | 0.49 | 0.77 |

### FLEX n-heptane, fifth generation (PSI-69)

Cohort `sha256:8ffa1875d0c963fe3b3ce6886ba7197a7b0c8675ffd837bbf5139049c1655797` · split `sha256:c43bbe3580ffce2a33c400d5aea73b3330cf721133f925c2963579dee6d32001`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| flex_heptane.wsp5 P1 — data-informed (train); physics-first laws, flame cameras read when the prefix carries them | `sha256:c27df7bb7f70bc3ce8cda22a1edc5c1b4a8b8c94b7760de61cedf901e40de360` | `sha256:c4924921…` | 109 / 603 |
| flex_heptane.wsp5 P0 — data-informed (train); the same laws, cameras not read | `sha256:c2c7b700867f3fab86e956100db6aed3db100438f6584347c9af5be6b75ad760` | `sha256:c4924921…` | 107 / 603 |
| flex_heptane.wsp5 carry of D2 — the carried numbers on the sixth droplet forward | `sha256:03e0f63cf88c1b2c626e7726356746dd16d673a3a208cca5461c9d1ec360bf0b` | `sha256:c4924921…` | 102 / 603 |
| flex_heptane.wsp5 D2-law — hand-written reference | `sha256:adac8c30e1907804928d876fc85ab0b84eec8d4fd786a063a8497e6cbabc5b33` | `sha256:c4924921…` | 42 / 603 |

**Validation** — 43 common cases; CRPS and MAE in µm.

| release | CRPS | MAE of median | coverage 50 % | coverage 90 % | lpd |
|---|---|---|---|---|---|
| flex_heptane.wsp5 carry of D2 (the carried numbers on the sixth droplet forward) | 46.1 | 63.0 | 0.71 | 0.97 | 8.25 |
| flex_heptane.wsp5 D2-law (hand-written reference) | 105.1 | 134.3 | 0.61 | 0.79 | 7.69 |
| flex_heptane.wsp5 P1 (data-informed (train); physics-first laws, flame cameras read when the prefix carries them) | 48.4 | 65.2 | 0.66 | 0.88 | 8.20 |
| flex_heptane.wsp5 P0 (data-informed (train); the same laws, cameras not read) | 47.2 | 65.6 | 0.69 | 0.92 | 8.28 |

Paired difference to flex_heptane.wsp5 P1 (data-informed (train); physics-first laws, flame cameras read when the prefix carries them) (other − reference, µm, 95 % CI):

- flex_heptane.wsp5 carry of D2 (the carried numbers on the sixth droplet forward): CRPS -2.3 [-7.3, +2.4] (39 cases, 7 clusters)
- flex_heptane.wsp5 D2-law (hand-written reference): CRPS +56.7 [+8.9, +133.2] (39 cases, 7 clusters)
- flex_heptane.wsp5 P0 (data-informed (train); the same laws, cameras not read): CRPS -1.2 [-17.0, +15.5] (39 cases, 7 clusters)

**Train** — µm. The mean is dominated by a few artefact runs (soot shell, gas bubble, droplet leaving and re-entering the view); the median and the mean without the worst 5 % of cases are given beside it.

| release | cases | refused | CRPS mean | CRPS median | CRPS mean, worst 5 % dropped | MAE of median | coverage 50 % | coverage 90 % |
|---|---|---|---|---|---|---|---|---|
| flex_heptane.wsp5 P1 (data-informed (train); physics-first laws, flame cameras read when the prefix carries them) | 125 | 9 | 44.6 | 21.0 | 22.8 | 51.8 | 0.75 | 0.92 |
| flex_heptane.wsp5 P0 (data-informed (train); the same laws, cameras not read) | 125 | 9 | 43.9 | 19.8 | 22.4 | 51.2 | 0.76 | 0.94 |
| flex_heptane.wsp5 carry of D2 (the carried numbers on the sixth droplet forward) | 125 | 9 | 46.2 | 21.8 | 22.6 | 54.1 | 0.76 | 0.93 |
| flex_heptane.wsp5 D2-law (hand-written reference) | 125 | 9 | 75.4 | 34.9 | 46.9 | 97.2 | 0.57 | 0.82 |


## Burner flames with the planned stop (private data root)

### ACME normal flames, fifth generation (PSI-159)

Cohort `sha256:92fb4728e88ed5175bd47da77707d042a0f046abca1bd8df351fd1d69fea4701` · split `sha256:c77bc5baee98d3e3494de144fa3f98ce0dee904efa35d5f7c0b48b91dbc82d14`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| acme_normal.wsp5 P1 — data-informed (train); the carried pair of laws with corrected burner-limit weights; reads the script's planned stop when the prefix carries it | `sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7` | `sha256:ead4023b…` | 275 / 2715 |
| acme_normal.wsp5 carry of wsp4 D1 — the carried model on the committee spec | `sha256:dce4d7f0b3ce75b6908d8be60313ecf7f7b94d0c25aefd79120af08a71f94a35` | `sha256:ead4023b…` | 202 / 2715 |
| acme_normal.wsp5 baseline set — the reference baselines, hand-written reference | `sha256:6044b69bd5d10a150734bea2b6d549a8903dbdc49ddc911f7ff1b6bf2c0b2bff` | `sha256:ead4023b…` | 134 / 2715 |

**Validation** — 84 common cases; integrated Brier score over 6–120 s (lower is better).

| release | extinction | operator termination |
|---|---|---|
| acme_normal.wsp5 baseline set (the reference baselines, hand-written reference) | 0.1809 | 0.0994 |
| acme_normal.wsp5 carry of wsp4 D1 (the carried model on the committee spec) | 0.1553 | 0.0984 |
| acme_normal.wsp5 P1 (data-informed (train); the carried pair of laws with corrected burner-limit weights; reads the script's planned stop when the prefix carries it) | 0.1452 | 0.0747 |

**Train** — integrated Brier score.

| release | cases | refused | extinction | operator termination |
|---|---|---|---|---|
| acme_normal.wsp5 P1 (data-informed (train); the carried pair of laws with corrected burner-limit weights; reads the script's planned stop when the prefix carries it) | 210 | 0 | 0.0536 | 0.0215 |
| acme_normal.wsp5 carry of wsp4 D1 (the carried model on the committee spec) | 210 | 0 | 0.0517 | 0.0275 |
| acme_normal.wsp5 baseline set (the reference baselines, hand-written reference) | 210 | 0 | 0.1507 | 0.0681 |

### ACME inverse flames, fifth generation (PSI-159)

Cohort `sha256:ed3e9beb717728fc4d371bccfc7896a940d38deea28502728be977f9fb79e45f` · split `sha256:747032c921c8c8f2b976fceb6bda8c90e4c5e18c761fdbd8dafeb5bc44e2b598`.

| release | id | forward | nonzero / authored |
|---|---|---|---|
| acme_inverse.wsp5 P1 — data-informed (train); the carried law (weight 2) and a limit-cycle scenario (weight 1); reads the script's planned stop when the prefix carries it | `sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2` | `sha256:ead4023b…` | 261 / 2715 |
| acme_inverse.wsp5 carry of wsp3 D1 — the carried model on the committee spec | `sha256:ff3449c4123f0ae5e1434f72b9a031588bb93a39a6514369624a906337035c26` | `sha256:ead4023b…` | 162 / 2715 |
| acme_inverse.wsp5 baseline set — the reference baselines, hand-written reference | `sha256:3d7d010fd8488d7071e756719f4bf8e7a2fb1f8a768179b063f55ec38d226424` | `sha256:ead4023b…` | 134 / 2715 |

**Validation** — 54 common cases; integrated Brier score over 6–120 s (lower is better).

| release | extinction | operator termination |
|---|---|---|
| acme_inverse.wsp5 baseline set (the reference baselines, hand-written reference) | 0.2556 | 0.1596 |
| acme_inverse.wsp5 P1 (data-informed (train); the carried law (weight 2) and a limit-cycle scenario (weight 1); reads the script's planned stop when the prefix carries it) | 0.1348 | 0.0478 |
| acme_inverse.wsp5 carry of wsp3 D1 (the carried model on the committee spec) | 0.1606 | 0.0939 |

**Train** — integrated Brier score.

| release | cases | refused | extinction | operator termination |
|---|---|---|---|---|
| acme_inverse.wsp5 P1 (data-informed (train); the carried law (weight 2) and a limit-cycle scenario (weight 1); reads the script's planned stop when the prefix carries it) | 204 | 0 | 0.0281 | 0.0341 |
| acme_inverse.wsp5 carry of wsp3 D1 (the carried model on the committee spec) | 204 | 0 | 0.0357 | 0.0639 |
| acme_inverse.wsp5 baseline set (the reference baselines, hand-written reference) | 204 | 0 | 0.3148 | 0.1975 |


## Burner flames without the planned stop (shared records)

Validation numbers of this section are the shared half of the evaluator's comparison, split by
`tools/wsp/analysis/cmp_split.py` (`runs/dev/gen5/cmp_split_acme_*_gen5.json`). The integrated Brier score
is the mean over cases of the evaluator's per-case contribution, as in every other table; the paired
difference is the evaluator's cluster-equal mean, bootstrapped by the script over clusters.

### ACME normal flames, fifth generation (PSI-159)

**Validation** — 84 cases of 28 runs; integrated Brier score over 6–120 s (lower is better).

| release | extinction | operator termination |
|---|---|---|
| acme_normal.wsp5 P1 | 0.1552 | 0.0968 |
| acme_normal.wsp5 carry of wsp4 D1 (the carried model) | 0.1553 | 0.0984 |
| acme_normal.wsp5 baseline set (hand-written reference) | 0.1809 | 0.0994 |

- acme_normal.wsp5 carry of wsp4 D1 (the carried model) minus acme_normal.wsp5 P1, extinction, cluster-equal: +0.0001 (2 clusters)
- acme_normal.wsp5 baseline set (hand-written reference) minus acme_normal.wsp5 P1, extinction, cluster-equal: +0.0261 (2 clusters)

**Train** — integrated Brier score.

| release | cases | refused | extinction | operator termination |
|---|---|---|---|---|
| acme_normal.wsp5 P1 (data-informed (train); the carried pair of laws with corrected burner-limit weights; reads the script's planned stop when the prefix carries it) | 210 | 0 | 0.0518 | 0.0274 |
| acme_normal.wsp5 carry of wsp4 D1 (the carried model on the committee spec) | 210 | 0 | 0.0517 | 0.0275 |
| acme_normal.wsp5 baseline set (the reference baselines, hand-written reference) | 210 | 0 | 0.1507 | 0.0681 |

### ACME inverse flames, fifth generation (PSI-159)

**Validation** — 54 cases of 21 runs; integrated Brier score over 6–120 s (lower is better).

| release | extinction | operator termination |
|---|---|---|
| acme_inverse.wsp5 P1 | 0.1288 | 0.0880 |
| acme_inverse.wsp5 carry of wsp3 D1 (the carried model) | 0.1606 | 0.0939 |
| acme_inverse.wsp5 baseline set (hand-written reference) | 0.2556 | 0.1596 |

- acme_inverse.wsp5 carry of wsp3 D1 (the carried model) minus acme_inverse.wsp5 P1, extinction, cluster-equal: +0.0306 (2 clusters)
- acme_inverse.wsp5 baseline set (hand-written reference) minus acme_inverse.wsp5 P1, extinction, cluster-equal: +0.1234 (2 clusters)

**Train** — integrated Brier score.

| release | cases | refused | extinction | operator termination |
|---|---|---|---|---|
| acme_inverse.wsp5 P1 (data-informed (train); the carried law (weight 2) and a limit-cycle scenario (weight 1); reads the script's planned stop when the prefix carries it) | 204 | 0 | 0.0271 | 0.0608 |
| acme_inverse.wsp5 carry of wsp3 D1 (the carried model on the committee spec) | 204 | 0 | 0.0357 | 0.0639 |
| acme_inverse.wsp5 baseline set (the reference baselines, hand-written reference) | 204 | 0 | 0.3148 | 0.1975 |

## Fabric flame (BASS, PSI-25)

**Validation** — 12 burns with cases, from 17 sealed tracks (not scoreable: 3 fewer than 6 flame stills, 1 flow unreadable, 1 position unit not sample width). Sample widths. Case = mean over its target stills, burn = mean over its cases, the table gives the mean over burns.

| release | CRPS | abs. error of median | coverage 50 % | coverage 90 % | track ended (Brier) | spread or not (Brier) |
|---|---|---|---|---|---|---|
| bass_front.wsp D1 (the carried model) | 0.1168 | 0.1615 | 0.66 | 0.90 | 0.1780 | 0.1703 |
| bass_front.wsp2 P1 (D1 with the narrow sample corrected) | 0.1137 | 0.1631 | 0.60 | 0.90 | 0.1976 | 0.1703 |

Paired difference to bass_front.wsp D1 (the carried model) (other − reference, sample widths, 95 % bootstrap over burns): CRPS -0.0031 [-0.0096, +0.0032] (40 cases, 12 burns; reference better in 1).

Train: the candidate equals D1 on the three train burns with cases (tip CRPS 0.076); on three burns of the
earlier campaign, which the narrow-sample powers were typed from, 0.157 against 0.220.
