# IGNIS model formulas

| | |
|---|---|
| **Scope** | The mathematical model behind every IGNIS release that forecasts a NASA microgravity combustion experiment: governing physics, every typed parameter with its meaning and value, how a forecast and its uncertainty are formed, and which numbers are zero-shot |
| **Audience** | Anyone who wants to check, re-derive or reproduce a forecast |
| **Related** | [COMPARISONS.md](COMPARISONS.md) (head-to-heads with published models and baselines) · [../wsp/PROTOCOL.md](../wsp/PROTOCOL.md) · [../wsp/cards/](../wsp/cards/) · [../REPRODUCIBILITY.md](../REPRODUCIBILITY.md) |

Every formula here was derived from the frozen forward that computes it, and each one cites the line
of code it comes from. Every value is a literal typed by the author model in the release's `edits.wsp`,
or the stored tensor of that release where an `edits.wsp` only lists changes from its parent.

## How an IGNIS model is built

IGNIS models are written by **weight-space programming**:

1. **The author types the weights.** The author model (Claude Fable, `claude-fable-5-2`) writes every
   parameter of a model as a literal assignment in an `edits.wsp` file, with a physical reason beside
   each line. An example is `rate.b0 = -0.53`, the log burning constant of methanol D3 (`configs/models/flex/wsp2/methanol/d3-fable/edits.wsp:24`).
2. **The tools compile and assemble it, without estimating anything.**
   - `ignis wsp compile` turns the edits into a `HandwrittenWeightPatch`.
   - `ignis wsp assemble` builds an immutable release. Its id is the sha256 of the release record.
3. **A frozen Rust forward runs the release.**
   - The forwards live in `crates/ignis-wsp/src/`, and every one declares
     `ForwardProfile::StrictNoFitting`. For example, `committee3.rs:301-305` and `front.rs:244`.
   - A call to `fit` bails.
   - No number is estimated, sampled or searched at request time.
   - Uncertainty is part of the typed model: spreads, scenario weights and quadrature nodes are typed
     numbers too.
4. **Most laws share one shape.** A law is a small hinge network,
   $N(x)=b_0+\mathrm{skip}\cdot x+\sum_j w_{2,j}\,\mathrm{relu}(b_{1,j}+W_{1,j,:}\cdot x)$
   (`crates/ignis-wsp/src/nn.rs:69-84`). The author writes each hinge as a physical statement: a
   threshold, a slope or a change of regime. Written out at its typed numbers, every law below is a
   closed-form expression.

Releases are `trained_on = not_trained` and `state = experimental`. Some releases say "Transcribed by
the assembly engineer (claude-opus-5-5); no number chosen, rounded or changed". In those, the author
wrote the numbers in a research dossier and a second model copied them verbatim into `edits.wsp`.

## Zero-shot and data-informed

- **Zero-shot:** typed before any outcome of that investigation was seen (the W0 candidates), and the
  reference baselines of each spec.
- **Data-informed:** revised after the author read the train partition, and in later generations
  also aggregate scores on validation days. With the exceptions each part states, every candidate other than W0 (D1–D5, P0–P3, F1–F4, C1, E1, S1 and so on) is data-informed.
- **What applies to both:** each candidate was committed before its held-out score existed, and that
  score decided only whether it replaced the model before it (`docs/wsp/PROTOCOL.md`, research repo).
  Each section below states the standing of each release.

## Path conventions

- **Unprefixed paths** are relative to the root of this repository (`ignis-web`).
- **The research repository** is `SpaceApps-Ardealu`, branch `feat/handwritten-science`, commit
  `542dad8e` (15 November 2026). Paths marked `research:`, `SpaceApps-Ardealu:` or *(research repo)*
  are there.
- **What is identical in both repositories:**
  - The forward sources in `crates/ignis-wsp/src/`, except `droplet10.rs`, which only the research repo
    has.
  - The release folders in `configs/models/`, except `configs/models/flex/wsp10`.
- **`models.rs`** differs between the two repositories. Its line numbers below are those of `ignis-web`.
- **A citation such as `droplet3.rs:1377-1381`** means `crates/ignis-wsp/src/droplet3.rs`, lines 1377 to
  1381.

## The releases at a glance

| Family | Adopted release | Release id | Forward | Standing |
|---|---|---|---|---|
| FLEX methanol droplets (PSI-69) | `flex_methanol.wsp2` D3 | `sha256:55e0cb12…43ce` | `droplet3.rs` | data-informed (train) |
| FLEX n-heptane droplets (PSI-69) | `flex_heptane.wsp2` D2 | `sha256:43f7871f…588d` | `droplet3.rs` | data-informed (train) |
| FLEX zero-shot references | `flex.wsp` W0, `flex_heptane.wsp` W0 | `sha256:fa8eb18c…e28f`, `sha256:6ea680d1…6d8f` | `flex.rs`, `droplet.rs` | zero-shot |
| CFI n-dodecane, FLEX-2 fuels (PSI-39, PSI-68) | `flex_<fuel>.wsp7` W0 | Part 1, section 5 | `droplet8.rs` | carries the heptane/methanol law (section 5.5) |
| ACME normal burner flames (CFI-G PSI-159, Flame Design PSI-10) | `acme_normal.wsp12` F2; since PROTOCOL 13.5 `acme_normal.wsp13` F3 stage 1 | `sha256:55fcf4cb…971d`; `sha256:d1660331…45aa` | `committee3.rs`; `committee4.rs` | data-informed |
| ACME inverse burner flames | `acme_inverse.wsp12` F2 | `sha256:dff7da0d…594e` | `committee3.rs` | data-informed; ethylene row zero-shot |
| s-Flame (PSI-23) | `acme_normal.wsp13` F3 stage 1, F4 rows | Part 2, section 2 | `committee4.rs` | data-informed |
| BASS-II fabric flame front (PSI-25) | `bass_front.wsp` D1 | `sha256:ac15e49f…855c` | `front.rs` | data-informed (four train burns, published theory) |
| Saffire I–III (PSI-98/99/100) | `bass_front.wsp` D1, unchanged | `sha256:ac15e49f…855c` | `front.rs` | blind transfer, zero weights changed |
| BRE, CLD, E-FIELD, SLICE, SPICE, SAME-R, SAME, DAFT, BASS plastics | one release each on the generic law forward | Part 4 | `law.rs` | per release, Part 4 |

## Contents

1. [FLEX droplet combustion](#part-1-flex-droplet-combustion): the $d^2$-law with a variable
   burning constant, hot / cool / no-flame states, diffusive and radiative extinction, the prefix
   encoder, and the scenario mixture.
2. [ACME spherical burner flames](#part-2-acme-spherical-burner-flames): the quasi-steady flame
   radius, the radiative-extinction radius, the erfc growth clock, the burner-heating law, competing
   log-logistic clocks, the committee of member laws, and the planned stop.
3. [BASS flame front and the Saffire transfer](#part-3-bass-flame-front-and-the-saffire-transfer):
   the concurrent-flow spread law, flame overhang, burn-through, the piecewise-exponential quench
   hazard, the run offset from the prefix, and the transfer to 40.6 cm sheets.
4. [The generic law forward](#part-4-the-generic-law-forward-all-other-investigations): the
   declarations, log-normal mixture targets, binary targets, competing log-logistic event clocks, and
   the typed law of each remaining investigation, with the published-law baselines (Lin and Faeth,
   Bhattacharjee).
5. [How to reproduce a forecast](#part-5-how-to-reproduce-a-forecast): build, rebuild the releases with identical ids, inspect the typed numbers, and run a forward.

## Forecasts and uncertainty, in one paragraph per kind

- **Continuous trajectories** (droplet diameter, flame-tip position). The forward integrates the law
  over a set of equally weighted scenario nodes, and sometimes over discrete states as well. Each node
  is a quantile of a typed spread, as in $\ln K \to \ln K + s_z z_k$. The forward reports quantiles
  0.05, 0.25, 0.5, 0.75 and 0.95 of the resulting mixture at each forecast time.
  - *Droplets:* cutoffs 1, 3 and 5 s, horizon 10 s, scored by CRPS.
  - *Fabric:* cutoffs 5, 10, 20 and 40 s, scored by CRPS in sample widths.
- **Event times** (burner-flame extinction against operator termination; the generic-law events). The
  forward combines competing log-logistic or exponential clocks, plus point masses for planned stops,
  into a cumulative incidence per cause. It reports these at the scoring times (6 to 120 s for burner
  flames), which are scored with the IPCW integrated Brier score.
- **Binary outcomes** (kept burning, blow-off, detector at full scale). A logistic of the law, scored
  by Brier.
- **Positive quantities** in the generic forward (smoke-point length, quench speed, particle
  diameter, ion current). A typed two-component log-normal mixture around the law's median, scored by
  a quantile score or CRPS.



## Part 1. FLEX droplet combustion

All paths are relative to the root of `ignis-web` unless prefixed `SpaceApps-Ardealu:`. The forward files `crates/ignis-wsp/src/*.rs` cited here are byte-identical in both repositories (checked for `droplet3.rs`, `flex.rs`, `common.rs`). Every number in a parameter table is a literal typed in the release's `edits.wsp`. Nothing is fitted: a release is its typed tensors (`spec.json` + `edits.wsp` → `patch.json` → `release.json`) plus a frozen forward.

### 0. Notation and shared kernels

| Symbol | Definition | Code |
|---|---|---|
| $q$ | $D^2$, liquid diameter squared (m²) | droplet3.rs:942 |
| $q_0$ | $D_0^2$, back-extrapolated to the time origin (section 1.6) | droplet3.rs:999-1001 |
| $w$ | burn progress, $\mathrm{clamp}(1-q/q_0,\,0,\,1)$ | droplet3.rs:696, 1361 |
| $s$ | diffusion clock, $t\,K_{ref}/q_0$, with $t$ in s since the time origin (igniter energised) | droplet3.rs:697, 1362 |
| $K_{ref}$ | $10^{-6}$ m²/s (`norm.k_ref_m2_s`) | droplet3.rs:393 |
| $D_{ref}$ | 2 mm (`norm.d_ref_m`) | droplet3.rs:391 |
| $p_{ref}$ | $10^5$ Pa (`norm.p_ref_pa`) | droplet3.rs:392 |

The condition vector $c$ is built at droplet3.rs:975-981 and 999-1001:

$$c=\big(c_{O_2},c_{CO_2},c_{He},c_{\log p},c_{\log D_0}\big)=\Big(\tfrac{x_{O_2}-0.21}{0.05},\ \tfrac{x_{CO_2}}{0.20},\ \tfrac{x_{He}}{0.20},\ \ln\tfrac{p}{p_{ref}},\ \ln\tfrac{\sqrt{q_0}}{D_{ref}}\Big)$$

Every law is a one-hidden-layer ReLU network with a linear skip path. The author writes the hinges; any hidden unit the patch does not write is zero (`sparse`, spec.rs:46-47). The kernel is at nn.rs:69-84:

$$N(x)=b_0+\mathrm{skip}\cdot x+\sum_j w_{2,j}\,\mathrm{relu}\big(b_{1,j}+W_{1,j,:}\cdot x\big),\qquad \mathrm{relu}(u)=\max(u,0)$$

Hidden widths: 16 for the rate network and 8 for every other law (droplet3.rs:86-88). The logistic $\sigma(u)=1/(1+e^{-u})$ is at nn.rs:44-51.

**Releases covered**

| Model | Candidate | Release id | Forward | Zero-shot? |
|---|---|---|---|---|
| `flex_methanol.wsp2` | D3 (`configs/models/flex/wsp2/methanol/d3-fable`) | `sha256:55e0cb12…43ce` | `droplet3.rs` | no, data-informed (train) |
| `flex_heptane.wsp2` | D2 (`configs/models/flex/wsp2/heptane/d2-fable`) | `sha256:43f7871f…588d` | `droplet3.rs` | no, data-informed (train) |
| `flex.wsp` (methanol) | W0 (`configs/models/flex/wsp/w0-fable`) | `sha256:fa8eb18c…e28f` | `flex.rs` | yes |
| `flex_heptane.wsp` | W0 (`configs/models/flex/wsp/heptane/w0-fable`) | `sha256:6ea680d1…6d8f` | `droplet.rs` | yes |
| `flex_methanol.wsp2` | baseline-d2 | `sha256:abf919b6…247b` | `droplet3.rs` | reference baseline |
| `flex_heptane.wsp2` | baseline-d2 | `sha256:8e40db9d…ebb1` | `droplet3.rs` | reference baseline |
| `flex_<fuel>.wsp7` | W0, six fuels | section 5 | `droplet8.rs` | see section 5.5 |
| `flex_heptane.wsp10` | D4 | `sha256:f3ccc122…b164` | SpaceApps-Ardealu:`droplet10.rs` | no, data-informed |

Model key → forward: crates/ignis-wsp/src/models.rs:131 (`flex` → `Forward::Flex1`), :146-153 (`flex_methanol`, `flex_heptane` → `droplet.rs`), :170-179 (`flex_methanol2`, `flex_heptane2` → `Forward::Droplet3`), and :404 onward (`flex_<fuel>7` → `Forward::Droplet8`).

---

### 1. Adopted releases: `flex_methanol.wsp2` D3 and `flex_heptane.wsp2` D2 (`droplet3.rs`)

The forward is shared and the weights are not. Methanol has no cool flame: `has_cool` is false (droplet3.rs:131-133), so its `cool.*` tensors do not exist, and $p_c=K_c=D_{ce}=0$ (droplet3.rs:721).

A free droplet is in one of three states: **hot flame**, **cool flame** (n-heptane only) or **no flame**. The forward carries a probability for each state through a mixture of closed-form and RK4 paths.

#### 1.1 Hot flame: the quasi-steady $d^2$-law with a variable burning constant

The population rate is at droplet3.rs:676-685. The run's path, at RK4 node $z_k$, is at droplet3.rs:1377-1381:

$$\frac{dq}{dt}=-K_h,\qquad K_h=K_{ref}\exp\!\big(N_{rate}(c,w,s)+b+s_z z_k\big),\qquad x_{rate}=(c,\,w,\,s)\in\mathbb R^7$$

With $N_{rate}$ constant this is the classical law $D^2=D_0^2-Kt$. Here the burning constant depends on the atmosphere, on the progress $w$ (methanol absorbing water, a large droplet's flame still developing) and on the clock $s$ (heat-up). $b$ is the run's log-rate offset from the prefix (1.6), and $s_z z_k$ is the scenario spread (1.8).

- **Integration.** Fixed-step classical RK4 with $h$ = `num.dt_s` = 0.05 s on a uniform grid from the cutoff $\tau$. The grid has $n_{grid}=\max(1,\lceil (t_{end}-\tau)/h-10^{-9}\rceil)$ steps (droplet3.rs:1338, 1387-1398).
- **Floor.** Once $q\le q_{min}=d_{min}^2$, with $d_{min}$ = `num.d_min_m` = 0.25 mm, $q$ is held at $q_{min}$ for the rest of the path (droplet3.rs:1399-1404).
- **Forecast times** are linearly interpolated on the grid (droplet3.rs:1473-1477).

Typed rate laws, written out from the hinge units:

**Methanol D3**

$$\ln\frac{K_h}{K_{ref}}=-0.53+0.14c_{O_2}-0.04c_{CO_2}+0.18c_{He}+0.10c_{\log p}-0.35w-0.19\,\mathrm{relu}(-c_{O_2})+0.14\,\mathrm{relu}(-0.6-c_{O_2})$$
$$\qquad-0.5\,\mathrm{relu}(-0.3+c_{\log D_0}-0.65w)-0.6\,\mathrm{relu}(1-5.56s)-0.55\,\mathrm{relu}(-0.9-c_{O_2}-2s)$$

**n-Heptane D2**

$$\ln\frac{K_h}{K_{ref}}=-0.48+0.13c_{O_2}-0.08c_{CO_2}+0.15c_{He}+0.05c_{\log p}-0.15c_{\log D_0}-0.12\,\mathrm{relu}(-c_{O_2})-0.05\,\mathrm{relu}(-0.7-c_{O_2})$$
$$\qquad-0.05\,\mathrm{relu}(c_{He}-1.5)-0.3\,\mathrm{relu}(-0.4+c_{\log D_0}-0.65w)+0.11\,\mathrm{relu}(1-3.33s)-0.3\,\mathrm{relu}(1-1.667s)$$

At $c=0$ (air, 1 atm, $D_0$ = 2 mm), computed here from the typed numbers:

| Fuel | Plateau $K$ | $K$ at $s=0$ | Heat-up ends at |
|---|---|---|---|
| methanol | $e^{-0.53}$ = 0.589 mm²/s | $e^{-1.13}$ = 0.323 mm²/s | $s=0.18$ |
| n-heptane | $e^{-0.48}$ = 0.619 mm²/s | $e^{-0.67}$ = 0.512 mm²/s | $s=0.6$ |

For methanol the plateau also falls by the $-0.35w$ term as the burn progresses.

#### 1.2 Critical diameter and the two extinction classes

The laws are at droplet3.rs:707-734:

$$\ln\frac{D_{crit}}{D_{ref}}=N_{crit}(c_{O_2},c_{CO_2},c_{He},c_{\log p}),\qquad x_{exc}=c_{\log D_0}-\ln\frac{D_{crit}}{D_{ref}}=\ln\frac{D_0}{D_{crit}}$$

$$\pi_R=P(\text{radiative class})=\sigma(\kappa\,x_{exc}+\beta)$$

- **Below $D_{crit}$:** a droplet burns until its own history ends the flame. This is decided by **progress**: the diffusive class.
- **Above $D_{crit}$:** the flame grows until radiation quenches it. This is decided by the **clock**: the radiative class.

The two classes are mutually exclusive and are mixed with $\pi_R$.

Critical-diameter laws (both have $W_1=0$):

| Fuel | Law | $\kappa$ | $\beta$ | $D_{crit}$ in air, 1 atm |
|---|---|---|---|---|
| methanol D3 | $\ln(D_{crit}/2\,\mathrm{mm})=1.51+0.75c_{O_2}-0.16c_{CO_2}-0.18c_{He}+0.7c_{\log p}$ | 10 | 0.7 | 9.05 mm (3.68 mm at 15 % O₂) |
| heptane D2 | $\ln(D_{crit}/2\,\mathrm{mm})=0.615+0.62c_{O_2}-0.45c_{CO_2}-0.08c_{He}$ | 14 | 0.5 | 3.70 mm |

#### 1.3 Diffusive extinction (progress law, Gompertz in $w$)

droplet3.rs:712 and 736-740:

$$w_D=\sigma\big(N_{diff}(c,x_{exc})\big),\qquad \Lambda_D(w)=\ln 2\,\Big(e^{a_D(w-w_D)}-e^{-a_D w_D}\Big),\qquad S_D(w)=e^{-\Lambda_D(w)}$$

$\Lambda_D(0)=0$ and $\Lambda_D(w_D)=\ln 2\,(1-e^{-a_Dw_D})\approx\ln 2$, so $w_D$ is the median progress at extinction and $a_D$ its sharpness.

| Fuel | $\mathrm{logit}\,w_D$ | $a_D$ | Air, $D_0$ = 2 mm (computed) |
|---|---|---|---|
| methanol D3 | $1.35+0.12c_{O_2}-0.6c_{\log p}+0.5\,\mathrm{relu}(c_{O_2}-1)-2.0\,\mathrm{relu}(x_{exc}+0.3)$ | 14 | $w_D$ = 0.794, $D/D_0$ = 0.454 |
| heptane D2 | $2.44+0.5c_{O_2}-0.3c_{CO_2}+0.5c_{\log D_0}-0.5\,\mathrm{relu}(c_{O_2})-1.0\,\mathrm{relu}(x_{exc}+0.3)$ | 10 | $w_D$ = 0.920, $D/D_0$ = 0.283 |

#### 1.4 Radiative extinction (clock law, Gompertz in $s/s_R$)

droplet3.rs:713 and 742-746:

$$s_R=\exp\big(N_{rad}(c,x_{exc})\big),\qquad \Lambda_R(s)=\ln 2\,\Big(e^{a_R(s/s_R-1)}-e^{-a_R}\Big),\qquad S_R(s)=e^{-\Lambda_R(s)}$$

| Fuel | $\ln s_R$ | $a_R$ |
|---|---|---|
| methanol D3 | $-0.22c_{He}-3.8\,\mathrm{relu}(x_{exc}-0.05)$ | 5 |
| heptane D2 | $0.227-2.0\,\mathrm{relu}(x_{exc})-1.0\,\mathrm{relu}(c_{\log D_0}-0.14)$ | 4 |

For methanol, $s_R=1$ without helium up to $x_{exc}=0.05$ and collapses above it. For heptane, a droplet at $D_0=D_{crit}$ = 3.7 mm in air has $s_R=e^{0.227-0.475}$ = 0.78 (computed).

The hot flame has no reignition and no other end. There is also a constant **disruption hazard** $\lambda_{dis}=e^{\texttt{haz.log\_disruption}}$ (droplet3.rs:651). It is reported in the events and never changes the diameter: methanol $e^{-4.83}$ = 0.0080 /s, heptane $e^{-3.9}$ = 0.0202 /s.

#### 1.5 Cool flame (n-heptane) and the unlit droplet

**Cool flame.** It occurs only after a *radiative* extinction, never after a diffusive one ($p_c$ is set to 0 for diffusive bins, droplet3.rs:1499). The laws are at droplet3.rs:714-720:

$$p_c=\sigma\big(N_{cool,p}(c,x_{exc})\big),\qquad K_c=K_{ref}\,e^{N_{cool,k}(c)},\qquad \ln\frac{D_{ce}}{D_{ref}}=N_{cool,d}(c)+sd_{ce}\,u$$

The burning law, the closed form at droplet3.rs:1435-1445:

$$q(t)=\max\big(q_e-K_c(t-t_e),\,q_{min}\big)\quad\text{for } t\le t_x=t_e+\frac{q_e-q_x}{K_c},\qquad q_x=D_{ce}^2$$

After $t_x$ the droplet is unlit from $(t_x,q_x)$, with the post law at scale $\lambda_{cool}$ = `post.cool_scale`. If $q_x\ge q_e$ or $K_c\le0$, the cool flame has zero length and the droplet is unlit at once with scale $\lambda_{cool}$ (droplet3.rs:1436-1438).

The scenario rate multiplies the cool rate:
- after a radiative extinction inside the forecast, $K_c\to K_c\,e^{b+s_zz_k}$, the *hot* run offset (droplet3.rs:1485, 1504);
- for a cool flame already burning at the cutoff, $K_c\to K_c\,e^{b_{cool}+s_zz_k}$ (droplet3.rs:1556).

$D_{ce}$ takes 5 nodes $u=-2,\dots,2$ (step `num.ce_step` = 1) with weights $\propto e^{-u^2/2}$ (droplet3.rs:1343-1360).

Typed cool-flame laws, heptane D2:

$$\mathrm{logit}\,p_c=3.0-7.5\,\mathrm{relu}(-1-c_{O_2})-7.0\,\mathrm{relu}(c_{He}-0.75)$$
$$\ln\frac{K_c}{K_{ref}}=-0.97+0.2c_{O_2}-0.05c_{CO_2}+0.14c_{He}+0.1c_{\log p}+0.2c_{\log D_0}$$
$$\ln\frac{D_{ce}}{2\,\mathrm{mm}}=-0.77-0.25c_{O_2}+0.05c_{CO_2}+0.2c_{He}-1.0c_{\log p}+0.8c_{\log D_0},\qquad sd_{ce}=0.12$$

Computed here: $p_c$ = 0.953 in air, 0.818 at 15 % O₂ and 0.5 at 14 % O₂. At $c=0$, $K_c$ = 0.379 mm²/s and $D_{ce}$ = 0.93 mm.

**No flame.** A droplet unlit since $(t_e,q_e)$ follows a long-time rate plus a decaying transient from the flame's heat (droplet3.rs:748-768, 1431-1433). With $r=q_e/D_{ref}^2=(D_e/2\,\mathrm{mm})^2$ and $u=\max(t-t_e,0)$:

$$q(t)=\max\Big(q_e-\big[k_\infty r^{m}\,u+\lambda\,k_{hot}r^{h}\,\tau_e\big(1-e^{-u/\tau_e}\big)\big],\ q_{min}\Big),\qquad \tau_e=\max\big(\tau_0 r^{n},\,10^{-3}\,\mathrm{s}\big)$$

This is the integral of $-dq/dt=k_\infty r^m+\lambda k_{hot}r^h e^{-u/\tau_e}$. The scale $\lambda$ depends on the path:

| Path | $\lambda$ |
|---|---|
| after a hot flame | 1 |
| after a cool flame | `post.cool_scale` |
| unlit at the cutoff | $\lambda_{now}$, section 1.7 |

#### 1.6 What is read from the prefix

All of this is at droplet3.rs:910-1215.

**1. Admission.** The forward keeps liquid-diameter observations that are valid, positive and inside $0\le t\le\tau$. Of equal time stamps it keeps the first, and it refuses an unsorted prefix (droplet3.rs:922-943). It requires the modelled phase to be established at the cutoff (droplet3.rs:915-920).

**2. Validity gate.** An observation $(t_i,q_i)$ is dropped if at least `gate.min_later` = 3 later values have $q_j>q_i(1+0.05)$, or at least 3 later values have $q_j<q_i(1-0.05)-k_{max}(t_j-t_i)$. The two counts are kept separately, and $k_{max}$ = 2e-6 m²/s (droplet3.rs:871-901). Fewer than 2 kept points is a refusal (droplet3.rs:945-954).

**3. Windows.** Let $t_1,t_n$ be the first and last kept times and $\Delta=t_n-t_1$. With $f=0.25$:
- head $[t_1,t_1+f\Delta]$;
- mid $[t_1+\tfrac{1-f}{2}\Delta,\ t_1+\tfrac{1+f}{2}\Delta]$;
- tail $[t_n-f\Delta,t_n]$;
- end windows $[t_n-2f_e\Delta,\ t_n-f_e\Delta]$ and $[t_n-f_e\Delta,\ t_n]$, with $f_e$ = 0.15.

Each window gives its mean time $\bar t$ and mean $\bar q$ (droplet3.rs:850-865, 983-996).

**4. Initial diameter**, one pass (droplet3.rs:999-1001). With $c_{\log D_0}$ first set from $\bar q_{head}$:

$$q_0=\bar q_{head}+K_{ref}e^{N_{rate}(c,0,0)}\,\bar t_{head}$$

The conditions $c$ must lie in the spec domain or the forecast is refused: $c_{O_2}\in[-2.2,4.8]$, $c_{CO_2}\in[0,4]$, $c_{He}\in[0,3]$, $c_{\log p}\in[-1,1.5]$, $c_{\log D_0}\in[-0.9,1.26]$ (droplet3.rs:457-463, 1004-1019).

**5. Innovations.** For two windows $a\to b$ (droplet3.rs:688-704, 1021-1037):

$$K_{obs}=\frac{\bar q_a-\bar q_b}{\bar t_b-\bar t_a},\qquad K_{mod}=\Big[\tfrac16\Big(\tfrac{1}{K_{pop}(w_a,s_a)}+\tfrac{4}{K_{pop}(\bar w,\bar s)}+\tfrac{1}{K_{pop}(w_b,s_b)}\Big)\Big]^{-1}$$

$$e=\mathrm{clamp}\Big(\ln\frac{\max(K_{obs},k_{floor})}{K_{mod}},\,-e_{lo},\,e_{hi}\Big),\qquad \nu=\frac{2\sqrt{\bar q_b}\,\sigma_{frame}\sqrt{1/n_a+1/n_b}}{(\bar t_b-\bar t_a)\,K_{mod}}$$

- $K_{pop}$ is the population hot rate without $b$.
- $k_{floor}$ = 2e-8 m²/s.
- $\nu$ is the frame-noise sd of the ratio $K_{obs}/K_{mod}$.
- $e_{early}$ uses head→mid and $e_{late}$ uses mid→tail. Both use head→tail when the mid window is empty or not ordered (droplet3.rs:1038-1044).
- The end rate $K_{end}$, with $K_{mod,end}$ and $\nu_{end}$, is the innovation between the two end windows. If either end window is empty, the late values are used instead (droplet3.rs:1045-1050).

**6. Gains.** Let $g=(\ln(\bar t_{tail}-\bar t_{head}),\ \ln(n/32),\ \mathrm{clamp}(w_{tail}/0.08,0,1))$ (droplet3.rs:1056-1079). Then:

$$\gamma=\sigma(\texttt{enc.gain\_b}+\texttt{enc.gain\_w}\cdot g),\qquad g_s=\sigma(\texttt{filt.gain\_b}+\texttt{filt.gain\_w}\cdot g)$$

Both are 0 if $\bar t_{tail}\le\bar t_{head}$.

**7. Run offset and scenario spread** (droplet3.rs:1080-1081):

$$b=\gamma\,(m_0e_{late}+m_1e_{early}),\qquad s_z=\sqrt{s_{floor}^2+(1-\gamma)\,s_{pop}^2}$$

**8. Level at the cutoff (anchor).** The last end window is carried to $\tau$ along the end rate, clamped (droplet3.rs:1085-1095). No weight enters it except through the cap:

$$q_{cut}=\bar q_{end}-\mathrm{clamp}\big(K_{end},\,0,\,\min(2\max(K_{late},k_{floor}),\,k_{max})\big)\,(\tau-\bar t_{end})$$

- If $K_{late}$ is not finite, the cap is $k_{max}$; if $K_{end}$ is not finite, the slope is 0.
- If the end windows are unused, the tail window is carried instead.
- The forecast is refused if $q_{cut}\le q_{min}$ or $\tau\ge$ `num.t_valid_s` = 45 s (droplet3.rs:1096-1111).

**9. Prior state at the cutoff**, from the laws at the observed $w_{cut}=w(q_{cut})$ and $s_{cut}=\tau K_{ref}/q_0$ (droplet3.rs:1113-1129):

$$P_{hot}=\pi_R S_R(s_{cut})+(1-\pi_R)S_D(w_{cut}),\qquad P_{cool}=\pi_R\big(1-S_R(s_{cut})\big)\,p_c\,\Phi\Big(\frac{\ln\sqrt{q_{cut}}-\ln D_{ce}}{sd_{ce}}\Big),\qquad P_{none}=\max(1-P_{hot}-P_{cool},0)$$

When $sd_{ce}=0$, $\Phi(\cdot)$ is replaced by $\mathbb 1[\sqrt{q_{cut}}>D_{ce}]$.

**10. State filter on the end rate.** It runs only if $g_s>0$ and $K_{end}$, $K_{mod,end}>0$ are finite (droplet3.rs:1132-1157). With $r=K_{end}/K_{mod,end}$:

$$P_i\propto \text{prior}_i\Big[\mathcal N\big(r;\mu_i,\sqrt{\sigma_i^2+\nu_{end}^2}\big)+l_{floor}\Big]^{g_s}$$

| State | $\mu_i$ | $\sigma_i$ |
|---|---|---|
| hot | 1 | `filt.sd_hot` |
| cool | $\rho_c=K_c/K_{mod,end}$ | `filt.sd_hot`$\cdot\max(\rho_c,10^{-6})$ |
| no flame | $\rho_e=(k_\infty r_c^m+\tfrac12k_{hot}r_c^h)/K_{mod,end}$, with $r_c=q_{cut}/D_{ref}^2$ | `filt.sd_evap` |

The combined sd is floored at $10^{-9}$. If the normaliser is not positive and finite, the prior is kept.

**11. Cool-flame offset** (droplet3.rs:1158-1164), set only when $K_c>0$:

$$b_{cool}=\texttt{enc.cool\_gain}\cdot g_s\cdot\mathrm{clamp}\big(\ln(\max(K_{end},k_{floor})/K_c),-e_{lo},e_{hi}\big)$$

**12. Class given hot** (droplet3.rs:1166-1170): $\rho_R=\pi_RS_R(s_{cut})/P_{hot}$, or $\pi_R$ if $P_{hot}=0$.

Computed here for the gains:

| Fuel | $\gamma$ at $\bar t_{tail}-\bar t_{head}$ = 0.5 / 1 / 2 / 4 s |
|---|---|
| methanol D3, $\gamma=\sigma(-1.6+1.4\ln\Delta\bar t)$ | 0.07 / 0.17 / 0.35 / 0.58 |
| heptane D2, $\gamma=\sigma(-1.8+0.8\ln\Delta\bar t)$ | 0.09 / 0.14 / 0.22 / 0.33 |

The edits comments state 0.08 / 0.36 / 0.55 (methanol) and 0.09 / 0.23 / 0.31 (heptane) at cutoffs 1 / 3 / 5 s. The exact $\Delta\bar t$ these correspond to is not determined from the code.

#### 1.7 Forecast distribution: mixture over scenarios, states and transition times

**Scenario nodes.** $K$ = `num.z_count` = 15 nodes $z_k=(k-7)\cdot0.5$, $k=0..14$, with weights $\omega_k=e^{-z_k^2/2}/\sum_j e^{-z_j^2/2}$ (droplet3.rs:1343-1352). This is a deterministic quadrature of $z\sim\mathcal N(0,1)$ truncated at $\pm3.5$. It is not Monte Carlo, and quad.rs's Gauss–Hermite rule is not used here; only `normal_cdf` from quad.rs:160-162 is.

**Hot path per node.** One RK4 path $q_k(t)$ per node serves both classes (section 1.1). The conditional survivals since the cutoff are (droplet3.rs:1363-1373, 1405, 1418):

$$\tilde S_{D,k}(t_i)=\min\Big(\tilde S_{D,k}(t_{i-1}),\ \frac{e^{-\Lambda_D(w(q_k(t_i)))}}{\max(S_D(w_{cut}),10^{-300})}\Big),\qquad \tilde S_R(t_i)=\frac{e^{-\Lambda_R(s(t_i))}}{\max(S_R(s_{cut}),10^{-300})}$$

**Extinction bins.** Extinction mass per step, $\tilde S(t_i)-\tilde S(t_{i+1})$ floored at 0, is collected in bins of `num.bin_s` = 0.5 s (10 steps). Each bin carries its mass-weighted mean time and mean $q$, taken at step midpoints (droplet3.rs:1406-1428). The bin in progress at a forecast time is emitted with its mass up to now, at time $\tfrac12(t_{start}+t)$ and level $\tfrac12(q_{start}+q_k(t))$ (droplet3.rs:1519-1544).

**Atoms at forecast time $t$** (diameters $\sqrt q$, droplet3.rs:1470-1581):

| Atom | Weight | Diameter |
|---|---|---|
| still hot | $P_{hot}\,\omega_k\,[(1-\rho_R)\tilde S_{D,k}(t)+\rho_R\tilde S_R(t)]$ | $\sqrt{q_k(t)}$ |
| diffusive bin $\beta$ | $P_{hot}\,\omega_k\,(1-\rho_R)\,m^D_{k\beta}$ | unlit from $(t_\beta,q_\beta)$, $\lambda=1$ |
| radiative bin, no cool flame | $P_{hot}\,\omega_k\,\rho_R\,m^R_{\beta}(1-p_c)$ | unlit from $(t_\beta,q_\beta)$, $\lambda=1$ |
| radiative bin, cool flame, node $u$ | $P_{hot}\,\omega_k\,\rho_R\,m^R_{\beta}\,p_c\,\omega^{ce}_u$ | cool law from $(t_\beta,q_\beta)$ with $K_ce^{b+s_zz_k}$, $q_x=q_{ce,u}$ |
| cool at cutoff | $P_{cool}\,\omega_k\,\omega^{ce}_u/\sum_{u':q_{ce,u'}<q_{cut}}\omega^{ce}_{u'}$, for nodes with $q_{ce,u}<q_{cut}$ | cool law from $(\tau,q_{cut})$ with $K_ce^{b_{cool}+s_zz_k}$ |
| cool at cutoff, no node qualifies | $P_{cool}\,\omega_k$ | unlit from $(\tau,q_{cut})$ with $\lambda_{cool}$ |
| unlit at cutoff | $P_{none}$ | unlit from $(\tau,q_{cut})$ with $\lambda_{now}$ |

Atoms of weight $\le10^{-12}$ are skipped. The scale for a droplet unlit at the cutoff is (droplet3.rs:1450-1457):

$$\lambda_{now}=g_s\,\mathrm{clamp}\Big(\frac{K_{end}-k_\infty r_c^m}{\max(k_{hot}r_c^h,10^{-12})},0,3\Big)+\frac{1-g_s}{2}$$

It falls back to 0.5 if $k_{hot}=0$ or $K_{end}$ is not finite.

**Outputs at each $t$** (droplet3.rs:1583-1597):
- **Mean:** $\sum_a D_aW_a/\sum_aW_a$.
- **Latent quantiles:** step quantiles, the smallest atom with cumulative weight $\ge p\sum W$ (droplet3.rs:1219-1233).
- **Measurement quantiles.** The atoms are pooled on cells of $\sigma_d/4$, starting from the smallest atom, keeping mass and mean. The predictive CDF of one measurement is

$$F(x)=\frac{1}{\sum W}\sum_{\text{cells}}W_c\,\Phi\Big(\frac{x-\bar D_c}{\sigma_d}\Big)$$

  with $\Phi=1$ beyond $+8\sigma$ and 0 below $-8\sigma$. Each quantile is found by 44 bisections on $[\bar D_{min}-8\sigma_d,\ \bar D_{max}+8\sigma_d]$ and floored at 0 (droplet3.rs:1239-1298, 1591-1594). This inverts the mixture; it involves no fitted spread.
- **State probabilities:** hot $=\sum(\text{hot atoms})/\sum W$, and cool $=$ the weight of atoms still inside a cool flame.

**Events.** These are competing risks for a droplet hot at the cutoff (droplet3.rs:1605-1622). Let $A(t)=P_{hot}(t)\,e^{-\lambda_{dis}(t-\tau)}$. The disruption incidence $C$ is the trapezoid integral of $\lambda_{dis}A$, with $C(t_0)=P_{hot}(t_0)-A(t_0)$. Then:

$$E(t)=\mathrm{clamp}\big(\max(E_{prev},\,1-A-C),0,1\big),\qquad \text{disruption}=\min(C,1-E),\qquad \text{active}=\max(1-E-\text{disruption},0)$$

The event rows are `active_supported`, `hot_phase_end` $=E$, `disruption` and `unresolved` $=0$ (event categories at droplet3.rs:1734-1739; `FLEX_CAUSES` at crates/ignis-core/src/views.rs:16).

**Emitted grid.** The default quantile levels are $(0.05,0.25,0.5,0.75,0.95)$ (common.rs:14, 65-71). The default times are 41 equally spaced points from $\tau$ to $\min(\tau+10\,\mathrm{s},45\,\mathrm{s})$ (common.rs:75-96, droplet3.rs:1716). Requested times outside $[\tau,45]$ are refused (droplet3.rs:1717-1730). The uncertainty kind is `assumed_noise` (droplet3.rs:1748): the spread is an assumed scenario distribution, not a calibrated posterior.

#### 1.8 Parameter table (`edits.wsp` of D3 and D2)

Both releases are literal revisions of their baseline-d2 root. That root has no hidden units and every authored scalar is rewritten, so the effective values are those typed in the edits (assembly order: zero-init → spec constants → parent tensors → patch, crates/ignis-wsp/src/assemble.rs:7-8, 82-109).

**Laws**

| Symbol | Name in edits.wsp | Meaning | Methanol D3 | Heptane D2 | Unit |
|---|---|---|---|---|---|
| — | `rate.*` | hot burning-rate law | 1.1 | 1.1 | ln(K/1e-6 m²/s) |
| — | `crit.b0`, `crit.skip` | critical-diameter law | 1.51; (0.75, −0.16, −0.18, 0.7) | 0.615; (0.62, −0.45, −0.08, 0) | ln(D/2 mm) |
| $\kappa$, $\beta$ | `class.kappa`, `class.bias` | sharpness and logit of the radiative class at $D_0=D_{crit}$ | 10, 0.7 | 14, 0.5 | 1 |
| — | `diff.*` | median progress at diffusive extinction | 1.3 | 1.3 | logit |
| $a_D$ | `diff.sharp` | Gompertz sharpness in $w$ | 14 | 10 | 1 |
| — | `rad.*` | median clock at radiative extinction | 1.4 | 1.4 | ln |
| $a_R$ | `rad.sharp` | Gompertz sharpness in $s/s_R$ | 5 | 4 | 1 |
| — | `cool.p.*`, `cool.k.*`, `cool.d.*` | cool-flame onset, rate, end diameter | — | 1.5 | logit; ln K; ln D |
| $sd_{ce}$ | `cool.d.sd` | sd of $\ln D_{ce}$ | — | 0.12 | 1 |
| $\lambda_{cool}$ | `post.cool_scale` | $k_{hot}$ multiplier after a cool flame | — | 1.0 | 1 |

**Unlit droplet**

| Symbol | Name in edits.wsp | Meaning | Methanol D3 | Heptane D2 | Unit |
|---|---|---|---|---|---|
| $k_\infty$ | `post.k_inf` | long-time evaporation rate at $D_e=D_{ref}$ | 1.5e-8 | 3.0e-8 | m²/s |
| $m$ | `post.size_exp` | exponent of $r$ in $k_\infty$ | 0.72 | 0.3 | 1 |
| $k_{hot}$ | `post.k_hot` | initial extra evaporation at $D_e=D_{ref}$ | 1.3e-7 | 1.2e-7 | m²/s |
| $h$ | `post.hot_exp` | exponent of $r$ in $k_{hot}$ | 0.45 | 0.3 | 1 |
| $\tau_0$ | `post.tau0_s` | decay time at $D_e=D_{ref}$ | 2.8 | 3.7 | s |
| $n$ | `post.tau_exp` | exponent of $r$ in $\tau_e$ | 0.2 | 0.2 | 1 |

**Prefix encoder and state filter**

| Symbol | Name in edits.wsp | Meaning | Methanol D3 | Heptane D2 | Unit |
|---|---|---|---|---|---|
| — | `enc.gain_w`, `enc.gain_b` | rate-offset gain logit on $g$ | (1.4, 0, 0), −1.6 | (0.8, 0, 0), −1.8 | 1 |
| $m_0,m_1$ | `enc.mix` | weights of $e_{late}$, $e_{early}$ | 0.85, 0.15 | 0.85, 0.15 | 1 |
| $e_{lo},e_{hi}$ | `enc.e_clip` | clamp of the log innovations | 0.6, 0.8 | 0.3, 0.6 | 1 |
| — | `enc.cool_gain` | share of the cool end-rate innovation kept | 0 | 0.5 | 1 |
| — | `filt.sd_hot` | sd of $r$ inside a hot flame | 0.18 | 0.2 | 1 |
| — | `filt.sd_evap` | sd of $r$ without a flame | 0.2 | 0.15 | 1 |
| $l_{floor}$ | `filt.l_floor` | density added to each likelihood | 0.03 | 0.05 | 1 |
| — | `filt.gain_w`, `filt.gain_b` | evidence exponent logit $g_s$ | (2.0, 0.5, 2.0), −1.5 | (2.0, 0.5, 2.0), −1.5 | 1 |

**Spread and hazards**

| Symbol | Name in edits.wsp | Meaning | Methanol D3 | Heptane D2 | Unit |
|---|---|---|---|---|---|
| $s_{pop}$ | `unc.s_pop` | sd of a run's $\ln K$ before any prefix | 0.10 | 0.10 | 1 |
| $s_{floor}$ | `unc.s_floor` | sd of $\ln K$ no prefix removes | 0.06 | 0.06 | 1 |
| $\sigma_d$ | `unc.sigma_d` | sd of one diameter measurement | 2.0e-5 | 2.5e-5 | m |
| $\sigma_{frame}$ | `unc.sigma_frame` | frame-to-frame sd of $D$ | 6.0e-6 | 8.0e-6 | m |
| $\ln\lambda_{dis}$ | `haz.log_disruption` | constant disruption hazard | −4.83 | −3.9 | ln(1/s) |

**Spec constants**, fixed and the same for both fuels (`spec.json`; droplet3.rs:391-410): $D_{ref}$ 2 mm, $p_{ref}$ 1e5 Pa, $K_{ref}$ 1e-6 m²/s, $n_{ref}$ 32, window 0.25, end window 0.15, $k_{floor}$ 2e-8 m²/s, $w_{ref}$ 0.08, min points 2, gate tolerance 0.05, gate count 3, $k_{max}$ 2e-6 m²/s, dt 0.05 s, bin 0.5 s, $d_{min}$ 0.25 mm, $T_{valid}$ 45 s, z: 15 nodes × 0.5, ce: 5 nodes × 1.0.

---

### 2. Zero-shot references

#### 2.1 `flex.wsp` W0, methanol (`flex.rs`, `sha256:fa8eb18c…e28f`)

This forward has one hot state and a single extinction intensity per unit of burn progress, with no classes and no cool flame (flex.rs:4-13).

**Burning and extinction.** Classical RK4 on $(q,\Lambda)$ (flex.rs:647-711):

$$\frac{dq}{dt}=-K,\qquad \ln\frac{K}{K_{ref}}=N_{rate}(c,w,s)+b+s_zz_k$$

$$\frac{d\Lambda}{dt}=\eta\,\frac{K}{q_0},\qquad \ln\eta=\min\big(N_{haz}(c,w,s)+\ell_b\,b,\ 12\big)$$

- $\Lambda=0$ at the cutoff (flex.rs:666), so survival is conditional on burning at $\tau$.
- Since $dw/dt=K/q_0$, $\Lambda=\int\eta\,dw$: extinction is decided by progress.
- The cap 12 is `haz.log_eta_cap` (flex.rs:208).

**Steps.** Sub-steps end exactly on each forecast time: $n_{sub}=\max(1,\lceil(t_j-t)/0.05-10^{-9}\rceil)$ (flex.rs:672-673). At $q\le q_{min}$ the path is frozen and its exit time recorded (flex.rs:687-708).

**Extinguished droplet.** It evaporates at a constant rate (flex.rs:749):

$$q(t)=\max\big(q_{mid}-k_{evap}(t-t_{mid}),\,q_{min}\big)$$

There is one atom per RK4 step that carries extinction mass $e^{-\Lambda_i}-e^{-\Lambda_{i+1}}$, located at the step midpoint (flex.rs:692-700). There are no bins.

**Typed laws**

$$\ln\frac{K}{K_{ref}}=-0.58+0.16c_{O_2}-0.075c_{CO_2}+0.11c_{He}+0.08c_{\log p}-0.08c_{\log D_0}+0.052w$$
$$-0.035\,\mathrm{relu}(c_{O_2}-0.8)-0.03\,\mathrm{relu}(c_{O_2}-2)-0.05\,\mathrm{relu}(-c_{O_2}-0.6)-0.06\,\mathrm{relu}(-c_{O_2}-1.2)$$
$$-0.14\,\mathrm{relu}(v_K-0.2)-0.2\,\mathrm{relu}(v_K-0.6)-0.2\,\mathrm{relu}(w-0.25)-0.25\,\mathrm{relu}(w-0.6)-1.0\,\mathrm{relu}(1-6.6667s)$$

$$v_K=-0.05c_{O_2}+0.06c_{CO_2}+0.15c_{\log p}+c_{\log D_0}-0.65w$$

$$\ln\eta=-9.306-1.56c_{O_2}+0.96c_{CO_2}+0.84c_{He}-0.6c_{\log p}+1.2c_{\log D_0}+12w+1.08\,\mathrm{relu}(c_{O_2})+0.3\,\mathrm{relu}(c_{O_2}-0.8)$$
$$+2.4\,\mathrm{relu}(-c_{O_2}-0.6)+4.2\,\mathrm{relu}(-c_{O_2}-1)+3.6\,\mathrm{relu}(-c_{O_2}-1.4)+0.96\,\mathrm{relu}(c_{CO_2}-2)+3.0\,\mathrm{relu}(v_\eta-0.45)+10.8\,\mathrm{relu}(v_\eta-0.85)$$

$$v_\eta=-0.25c_{O_2}+0.08c_{CO_2}+0.15c_{\log p}+c_{\log D_0}$$

Computed here, at $c=0$: $K=e^{-0.58}$ = 0.560 mm²/s on the plateau. Integrated from $w=0$, $\eta=e^{-9.306+12w}$ gives $\Lambda(w)=\ln2$ at $w\approx0.95$.

**Encoder** (flex.rs:409-592). The same admission, windows, $q_0$, innovations, $\gamma$, $b$ and $s_z$ as in 1.6 (flex.rs:481-548), with these differences:
- no validity gate;
- a symmetric clip $e\in[-1,1]$, from the constant `enc.e_clip` = 1.0 (flex.rs:204, 521-523);
- no end windows and no state filter.

The level carries the tail window along the **unclamped** late finite difference (flex.rs:552-553):

$$q_{cut}=\bar q_{tail}-K_{late}(\tau-\bar t_{tail})$$

The slope is 0 if $K_{late}$ is not finite. $K_{cut}$ (flex.rs:551) is reported only. $T_{valid}$ is 30 s, and the $c_{\log D_0}$ domain is $[-0.7,1.26]$ (flex.rs:156, 211).

**Quantiles** (flex.rs:720-778). First, interpolated atom quantiles are taken at the cumulative-weight centres (common.rs:166-189); this gives the latent quantiles. Measurement noise is then added around the median in quadrature, with $z_p=\Phi^{-1}(p)$ (quad.rs:111-157):

$$Q_p=\mathrm{med}\pm\sqrt{(Q^{lat}_p-\mathrm{med})^2+(z_p\sigma_d)^2}$$

The sign follows $p\lessgtr0.5$, and the lower side is floored at 0.

**Events** (flex.rs:744-784):

| Row | Value |
|---|---|
| active | $\sum_k\omega_ke^{-\Lambda_k}e^{-\lambda_{dis}(t-\tau)}$, for paths not yet at the floor |
| hot_phase_end | $\sum_k\omega_k\sum_{\text{steps}}m\,e^{-\lambda_{dis}(t_{mid}-\tau)}$ |
| unresolved | burning paths that reached the floor: $\omega_ke^{-\Lambda_k}e^{-\lambda_{dis}(t_{exit}-\tau)}$ |
| disruption | the remainder |

**Values (`configs/models/flex/wsp/w0-fable/edits.wsp`)**

| Name | Value | Unit |
|---|---|---|
| `haz.link_b` | 0.0 | 1 |
| `haz.log_disruption` | −4.83 | ln(1/s) |
| `enc.gain_w`, `enc.gain_b` | (2.0, 0.5, 2.0), −1.5 | 1 |
| `enc.mix` | (0.85, 0.15) | 1 |
| `unc.s_pop`, `unc.s_floor` | 0.22, 0.06 | 1 |
| `unc.sigma_d` | 2.0e-5 | m |
| `post.k_evap` | 2.0e-8 | m²/s |

Scenario nodes are as in 1.7: 15 × 0.5 (flex.rs:627-637).

#### 2.2 `flex_heptane.wsp` W0 (`droplet.rs`, `sha256:6ea680d1…6d8f`)

`droplet.rs` has the same structure as `droplet3.rs`: three states, two classes, Gompertz laws, cool flame, bins, scenario nodes and mixture quantiles. Its laws are at droplet.rs:652-707, the encoder at 838-1099, and the integration at 1214-1523. A diff of the bodies against droplet3.rs shows only these differences:

| | droplet.rs (W0) | droplet3.rs (D3/D2) |
|---|---|---|
| Unlit law (droplet.rs:695-707) | $q_e-k_\infty r^mu-\lambda k_{hot}\,\tau_e(1-e^{-u/\tau_e})$ with $\tau_e=\max(\tau_0r,10^{-3})$: $h=0$, $n=1$ fixed | $h$, $n$ typed |
| State evidence | the mid→tail ratio $r=K_{late}/K_{mod,late}$ with exponent $\gamma$ (the rate gain); hot sd $s_{pop}$ (droplet.rs:1031-1052) | end-window rate, own exponent $g_s$, `filt.sd_hot` |
| Level at the cutoff | $\bar q_{tail}-\mathrm{clamp}(K_{late},0,k_{max})(\tau-\bar t_{tail})$ (droplet.rs:989-994) | 1.6 step 8 |
| Cool offset | $b_{cool}=\gamma\,\mathrm{clamp}(\ln(\max(K_{late},k_{floor})/K_c),\cdot)$ (droplet.rs:1053-1058) | `enc.cool_gain`$\cdot g_s\cdot$… on $K_{end}$ |
| $\lambda_{now}$ | uses $K_{late}$, $\gamma$ and $k_{hot}$ (droplet.rs:1334-1338) | $K_{end}$, $g_s$, $k_{hot}r^h$ |

**Typed laws (`configs/models/flex/wsp/heptane/w0-fable/edits.wsp`)**

$$\ln\frac{K_h}{K_{ref}}=-0.51+0.13c_{O_2}-0.08c_{CO_2}+0.15c_{He}+0.05c_{\log p}-0.22c_{\log D_0}+0.10w-0.06\,\mathrm{relu}(-c_{O_2})-0.05\,\mathrm{relu}(-0.7-c_{O_2})$$
$$-0.05\,\mathrm{relu}(c_{He}-1.5)-0.3\,\mathrm{relu}(-0.4+c_{\log D_0}-0.65w)-0.7\,\mathrm{relu}(1-5s)$$

$$\ln\frac{D_{crit}}{2\,\mathrm{mm}}=0.34+0.55c_{O_2}-0.15c_{CO_2}-0.05c_{He}-0.2c_{\log p},\qquad \kappa=6,\ \beta=0$$

$$\mathrm{logit}\,w_D=3.2+1.5c_{O_2}-0.3c_{CO_2}+2.0c_{\log D_0}-1.2\,\mathrm{relu}(c_{O_2})-1.0\,\mathrm{relu}(x_{exc}+0.3),\qquad a_D=10$$

$$\ln s_R=-0.6-0.8\,\mathrm{relu}(x_{exc}-0.1),\qquad a_R=4$$

$$\mathrm{logit}\,p_c=1.7-0.2c_{CO_2}-0.8c_{He}+0.5c_{\log p}+1.0x_{exc}-1.0\,\mathrm{relu}(-c_{O_2})$$

$$\ln\frac{K_c}{K_{ref}}=-0.92+0.2c_{O_2}-0.05c_{CO_2}+0.14c_{He}+0.1c_{\log p}+0.2c_{\log D_0}$$

$$\ln\frac{D_{ce}}{2\,\mathrm{mm}}=-0.57-0.18c_{O_2}+0.05c_{CO_2}+0.2c_{He}-1.0c_{\log p}+0.5c_{\log D_0},\qquad sd_{ce}=0.25$$

**Other values**

| Group | Values |
|---|---|
| Unlit droplet | `post.cool_scale` 0.5, `post.k_inf` 8.0e-9 m²/s, `post.k_hot` 4.0e-8 m²/s, `post.tau0_s` 4.0 s, `post.size_exp` 0.7 |
| Encoder | `enc.gain_w` (2.0, 0.5, 2.0), `enc.gain_b` −1.5, `enc.mix` (0.85, 0.15), `enc.e_clip` (0.7, 1.0) |
| Filter | `filt.sd_evap` 0.15, `filt.l_floor` 0.05 |
| Spread | `unc.s_pop` 0.25, `unc.s_floor` 0.08, `unc.sigma_d` 3.0e-5 m, `unc.sigma_frame` 8.0e-6 m |
| Hazard | `haz.log_disruption` −3.9 |

Constants are as in droplet3.rs, without the end window or `filt.gain_*` (droplet.rs:359-377).

---

### 3. The reference baseline: "the D-squared law of the spec" (`baseline-d2`)

Each droplet spec has a root release `baseline-d2`. It runs on the **same forward** as the candidates and uses neutral numbers:

- `configs/models/flex/wsp2/methanol/baseline-d2`: `sha256:abf919b6…247b`, on `droplet3.rs`.
- `configs/models/flex/wsp2/heptane/baseline-d2`: `sha256:8e40db9d…ebb1`, on `droplet3.rs`.
- The gen-1 `configs/models/flex/wsp/{methanol,heptane}/baseline-d2` (`sha256:6c6fd99d…6962`, `sha256:7b4ce460…cd6e`, on `droplet.rs`) has the same numbers, minus the tensors the later spec added (checked by diff).

PROTOCOL names `configs/models/flex/wsp2/heptane/baseline-d2` as "the D-squared law of the same spec" (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md:984-985).

**What it computes**, with the forward of section 1 and these values:

| Element | Values | Effect |
|---|---|---|
| Rate | `rate.b0` −0.777 (methanol) or −0.6 (heptane); all skips and hinges 0 | constant $K_{pop}=e^{-0.777}$ = 0.460 mm²/s or $e^{-0.6}$ = 0.549 mm²/s, so $dq/dt=-K_{pop}e^{b+s_zz_k}$: the $d^2$-law |
| Rate from the prefix | `enc.mix` (1, 0), `enc.e_clip` (3, 3), gain $\gamma=\sigma(-1.5+2\ln\Delta\bar t+0.5\ln(n/32)+2g_{prog})$ | the run's rate becomes $K_{pop}e^{\gamma e_{late}}$: as $\gamma\to1$, the prefix's own mid→tail finite-difference rate, clipped to $[e^{-3},e^{3}]K_{pop}$ |
| Spread | $s_z=\sqrt{0.08^2+(1-\gamma)0.25^2}$; $\sigma_d$ 2e-5 m (methanol) or 3e-5 m (heptane) | |
| Radiative class | `crit.b0` 5, $\kappa$ 20, $\beta$ 0 | $D_{crit}=2e^5$ mm, so $\pi_R=\sigma(20(c_{\log D_0}-5))\approx0$ |
| Cool flame | `cool.p.b0` −30 (heptane) | $p_c\approx10^{-13}$ |
| Diffusive extinction | `diff.b0` 12, $a_D$ 40 | not exactly zero, see below |
| Evaporation | `post.k_inf` = `post.k_hot` = 0 | an extinguished droplet keeps its diameter |
| Disruption | `haz.log_disruption` −20 | none |
| Filter | `filt.l_floor` 1.0, so the state evidence is nearly flat | |

**Diffusive extinction is near zero, not zero.** With $w_D=\sigma(12)$ and $a_D=40$, computed here, $S_D$ is 0.9998 at $w=0.8$, 0.987 at $w=0.9$, 0.910 at $w=0.95$, and 0.694 at the floor of a 2 mm droplet ($w=0.984$). So near burnout part of the mass is "extinguished" at a frozen diameter close to the floor.

The meta.json describes the root as "the classical D^2-law … no extinction and no evaporation" (`configs/models/flex/wsp2/methanol/baseline-d2/meta.json`, rationale). The residual diffusive term above is what the code does with those numbers.

**Other D² baselines.** `configs/models/flex/d2_population.toml` and `d2_prefix.toml` are a different thing: legacy *fitted* D² baselines (MAP, Adam + L-BFGS, `[fit]` section) that are not WSP releases.

---

### 4. Targets, cases, and what is zero-shot

**Target.** The measured liquid diameter (m) of a free-floating droplet at times after the cutoff. It is an unconditional predictive distribution over the droplet's state, plus the cumulative incidence of the end of the hot phase (droplet3.rs:426-437). Flame diameter, soot and temperature are excluded (droplet3.rs:412-420). Release applicability states the time origin "NASA DropletData processed time base (phase entry not adjudicated)" (d3-fable/release.json).

**Development cases** (SpaceApps-Ardealu:crates/ignis-wsp-dev/src/main.rs:53-63, 334-356):
- Cutoffs are 1, 3 and 5 s on the run's time axis.
- A (run, cutoff) case is admitted only if the run record establishes the phase at the cutoff (`at_risk`, SpaceApps-Ardealu:crates/ignis-core/src/analysis.rs:121-123).
- The horizon is 10 s, with 41 forecast times from $\tau$ to $\tau+10$ s.
- Default levels are 0.05, 0.25, 0.5, 0.75 and 0.95.
- Scores are quantile CRPS, MAE of the median, and coverage of the 50 % and 90 % bands (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md:1001, "As on FLEX").

**Reported scores** (SpaceApps-Ardealu:docs/wsp/README.md:61-62; quoted, not recomputed here):

| Model | Development CRPS | Comparison |
|---|---|---|
| methanol D3 | 40.0 µm | zero-shot `flex.wsp` W0: 50.4 |
| heptane D2 | 46.1 µm | zero-shot W0: 60.3; D²-law: 105 |

**Zero-shot versus data-informed.** README:88 says that only W0 candidates and the reference baselines are zero-shot. Every D-named candidate was typed after its author read development (train) data.

- `flex_methanol.wsp2` D3 and `flex_heptane.wsp2` D2 are data-informed: their edits headers say "DATA-INFORMED … typed after the author read the … TRAIN runs". Each was committed before its validation score existed, and validation acted only as a veto (PROTOCOL.md §8).
- `flex.wsp` W0 and `flex_heptane.wsp` W0 are zero-shot. The heptane W0 was written before any n-heptane outcome was opened, but it used "the structure (not the numbers) learned from the methanol train runs" (its edits header).
- The baseline-d2 roots are reference baselines with no data exposure (meta.json `exposure.datasets` = []).

---

### 5. Generation 6/7 multi-fuel forward `droplet8.rs` (`flex_<fuel>.wsp7` W0), in brief

#### 5.1 Relation to the forwards above

`droplet8.rs` adds no new law. It is `droplet6.rs` with:
- eight fuels;
- `has_cool` true for every fuel except methanol and ethanol (droplet8.rs:332-334);
- $c_{He}\le4.25$ and $c_{\log p}\le1.7$ (droplet8.rs:245-247, 738-739);
- $T_{valid}$ = 200 s (droplet8.rs:249, 633);
- fuel-name normalisation (droplet8.rs:342-352).

These differences come from a diff of droplet6.rs against droplet8.rs. The other mechanisms are inherited:
- from `droplet5.rs`: the heavy-tailed radiative clock (`rob.eps_r`) and the flame cameras (`fl.*`);
- from `droplet6.rs`: self-cooling (`sc.*`), the dying flame (`dy.*`) and the closed-form cool-flame end (`ce.*`).

With `sc.on` = `ce.on` = 0 and `dy.frac` = 0 the forward equals droplet5 (droplet8.rs:795). Everything else is as in `droplet3.rs`: $c$, $q_0$, hot rate, $D_{crit}$, class, $\Lambda_D$, $\Lambda_R$, gate, windows, end rate, gains, $b$, $s_z$, level, filter, RK4, bins, nodes and mixture quantiles.

#### 5.2 Additions to the laws

**Radiative survival with a heavy tail.** A log-logistic clock of shape 1 and the same median is mixed in (droplet8.rs:1251-1258):

$$S_R(s)=(1-\varepsilon_r)e^{-\Lambda_R(s)}+\frac{\varepsilon_r}{1+\max(s,0)/s_R}$$

**Cool-flame end in O₂ partial pressure** (`ce.on` = 1; droplet8.rs:1153-1169):

$$D_{ce}=D_0\Big(\frac{D_0}{D_{ref}}\Big)^{n_{ce}-1}g_0\sqrt{\frac{p_{ref}-\beta}{\max(p_{O_2}-\beta,\epsilon)}}\,(1+c_{He}x_{He}),\qquad p_{O_2}=\frac{x_{O_2}\,p}{101325\ \mathrm{Pa}}\ (\mathrm{atm})$$

$D_0=\sqrt{q_0}$, and the spread `cool.d.sd` is kept.

**Self-cooling unlit droplet** (`sc.on` = 1; droplet8.rs:1171-1228). The rate is $K_{floor}(u)+K_0/(1+u/t_0)$, integrated in closed form:

$$J_{sc}(u)=K_0t_0\ln\Big(1+\frac{u}{t_0}\Big)+k_{cold}'u+(k_{warm}'-k_{cold}')\tau_w\big(1-e^{-u/\tau_w}\big),\qquad t_0=\frac{T^*c_l\,q_e}{1.5K_0L}$$

with $g=\exp(\gamma_0x_{He}+\gamma_1x_{CO_2}+\gamma_2\ln(p/p_{ref}))$ and:

| Last flame | $K_0$ | $k'_{cold}$ | $k'_{warm}$ |
|---|---|---|---|
| lost to radiation | $g\,k_{0,hot}$ | $g\,k_{cold}$ | $k_{warm}$ |
| ended diffusively | $g\,f\,k_{0,hot}$ | $g\,f\,k_{cold}$ | $f\,k_{warm}$, with $f$ = `sc.diff_scale` |
| cool flame | $g\,k_{0,cool}$ | $g\,k_{cold}$ | $k_{warm}$ |

Branches: if $1.5K_0L\le0$ then $t_0=\infty$ and the heat term is $K_0u$; if $K_0\le0$ the heat term is 0.

**Dying flame after a radiative loss without a cool flame** (droplet8.rs:1230-1234, 2290-2317):

$$J_{die}(u)=k_{die}\tau_{dy}(1-e^{-u/\tau_{dy}}),\qquad k_{die}=\texttt{dy.frac}\cdot K_ce^{b+s_zz_k}$$

**Flame cameras** (droplet8.rs:1458-1489, 1833-1851). The last time with continuous UV or visible flame frames, $t_\ell$, shifts the state probabilities:

$$P\leftarrow\texttt{fl.trust}\cdot v+(1-\texttt{fl.trust})\,P$$

$v=(1,0,0)$ if the flame is seen within `fl.gap_s` of the cutoff; otherwise $v=(0,P_c,P_n)/(P_c+P_n)$. The flame diameter itself is not read.

#### 5.3 Where the fuel enters

There is no fuel-property table in the code. A fuel enters only:
- through its spec id, name and `has_cool`;
- through typed release numbers, including the property constants $c_l$, $L$ and $T^*$ inside `sc.*`.

The only physical constants in the code are 101325 Pa and 1e-6 (mm²/s → m²/s) (droplet8.rs:222-224).

#### 5.4 Typed values per fuel (`configs/models/flex/wsp7/<fuel>/w0-fable/edits.wsp`)

There is no heptane or methanol W0 in wsp7; their folders hold `carry-*` releases. Units are as in spec.json.

**Release ids and use**

| | decane | dodecane | ethanol | farnesane | iso-octane | octane |
|---|---|---|---|---|---|---|
| release id | `62e40661…e5bf` | `70042961…3e74` | `ae03363f…b139` | `66b398e4…a0d3` | `b2e57d80…e7f2` | `2c7a7716…8297` |
| used on | FLEX-2 | CFI | — | — | FLEX-2 | FLEX-2 |

**Hot flame and extinction**

| Parameter | decane | dodecane | ethanol | farnesane | iso-octane | octane |
|---|---|---|---|---|---|---|
| `rate.b0` [ln(K/1e-6 m²/s)] | −0.66 | −0.76 | −0.62 | −0.82 | −0.44 | −0.55 |
| `crit.b0` [ln(D/2 mm)] | 0.77 | 0.86 | 0.69 | 0.91 | 0.56 | 0.67 |
| `class.kappa` / `bias` | 6 / 0 | 6 / 0 | 6 / 0 | 6 / 0 | 6 / 0 | 6 / 0 |
| `diff.b0` / `sharp` | 2.44 / 10 | 2.44 / 10 | 2.6 / 8 | 2.44 / 10 | 2.44 / 10 | 2.44 / 10 |
| `rad.b0` / `sharp` | 0.227 / 4 | 0.227 / 4 | 0.0 / 4 | 0.227 / 4 | 0.227 / 4 | 0.227 / 4 |
| `rob.eps_r` | 0.2 | 0.2 | 0.2 | 0.2 | 0.2 | 0.2 |

**Cool flame and dying flame**

| Parameter | decane | dodecane | ethanol | farnesane | iso-octane | octane |
|---|---|---|---|---|---|---|
| `cool.p.b0` [logit] | 3.0 | 3.0 | — | 3.0 | −4.0 (+2.0 on $c_{\log p}$) | 3.0 |
| `cool.k.b0` [ln K] | −1.34 | −1.47 | — | −1.53 | −1.07 | −1.21 |
| `ce.on`; `ce.g0` | 1; 0.42 | 1; 0.40 | 0 | 1; 0.38 | 1; 0.48 | 1; 0.45 |
| `ce.beta_atm` [atm] | 0.03 | 0.03 | — | 0.08 | 0.15 | 0.09 |
| `dy.frac` / `dy.tau_s` [s] | 1 / 3.5 | 1 / 3.5 | 0 / 1.0 | 1 / 3.5 | 0 / 3.5 | 1 / 3.5 |

**Self-cooling unlit droplet**

| Parameter | decane | dodecane | ethanol | farnesane | iso-octane | octane |
|---|---|---|---|---|---|---|
| `sc.on` | 1 | 1 | 1 | 1 | 1 | 1 |
| `sc` k0_hot / k0_cool [mm²/s] | 0.25 / 0.18 | 0.15 / 0.17 | 0.29 / — | 0.15 / 0.16 | 0.44 / 0.17 | 0.47 / 0.19 |
| `sc.tstar_k` [K] | 33 | 36 | 16 | 38 | 28 | 29 |
| `sc.cl_j_kg_k` [J/(kg K)] | 2650 | 2800 | 2720 | 2800 | 2300 | 2500 |
| `sc.lat_j_kg` [J/kg] | 289000 | 272000 | 875000 | 234000 | 283000 | 316000 |

**Spread and hazard**

| Parameter | decane | dodecane | ethanol | farnesane | iso-octane | octane |
|---|---|---|---|---|---|---|
| `unc.s_pop` / `s_floor` | 0.20 / 0.10 | 0.20 / 0.10 | 0.20 / 0.10 | 0.25 / 0.12 | 0.20 / 0.10 | 0.20 / 0.10 |
| `unc.sigma_d` [m] | 4.2e-5 | 4.2e-5 | 3.3e-5 | 4.2e-5 | 4.2e-5 | 4.2e-5 |
| `haz.log_disruption` | −3.9 | −3.9 | −4.83 | −3.9 | −3.9 | −3.9 |

Shared by all six: `fl.trust` 0.95. `enc.cool_gain` is 0.7 for the alkanes and 0 for ethanol.

The values `rate.b0`, `crit.b0`, `cool.k.b0`, `ce.g0`, `ce.beta_atm`, `sc.tstar_k`, `sc.lat_j_kg`, `unc.s_pop`, `rob.eps_r`, `fl.trust` and the release ids were checked directly against the edits and release files.

#### 5.5 Zero-shot status

PROTOCOL calls these "the zero-shot candidates of arm F" (SpaceApps-Ardealu:docs/wsp/PROTOCOL.md:989, 1136). Their own `meta.json` rationale says "one shot for this fuel and not zero-shot". No outcome of the new fuel was opened, but the law is the n-heptane law (the methanol law for ethanol), whose numbers were typed after those fuels' train partitions. The fuel-specific numbers are those numbers scaled by hand-worked property ratios. PROTOCOL.md:1318-1320 plans dodecane W0 on the CFI test-day droplet and octane/decane on FLEX-2.

---

### 6. `flex_heptane.wsp10` D4 (SpaceApps-Ardealu:`droplet10.rs`), brief

`droplet10.rs` is a copy of `droplet3.rs`, not of droplet8. It adds three tensors, `trans.depth`, `trans.tau_s` and `trans.tau_exp` (SpaceApps-Ardealu:crates/ignis-wsp/src/droplet10.rs:357-368). They apply only to a cool flame that follows a hot flame lost *inside* the forecast; a cool flame burning at the cutoff is unchanged (droplet10.rs:1567, 1621). The cool rate rises from a sag after the transition (droplet10.rs:1489-1490):

$$K(t)=K_c\big(1-a\,e^{-(t-t_e)/\tau_t}\big),\qquad \tau_t=\max\big(\tau_s(q_e/D_{ref}^2)^{n_t},10^{-3}\big)$$

$$q(t)=q_e-K_c\Big[(t-t_e)-a\tau_t\big(1-e^{-(t-t_e)/\tau_t}\big)\Big]$$

The end time where $q=q_x$ is found by 40 bisections on $[(q_e-q_x)/K_c,\ (q_e-q_x)/K_c+a\tau_t]$ (droplet10.rs:1492-1502). With $a=0$ the forward is droplet3.

**The release.** `configs/models/flex/wsp10/heptane/d4-fable`, `sha256:f3ccc122…b164`. It uses D2's laws plus D3's cool-rate block (`cool.k.b0` −1.125, skip (0, 0, 0.2, 0, 0.15), a pressure hinge with $w_2$ = 0.5), with `trans.depth` 0.25, `trans.tau_s` 3.7 s and `trans.tau_exp` 0.2. It is data-informed: its header says "DATA-INFORMED, not zero-shot", and it was typed from n-heptane train runs.

---

### 7. Not determined or not checked

- **Gain values at the cutoffs.** The value of $\bar t_{tail}-\bar t_{head}$ at the 1/3/5 s cutoffs depends on each run's sampling and first frame. The gain values per cutoff quoted from the edits comments were not recomputed.
- **Release tensors.** `release.json` tensor arrays were not compared element by element with `edits.wsp`. Assembly semantics were read from assemble.rs, and the parameter counts in release.json (`authored_nonzero` 68 for methanol, 93 for heptane) were not reconciled.
- **Sections 5 and 6** come from a delegated read of droplet8.rs, droplet10.rs and their release folders. Of those, I checked the droplet8 lines for $S_R$, $D_{ce}$, self-cooling and the dying flame, plus the ten values and the release ids listed in section 5.4. The remaining wsp7 table entries, the camera rule and every droplet10 line reference were not rechecked.
- **Scores** in section 4 are quoted from the README, not recomputed.
- **Not run.** No forward was run for this note.


## Part 2. ACME spherical burner flames

The code paths are relative to the root of `ignis-web`. The forward files `crates/ignis-wsp/src/*.rs` are byte-identical in `SpaceApps-Ardealu` (branch `feat/handwritten-science`, `542dad8e`).

Files that exist only in the research repo are marked *(research repo)*.

Numeric values come from three places:

- the typed `edits.wsp` lines;
- the stored tensors of each release (`artifacts/models/<id>/params.npz`, research repo), read with numpy;
- the fuel tables in `configs/models/acme/fuel_tables/`.

An `edits.wsp` lists only the statements that differ from its parent. Its slot values are copies made by `tools/wsp/committee3_edits.py`. The full value set is therefore the stored one.

In this document, "hinge law" means a law written with `nn::hinge_net`, and "growth law" means the erfc growth clock of `flame3.rs` and `flame6.rs`.

Nothing in any of these forwards is fitted. Each spec declares `ForwardProfile::StrictNoFitting` (e.g. `crates/ignis-wsp/src/committee3.rs:301-305`), and `fit` bails (`committee3.rs:979-981`).

No Weibull law appears anywhere in `crates/ignis-wsp/src` (grep). Every event clock below is log-logistic, exponential, or a point mass spread over a 1 s window. The 7-point Gauss–Hermite rule `quad::GH7_*` (`crates/ignis-wsp/src/quad.rs:90-107`) is defined but not used by these forwards. Their quadratures use equally likely normal-quantile nodes.

---

### 0. Shared kernels

**Hinge network** (`crates/ignis-wsp/src/nn.rs:69-84`). For an input $x\in\mathbb R^n$ and hidden width $h$:

$$N(x)=b_0+\sum_{k}s_k x_k+\sum_{j=1}^{h} w_{2,j}\,\max\!\Big(0,\;b_{1,j}+\sum_k W_{1,jk}x_k\Big).$$

The tensors are `<law>.b0`, `<law>.skip`, `<law>.w2`, `<law>.b1` and `<law>.W1`, with $W_1$ row-major $[h,n]$. With every hidden weight zero, $N$ is the linear law $b_0+s\cdot x$.

**Logistic** (`nn.rs:44-51`):

$$\sigma(x)=\frac{1}{1+e^{-x}}$$

It is computed as $e^{x}/(1+e^{x})$ for $x<0$.

**Normal quantile.** $\Phi_N^{-1}(p)$ is Acklam's rational approximation, with $|\text{rel. err}|<1.2\cdot10^{-9}$ (`quad.rs:111-157`).

**Equal-probability nodes** (`crates/ignis-wsp/src/flame6.rs:405-411`, the same in `flame7.rs:379-385`):

$$z_i=\begin{cases}0,& n=1\\[2pt] \Phi_N^{-1}\!\big(\tfrac{i+1/2}{n}\big),& i=0,\dots,n-1\end{cases}$$

**Log-logistic survival and hazard** (`flame6.rs:390-395`, `flame7.rs:360-375`):

$$S_{LL}(t;\tau,k)=\begin{cases}1,&t\le 0\\[2pt] \dfrac{1}{1+(t/\tau)^k},&t>0\end{cases}\qquad
\lambda_{LL}(t;\tau,k)=\begin{cases}0,&t\le0\ \text{or}\ \tau\le0\\[2pt] \dfrac{k}{t}\dfrac{(t/\tau)^k}{1+(t/\tau)^k},&\text{otherwise}\end{cases}$$

**Inverse complementary error function** (`flame6.rs:399-401`):

$$\operatorname{erfc}^{-1}(y)=-\Phi_N^{-1}(y/2)/\sqrt2$$

**Forecast times** (`crates/ignis-wsp/src/common.rs:75-96`; members `flame6.rs:977-985`):

- The times are the request's `times_s`, sorted.
- Otherwise there are `n_points` (default 41, at least 2) equally spaced times from the cutoff to $\min(\tau+h,\,t_{\max})$, with default $h=60$ s and $t_{\max}=$ `num.horizon_s`.
- Times $<\tau$ and non-finite times are dropped, and exact duplicates are removed.
- An empty list is a horizon refusal.

---

### 1. `acme_normal.wsp12` F2 and `acme_inverse.wsp12` F2 (forward `committee3.rs`)

| Release | Folder | Id |
|---|---|---|
| normal F2 | `configs/models/acme/wsp12/normal/f2-fable` | `sha256:55fcf4cbbf500ca57a4e4207426826d5c3d3b21d84fb279479e29b7e5a65971d` |
| inverse F2 | `configs/models/acme/wsp12/inverse/f2-fable` | `sha256:dff7da0da8e77785acecc0ffb5acaf222358d4274b36e2c52f39426aaebd594e` |

The model keys `acme_normal.wsp12` and `acme_inverse.wsp12` run `Forward::Committee3` (`crates/ignis-wsp/src/models.rs:362-371`). The committee has ten slots (`committee3.rs:68-78`):

| Slots | Member forward | Tensors |
|---|---|---|
| a0, a1 | `flame5.rs` | `a0.*`, `a1.*` |
| b0–b3 | `flame6.rs` | `b0.*` … `b3.*` |
| c0–c3 | `flame7.rs` | `c0.*` … `c3.*` |

Each slot holds a full member release, including its own fuel table (`committee3.rs:279-286`, `357-394`).

**Which slots are run in F2.** A slot with weight zero is not run (`committee3.rs:758-764`).

- **Normal F2:**
  - b0 and c0, weight ½ each, early era, no planned stop;
  - b1 and c1, late era, no planned stop;
  - b2 and c2, early era, with the planned stop;
  - b3 and c3, late era, with the planned stop.
  - Slots a0 and a1 have weight 0 in every vector, so `flame5.rs` is not run.
- **Inverse F2:**
  - b0 (weight 2) and b1 (weight 1) without the planned stop;
  - b2 (2) and b3 (1) with it.
  - The a and c slots are never run.

#### 1.1 Inputs and condition features

The prefix conditions used are `pressure` $p$, `injected_mass_flow` $\dot m$ (the burner-side reactant alone), `x_o2_role`, `x_fuel_role`, `tad` $T_{ad}$ and `zst` $Z_{st}$. Each must be resolved and finite (`common.rs:50-62`).

The feature vector (`flame6.rs:734-744`, identical in `flame7.rs`) is

$$c=\Big(\ln\tfrac{p}{p_{ref}},\ \ln\tfrac{\dot m}{\dot m_{ref}},\ \tfrac{x_{O_2}-x_{O_2,ref}}{s_{O_2}},\ \ln\tfrac{x_f}{x_{f,ref}},\ \tfrac{T_{ad}-T_{ref}}{T_{sc}},\ \tfrac{Z_{st}-Z_{ref}}{Z_{sc}},\ e_k^{ft},\ b_k^{ft}\Big)$$

- The last two entries are the fuel's `fuel.ft_coord` (§1.2).
- Normal flames: $x_{O_2}$ is the ambient O₂ fraction and $x_f$ is the injected fuel fraction.
- Inverse flames: $x_{O_2}$ is the injected O₂ fraction and $x_f$ is the ambient fuel fraction (`crates/ignis-wsp/src/flame3.rs:184-224`).

**Burner-side reactant** (`flame6.rs:705-733`; `flame7.rs:656-665`). The reactant fraction and molar mass are

- normal flames: $x_R=x_f$ and $W_R=$ `fuel.mw_g_mol[k]`;
- inverse flames: $x_R=x_{O_2}$ and $W_R=$ `phys.mw_o2`.

The derived burner quantities are

$$Y_R=\frac{x_RW_R}{x_RW_R+(1-x_R)W_{N_2}},\qquad \dot m_{tot}=\dot m/Y_R,\qquad
L=\begin{cases}\ln\frac{1}{1-Z_{st}}&\text{normal}\\[2pt]\ln\frac1{Z_{st}}&\text{inverse}\end{cases}$$

**Condition-gate refusals** (`flame6.rs:683-717`):

- $p\le0$, $\dot m\le0$ or $x_f\le0$;
- $Z_{st}\notin(0,1)$;
- $x_R\notin(0,1]$;
- a fuel-table refusal (§1.2).

**Domain refusal.** The six continuous features are checked against the declared domain, on $c$ before any fuel shift (`flame6.rs:954-970`). The domain is defined in `flame3.rs:196-224`:

| feature | normal | inverse |
|---|---|---|
| $c_{logp}$ | $[-0.92,1.25]$ | $[-0.92,1.25]$ |
| $c_{logm}$ | $[-2.31,1.39]$ | $[-1.61,0.88]$ |
| $c_{o2}$ | $[-1.5,1.5]$ | $[-1.6,1.6]$ |
| $c_{xf}$ | $[-2.71,1.21]$ | $[-0.70,1.61]$ |
| $c_{tad}$ | $[-2.67,2.33]$ | $[-2.0,3.0]$ |
| $c_{zst}$ | $[-1.1,2.5]$ | $[-1.85,1.5]$ |

#### 1.2 The fuel table and how the fuel is read (`crates/ignis-wsp/src/fuel.rs`)

The rows are 0 ethane, 1 propane, 2 butane, 3 ethylene, 4 methane, 5 hydrogen (`fuel.rs:53-55`).

**The row $k$** (`fuel.rs:244-305`):

- It is the text condition `fuel_species`, trimmed and lower-cased; `n-butane` maps to row 2 (`fuel.rs:234-240`).
- Without it, it comes from the stored indicators `fuel_is_ethane` and `fuel_is_butane`:
  - (1,0) gives row 0;
  - (0,0) gives row 1;
  - (0,1) gives row 2;
  - anything else is refused.
- A present indicator that is not the row's own is refused. `fuel_is_ethane` must be 1 exactly for row 0, and `fuel_is_butane` exactly for row 2.

**Release gates** (`fuel.rs:308-325`):

- `fuel.admit[k]`$=0$ is refused, and so is any value other than 0 or 1;
- $T_{ad}<$ `fuel.tad_min_k[k]` is refused.

**Vertex weighing** (`fuel.rs:343-393`). A law network $N$ that reads the two indicator entries is evaluated at the alkane vertices $E=(1,0)$, $P=(0,0)$ and $B=(0,1)$:

$$N^*(x)=\sum_{v\in\{E,P,B\},\,w_v\neq0} w_v\,N(x\,|\,(x_{i_e},x_{i_b})\leftarrow v),\qquad (w_E,w_P,w_B)=(e,\,1-e-b,\,b)$$

- The coordinates $(e,b)$ are `fuel.se_coord[k,:]` for the self-extinction laws and `fuel.ft_coord[k,:]` for the burner-rise law.
- A weight that is exactly zero (IEEE) is skipped.
- Methane's `ft_coord` $(1.9,-0.9)$ gives $w_P=1-1.9+0.9=1.1\cdot10^{-16}$ in IEEE arithmetic, so the P vertex is evaluated with that weight.
- The programmed-stop logits are linear and read the coordinates directly: $c$ with entries 6 and 7 set to `ft_coord` (`fuel.rs:398-403`).
- For an alkane on its own vertex with zero shifts, the forecast is that of the indicator forward bit for bit (`fuel.rs:29-31`).

**Fuel tables of the F2 releases** (stored values; the same in every slot of a kind):

| row | admit | $W$ (g/mol) | `fuel.s` growth b | `fuel.s` hinge c | `se_coord` | `ft_coord` | `dosc` | `tad_min_k` (K) |
|---|---|---|---|---|---|---|---|---|
| normal: ethane | 1 | 30.07 | 0 | 0 | (1,0) | (1,0) | 0 | 0 |
| propane | 1 | 44.1 | 0 | 0 | (0,0) | (0,0) | 0 | 0 |
| butane | 1 | 58.12 | 0 | 0 | (0,1) | (0,1) | 0 | 0 |
| ethylene | 1 | 28.054 | −0.02 | −0.31 | (1,0) | (1,0) | 0 | 1900 |
| methane | 1 | 16.043 | 0.068 | 0.068 | (1,0) | (1.9,−0.9) | 0 | 1900 |
| hydrogen | **0** | 2.016 | 0.757 | 0.757 | (0,0) | (0,0) | 0 | 0 |
| inverse: ethane | 1 | 30.07 | 0 | – | (1,0) | (1,0) | 0 | 0 |
| propane | 1 | 44.1 | 0 | – | (0,0) | (0,0) | 0 | 0 |
| butane | 1 | 58.12 | 0 | – | (0,1) | (0,1) | 0 | 0 |
| ethylene | 1 | 28.054 | 0 | – | (1,0) | **(0.5,0)** | **0.7** | 0 |
| methane | **0** | 16.043 | 0 | – | (0,0) | (0,0) | 0 | 0 |
| hydrogen | **0** | 2.016 | 0 | – | (0,0) | (0,0) | 0 | 0 |

Sources: `configs/models/acme/fuel_tables/normal-f2-growth.wsp`, `normal-f2-hinge.wsp` and `inverse-f1.wsp`, confirmed against `params.npz`.

What reads each column:

- `fuel.s` is read only by normal flames (`flame6.rs:521`, `flame7.rs:480`).
- `fuel.dosc` is read only by `flame6.rs` (`flame6.rs:612`).
- An inverse flame's $W_R$ is `phys.mw_o2`, so the table's molar mass is not read (`flame6.rs:718-722`).

#### 1.3 Growth member `flame6.rs` (slots b): radius, extinction, growth clock

**Laws** (`flame6.rs:521-545`). The fuel shift is $s_f=$ `fuel.s[k]` for normal flames and $0$ for inverse flames. Units: $m_{cr}$ in g m⁻² s⁻¹, $D$ in mm² s⁻¹, and $\rho D=$ `se.rho_d_g_m_s` in g m⁻¹ s⁻¹; the code converts all three to SI.

$$\ln m_{cr}=N^*_{se}(\texttt{se.flux};c)-s_f,\qquad \ln D=N^*_{se}(\texttt{se.diff};c)$$

**Quasi-steady flame radius and radiative-extinction radius.** The extinction radius is the radius at which the reactant flux per flame area equals the critical flux:

$$r_{ss}=\frac{\dot m_{tot}}{4\pi\,\rho D\,L},\qquad
r_{ext}=\sqrt{\frac{\dot m}{4\pi\,m_{cr}}},\qquad
\hat\Phi=\Big(\frac{r_{ss}}{r_{ext}}\Big)^2,\qquad t_g=\frac{r_{ext}^2}{4D}$$

(`flame6.rs:539-545`).

**Growth clock.** The flame radius grows as

$$r_f=r_{ss}\,\operatorname{erfc}\!\big(r_f/(2\sqrt{Dt})\big)$$

The time to reach $r_{ext}$ is

$$R(\Phi)=\frac{t_g}{\big[\operatorname{erfc}^{-1}(\Phi^{-1/2})\big]^2}\quad\text{if }\ln\Phi>0,\ t_g>0,\ 0<\Phi^{-1/2}<1,\ \operatorname{erfc}^{-1}>0,\ R\ \text{finite},\ R<\texttt{num.tau\_cap\_s};\ \text{else none ("held")}$$

(`flame6.rs:418-433`). $\Phi^{-1/2}$ is taken as $e^{-\frac12\ln\Phi}$.

**Reduced-flow quadrature** (`flame6.rs:546-551`). With $n=\mathrm{clamp}(\texttt{num.phi\_nodes},1,64)$ (`flame6.rs:465`):

$$\ln\Phi_i=\ln\hat\Phi+\sigma_\phi z_i,\qquad R_i=R(\Phi_i)$$

$$p_{held}=\#\{i:R_i\text{ none}\}/n$$

**Weak class and second clocks** (`flame6.rs:530-537`, `601-613`):

$$p_w=\sigma\big(N^*_{se}(\texttt{se.flash};c)\big),\qquad
\tau_{osc}=\exp\big(N^*_{se}(\texttt{se.osc};c)+\texttt{fuel.dosc}[k]\big),\qquad
h_{hold}=e^{\texttt{se.hold\_log\_rate}}$$

**Self-extinction survival from the time origin.** With $t'=\max(t-t_{fl},0)$ and $t_{fl}=$ `se.floor_s` (`flame6.rs:631-646`):

$$S_w(t)=S_{LL}(t;\tau_w,k_w),\qquad
S_i(t)=\big[S_{LL}(t';R_i,k_{se})\ \text{or }1\text{ if held}\big]\;S_{LL}(t';\tau_{osc},k_{osc})\;e^{-h_{hold}t'}$$

$$S_{se}(t)=p_w S_w(t)+(1-p_w)\frac1n\sum_{i}S_i(t)$$

#### 1.4 Burner-heating law (both member kinds b and c)

**Burner features** (`flame6.rs:553-561`, `flame7.rs:496-504`):

$$b=\Big(\ln\frac{r_{ss}[\mathrm{mm}]}{r_{ref}},\ \ln\frac{\dot m_{tot}}{\dot m_{tot,ref}},\ c_{tad},\ c_{xf},\ c_{logp},\ e,\ b\Big)$$

**Equilibrium rise** (`flame6.rs:562`). With $T_0=$ `bt.t0_c` and $\tau_b=$ `bt.tau_s`, the burner temperature is $T_b(t)=T_0+\Delta T\,(1-e^{-t/\tau_b})$, where

$$\ln\Delta T=N^*_{ft}(\texttt{bt.rise};b)$$

**Time to limit $l$** (`flame6.rs:438-441`):

$$t_l(\Delta T)=-\tau_b\ln\!\Big(1-\frac{T_{lim,l}-T_0}{\Delta T}\Big)\quad\text{if }\frac{T_{lim,l}-T_0}{\Delta T}<1,\ \text{else never}$$

**Heating quadrature and limit mixture** (`flame6.rs:567-588`, `flame7.rs:510-530`). Set $\bar w_l=\texttt{bt.limit\_w}[l]/\sum\texttt{bt.limit\_w}$ and $n_b=\mathrm{clamp}(\texttt{num.bt\_nodes},1,64)$. Then

$$\ln\Delta T_j=\ln\Delta T+\sigma_{bt}z_j$$

$$S_{bt}(t)=\min\Big(1,\ p_{never}+\sum_{l:\bar w_l>0}\sum_{j:t_{jl}\text{ exists}}\frac{\bar w_l}{n_b}S_{LL}(t;t_{jl},k_{bt})\Big)$$

$p_{never}$ collects the weight $\bar w_l/n_b$ of every pair that never reaches its limit. When all limit weights are 0, $p_{never}=1$ and the clock is off.

A weighted limit with $T_{lim,l}\le T_0$ makes the release unrunnable: it is a semantic refusal (`flame6.rs:500-511`, `912-919`).

#### 1.5 Flow-termination clocks other than the burner limit (both kinds)

**Programmed-stop atoms** (`flame6.rs:590-599`). For $a=0..7$:

$$p_a=\sigma\big(\texttt{ft.atom\_logit}[a]+\texttt{ft.atom\_w}[a,:]\cdot c_{ft}\big)$$

- $c_{ft}$ is $c$ with entries 6 and 7 set to `ft_coord`.
- An atom is kept only if $p_a>10^{-9}$ and $T_a=\texttt{ft.atom\_t\_s}[a]>0$.
- Kept values are clipped to $\min(p_a,1-10^{-9})$.

**Survival, growth member** (`flame6.rs:652-656`). With $\beta=e^{\texttt{ft.log\_base}}$ and $h_a=$ `num.atom_half_s`:

$$S_{ps}(t)=e^{-\beta\max(t,0)}\prod_a(1-p_a)^{\mathrm{clamp}\left(\frac{t-(T_a-h_a)}{2h_a},0,1\right)}$$

**Hazard, hinge member** (`flame7.rs:573-578`), the same law written as a hazard:

$$\lambda_{ft}(t)=\beta+\sum_a\mathbf 1\{|t-T_a|\le h_a\}\frac{-\ln(1-p_a)}{2h_a}$$

#### 1.6 Hinge member with the heating law `flame7.rs` (slots c)

**Laws** (`flame7.rs:480-494`, `544-547`):

$$\ln\tau_R=N^*_{se}\big(\texttt{se.tau};\,c\ \text{with}\ c_{logm}\to c_{logm}+s_f\big),\qquad p_w=\sigma\big(N^*_{se}(\texttt{se.flash};c)\big)$$

- $s_f=$ `fuel.s[k]` for normal flames and 0 for inverse flames.
- $r_{ss}$ is computed only for the burner features.
- There is no growth clock, no quench clock, and `fuel.dosc` is not read.

**Hazards** (`flame7.rs:567-581`):

- the weak class has $\lambda_{se}=\lambda_{LL}(t;\tau_w,k_w)$;
- the main class has $\lambda_{se}=\lambda_{LL}(t-t_{fl};\tau_R,k_{se})$;
- $\lambda_{ft}$ is as in §1.5;
- the burner limit enters through $S_{bt}$ (§1.4, `flame7.rs:584-593`).

#### 1.7 Competing-risk integration and the emitted table (members)

The cutoff is $\tau=$ `prefix.cutoff_s`. The stop $t_{stop}$ is the earliest `t_start_s` of a control stage `scheduled_stop` or `flow_stop`, or of an operator action whose control starts with `mfcs off` (`flame6.rs:752-763`). Integration ends at $t_{end}=\min(t_{stop},\texttt{num.horizon\_s})$, or at the horizon when there is no stop (`flame6.rs:808`).

The grid between successive targets $\min(t_j,t_{end})$ has

$$n_{sub}=\max\big(1,\lceil (t_b-t_a)/\texttt{num.dt\_s}-10^{-9}\rceil\big)$$

equal steps (`flame6.rs:824-825`, `flame7.rs:731-732`).

**Growth member** (`flame6.rs:798-862`). The integration starts at $\tau$ with $s=1$. Each step uses survival ratios, so it is conditioned on survival to the cutoff:

$$\Lambda_e=-\ln\min\!\Big(1,\tfrac{S_{se}(t_b)}{S_{se}(t_a)}\Big),\quad
\Lambda_o=-\ln\min\!\Big(1,\tfrac{S_{bt}(t_b)}{S_{bt}(t_a)}\Big)-\ln\min\!\Big(1,\tfrac{S_{ps}(t_b)}{S_{ps}(t_a)}\Big)$$

- Each term is 0 if its survival at $t_a$ is $\le0$, and $+\infty$ if the ratio is 0 (`flame6.rs:786-796`).
- The step loses $\ell=s\,(1-e^{-(\Lambda_e+\Lambda_o)})$.
- Extinction receives the share $\Lambda_e/(\Lambda_e+\Lambda_o)$. With infinite terms the share is ½ if both are infinite, 1 if only $\Lambda_e$ is, and 0 otherwise (`flame6.rs:836-850`).

**Hinge member** (`flame7.rs:716-761`). Each class is integrated separately with the hazards taken at the midpoint of the step:

$$\lambda_o=\lambda_{ft}(t+h/2)-\tfrac1h\ln\min\!\big(1,S_{bt}(t_b)/S_{bt}(t_a)\big),\qquad \ell=s\,(1-e^{-(\lambda_{se}+\lambda_o)h})$$

- The loss is split $\lambda_{se}:\lambda_o$.
- A non-finite total puts all of $\ell$ into termination.

The weak-class probability at the cutoff (`flame7.rs:776-785`) is

$$p_w^\tau=\frac{p_wS^{(w)}(\tau)}{p_wS^{(w)}(\tau)+(1-p_w)S^{(m)}(\tau)}$$

- $S^{(\cdot)}(\tau)$ is each class run from 0 to $\tau$ with all its hazards.
- If the denominator is 0, $p_w^\tau=p_w$.
- The final table mixes the two classes with $p_w^\tau$ (`flame7.rs:789-795`).

**Emitted rows** at each $t_j$ (`flame6.rs:856-861`, `flame7.rs:796-801`). Let $s$, $F_e$ and $F_o$ be the active mass and the two cumulative incidences at $\min(t_j,t_{end})$. Define

- $\text{stopped}=[t_{stop}\text{ exists},\ t_j\ge t_{stop},\ t_{stop}\le t_{hor}]$;
- $\text{beyond}=[t_j\ge t_{hor}]\wedge\neg\text{stopped}$.

The four rows are:

$$\text{active\_supported}=\begin{cases}0&\text{stopped or beyond}\\ s&\text{else}\end{cases},\quad
\text{extinction}=F_e,\quad
\text{operator\_termination}=F_o+s\,\mathbf 1_{\text{stopped}},\quad
\text{unresolved\_support}=s\,\mathbf 1_{\text{beyond}}$$

So a known stop ends the fed phase: the hazards are off after it, and all mass still burning at the stop is counted as operator termination.

**Member refusals, in code order:**

- forward-hash mismatch (`flame6.rs:887-902`);
- release state Failed or Unsupported;
- a weighted limit $\le T_0$;
- a family mismatch;
- no confirmed established phase (`flame6.rs:930-935`);
- a stop at or before the cutoff (`flame6.rs:938-943`);
- condition and fuel gates;
- domain;
- $\tau\notin[0,t_{hor})$;
- no time $\ge\tau$;
- a non-finite or non-positive clock: $m_{cr}$, $D$, $r_{ss}$, $r_{ext}$, $\Phi$, $\tau_{osc}$, $\Delta T$ for flame6 (`flame6.rs:352-385`, `986-994`), and $r_{ss}$, $\Delta T$ for flame7 (`flame7.rs:333-356`);
- a zero survival at the cutoff: $S_{se}$, $S_{bt}$ or $S_{ps}$ for flame6 (`flame6.rs:995-1001`), and $S_{bt}$ for flame7 (`flame7.rs:934-939`).

#### 1.8 The committee (`committee3.rs:594-871`)

**Date and era** (`committee3.rs:469-505`, `640-653`). The test day is $d=1000\,\text{year}+\text{day of year}$. It is taken from the first valid `YYYY-MM-DD` in `session_id`; otherwise from `YYDDD` after the last `:` of `run_id`, read as year $2000+YY$; otherwise there is none.

$$p_{late}=\begin{cases}0& d\le d_{e}\\ 1& d\ge d_{l}\\ \texttt{era.p\_late\_between}& d_e<d<d_l\\ \texttt{era.p\_late\_unknown}&\text{no date}\end{cases}$$

The release is refused as unrunnable if $d_e\ge d_l$ or $p_{late}\notin[0,1]$ (`committee3.rs:621-629`, `654-662`).

**Script path** (`committee3.rs:666-681`). It is taken when all of these hold:

- the condition `programmed_stop` $T_{prog}$ is resolved, finite and $>0$;
- some entry of `mix.ws_early` or `mix.ws_late` is $>0$;
- some entry of `script.offset_w` is $>0$.

On the script path the weight pair is (`mix.ws_early`, `mix.ws_late`); otherwise it is (`mix.w_early`, `mix.w_late`).

**Slot weights** (`committee3.rs:573-582`, `683-714`). A vector whose era has non-zero probability must be $\ge0$ with a positive sum, or the forecast is refused. Each vector is divided by its sum, and then

$$w_i=\begin{cases}e_i&e_i=l_i\ (\text{exactly})\\ (1-p_{late})\,e_i+p_{late}\,l_i&\text{otherwise}\end{cases}$$

- When $p_{late}=1$ the early vector is neither checked nor used.
- When $p_{late}=0$ the late vector is neither checked nor used.

**Planned-stop nodes** (`committee3.rs:717-755`). For each $k$ with $\texttt{offset\_w}[k]>0$, where $o=\texttt{offset\_w}/\sum\texttt{offset\_w}$ and a negative entry is refused:

- the prefix loses `programmed_stop`;
- it gains a control stage `scheduled_stop` at $T_{prog}+\delta_k$, with $\delta_k=\texttt{script.offset\_s}[k]$.

**Mixture** (`committee3.rs:843-861`). For every row $c\in$ {active_supported, extinction, operator_termination, unresolved_support} and every time $t$:

$$P_c(t)=\sum_{k}o_k\sum_{i:w_i>0}w_i\,P^{(i,k)}_c(t)$$

- Off the script path there is a single pass with $o=1$, and a `programmed_stop` the prefix carries is left in place and not read.
- If exactly one member is run on one node, its table is returned bit for bit.
- A member refusal refuses the forecast, tagged with the slot and node (`committee3.rs:555-570`).
- Differing time grids are a hard failure (`committee3.rs:823-837`).

**What the code fixes.** These hold whatever the release types:

- Each member is a valid competing-risk table: $\sum_c P^{(i,k)}_c(t)=1$ up to rounding.
- The mixture is convex, so it is one too.
- A known scheduled stop at $T$ moves every member's remaining active mass into operator termination for $t\ge T$.

#### 1.9 The planned stop from the script file (`tools/wsp/ext/acme_script2.py`, research repo)

The tool reads only the pre-ignition parameter file `<inv>_Raw Data_as run<test>.txt` (`acme_script2.py:1-60`, `105-185`). All terms are in seconds.

**Stage sum $S$:**

- v1 build (Flame Design normal, 2019): $\texttt{stabilizationTime1Sec}+\texttt{stabilizationTime2Sec}+\texttt{ignitionToTestRampTimeSec}+\texttt{constantFlowTimeSec}$.
- v2 build (CFI-G and Flame Design inverse): the same first three terms plus $\texttt{constantFlowTime1Sec}+\texttt{testFlowRamp1To2TimeSec}+\texttt{constantFlowTime2Sec}+\texttt{testFlowRamp2To3TimeSec}+\texttt{constantFlowTime3Sec}$.

**Delay before retract:** $D=\texttt{igniterDelayBeforeRetractMs}/1000$.

**Fibre step $F$:**

- v2: $|\texttt{fiberTestPosition}-\texttt{fiberIgnitionPosition}|/\texttt{fiberVelocity}$, or 0 when the two positions are equal.
- v1: 8 s if `fiberMoveDuringTest` is true and the positions differ; otherwise 0.

**Igniter travel $G$:**

- v2: $(\texttt{igniterEndPosition}-\texttt{igniterStartPosition})/\texttt{igniterVelocityCountsPerSec}$, taken as an absolute value in the code;
- v1: 0.

$$T_{prog}=D+S+F-G\qquad(\text{forecast clock, origin at flame onset})$$

- A missing term gives branch `incomplete_file` and `planned_stop_s = None`.
- A file without a constant-flow stage gives `no_constant_flow_stage`.

The F2 header says the release "is meant for that input, and for the stage sum where the file is incomplete" (`configs/models/acme/wsp12/normal/f2-fable/edits.wsp:8-10`). Not verified: which tool writes $T_{prog}$, or the stage-sum fallback, into the prefix condition `programmed_stop`. I did not locate that adapter.

#### 1.10 F2 laws written out at their stored numbers

**Normal F2, slots b0–b3 (`flame6.rs`).** All four are identical except:

- `bt.limit_w`: b0 and b2 have $[0,1,0]$, so the limit is 510 °C; b1 and b3 have $[1,0,0]$, so the limit is 455 °C;
- b2 and b3 have all atoms off ($\texttt{atom\_logit}=-40$, so $p_a<10^{-9}$) and $\beta=e^{-9}$;
- b0 and b1 have atoms $T=(61.5,98,126.5,275.5)$ s with logits $(-2,-2.9,-2.2,0)$ and $\beta=e^{-7.4}$.

Here $c=(c_p,c_m,c_{o},c_x,c_T,c_z,f_e,f_b)$:

$$\ln m_{cr}=-1.715+0.5\max(0,\,c_p-c_o-0.9)-s_f$$

$$N_{diff}=3.92+0.45c_p-0.385c_o+0.2c_x+0.27f_b+0.21\max(0,c_o+5f_b-4.1)+0.18\max(0,c_o-5f_e-5f_b+0.9)$$

$$\operatorname{logit}p_w=-10.2-6.7c_T,\qquad \ln\tau_{osc}=13\ (\approx4.4\cdot10^5\ \text{s}),\qquad
\ln\Delta T=5.88-0.4b_{rss}-0.4b_{mtot}-0.15\max(0,c_x-0.5)$$

On the declared domain $c_o\le1.5$, so $\max(0,c_o-4.1)=0$ at vertices E and P.

**Normal F2, slots c0–c3 (`flame7.rs`).** The same pattern of limits and atoms applies. In addition:

- atom 0 reads $c_m$: its logit is $-2-2c_m$ (`c0.ft.atom_w[0,1]=-2`);
- $\beta=e^{-7.6}$ in c0 and c1, and $e^{-9}$ in c2 and c3.

With $c_m'=c_m+s_f$:

$$\begin{aligned}\ln\tau_R={}&2.43-0.28c_p+0.84c_o-0.2f_b+1.4\max(0,0.5-0.24c_p-c_m'+0.65c_o)+2.0\max(0,-0.22-0.24c_p-c_m'+0.65c_o)\\&-0.35\max(0,-5-c_x+5f_b)-0.14\max(0,-4.1+c_o+5f_b)-0.12\max(0,0.9+c_o-5f_e-5f_b)\end{aligned}$$

$$\operatorname{logit}p_w=-18-12c_T$$

Hidden unit 3 has `w2[3] = 0` and contributes nothing.

**Inverse F2, slots b0–b3 (`flame6.rs`).** All four have:

- `bt.limit_w` $=[0.4,0.6,0]$ on $[455,510]$ °C;
- $\sigma_{bt}=0.25$;
- the indicator-free laws

$$\ln m_{cr}=-4.6\ (0.01005\ \text{g O}_2\,\text{m}^{-2}\text{s}^{-1}),\qquad \ln D=3.332\ (27.99\ \text{mm}^2/\text{s})$$

$$\operatorname{logit}p_w=-5.33-5c_T-10\max(0,c_x-0.2)$$

$$\ln\tau_{osc}=2.17+0.45c_o+3c_x+10\max(0,c_x-0.3)-8\max(0,-1-c_m)+8\max(0,-0.6-c_o)\;[+20\max(0,c_m-0.6c_z-0.2)\ \text{in b1, b3}]+\texttt{dosc}_k$$

$$\ln\Delta T=5.88-0.4b_{rss}-0.4b_{mtot}+0.3f_e$$

For ethylene the burner rise is weighed $(0.5,0.5,0)$, which gives $+0.15$.

The scenarios differ as follows:

- b1 and b3 carry the extra quench hinge; b0 and b2 do not.
- b0 and b1 have atoms $T=(63,64,65,66,67,90)$ s with logits $(-2.94,-2.59,-2.2,0.49,-1.73,-1.39)$ and $\beta=e^{-5.5}$.
- b2 and b3 have the atoms off and $\beta=e^{-9}$.

Whether $\hat\Phi>1$ (a growth clock) at given conditions under $m_{cr}=0.01$ g m⁻² s⁻¹ was not evaluated here.

#### 1.11 Parameter tables, F2

**Committee tensors**, units 1 unless stated:

| symbol | edits.wsp name | meaning | normal F2 | inverse F2 |
|---|---|---|---|---|
| $e$ | `mix.w_early` | slot weights, early era, no planned stop | b0 = c0 = 1 | b0 = 2, b1 = 1 |
| $l$ | `mix.w_late` | same, late era | b1 = c1 = 1 | b0 = 2, b1 = 1 |
| $e^s$ | `mix.ws_early` | early era, planned stop | b2 = c2 = 1 | b2 = 2, b3 = 1 |
| $l^s$ | `mix.ws_late` | late era, planned stop | b3 = c3 = 1 | b2 = 2, b3 = 1 |
| $d_e$ | `era.last_early_day` | last early day (500 °C limit) | 2021235 | 1900001 |
| $d_l$ | `era.first_late_day` | first late day (450 °C limit) | 2021256 | 2100366 |
| | `era.p_late_between` | $p_{late}$ strictly between | 0.5 | 0.5 |
| | `era.p_late_unknown` | $p_{late}$ without a date | 0.15 | 0.5 |
| $\delta$ | `script.offset_s` | offsets of the planned stop (s) | $[-1,0,1]$ | $[-1,0,1]$ |
| $o$ | `script.offset_w` | node weights | $[0.25,0.5,0.25]$ | $[0.25,0.5,0.25]$ |

**Member tensors, normal F2.** In the table, "b" means slots b0–b3 and "c" means slots c0–c3.

| symbol | name | meaning | b | c | units |
|---|---|---|---|---|---|
| $\rho D$ | `se.rho_d_g_m_s` | density × diffusivity in $r_{ss}$ | 0.0334 | 0.0334 | g m⁻¹ s⁻¹ |
| $\sigma_\phi$ | `se.sigma_phi` | sd of $\ln\Phi$ | 0.25 | – | 1 |
| $n$ | `num.phi_nodes` | reduced-flow nodes | 15 | – | – |
| | `num.tau_cap_s` | growth-clock cap | $10^5$ | – | s |
| $t_{fl}$ | `se.floor_s` | establishment floor | 4 | 4 | s |
| $k_{se}$ | `se.shape` | growth / radiative clock shape | 4 | 3 | 1 |
| $k_{osc}$ | `se.osc_shape` | quench-clock shape | 3 | – | 1 |
| | `se.hold_log_rate` | ln hold hazard | −8 | – | ln s⁻¹ |
| $\tau_w,k_w$ | `se.flash_tau_s`, `se.flash_shape` | weak-class clock | 1, 4 | 1, 4 | s, 1 |
| $\tau_b$ | `bt.tau_s` | burner heating time constant | 30 | 30 | s |
| $T_0$ | `bt.t0_c` | burner temperature at ignition | 30 | 30 | °C |
| $\sigma_{bt}$ | `bt.sigma` | sd of $\ln\Delta T$ | 0.12 | 0.12 | 1 |
| $k_{bt}$ | `bt.shape` | limit-time shape | 8 | 8 | 1 |
| $n_b$ | `num.bt_nodes` | heating nodes | 7 | 7 | – |
| $T_{lim}$ | `bt.limit_c` | limits | [455, 510, 0] | [455, 510, 0] | °C |
| $\bar w$ | `bt.limit_w` | limit weights | early [0,1,0]; late [1,0,0] | same | 1 |
| $T_a$ | `ft.atom_t_s` | programmed-stop atoms | [61.5, 98, 126.5, 275.5, 0…] | same | s |
| | `ft.atom_logit` | atom logits | b0/b1: [−2, −2.9, −2.2, 0, −40…]; b2/b3: all −40 | same pattern | 1 |
| | `ft.atom_w` | atom slopes | 0 | [0,1] = −2, else 0 | 1 |
| $\ln\beta$ | `ft.log_base` | background termination | −7.4 (b0, b1); −9 (b2, b3) | −7.6 (c0, c1); −9 (c2, c3) | ln s⁻¹ |
| | `num.atom_half_s` | atom half-width | 0.5 | 0.5 | s |
| | `num.dt_s`, `num.horizon_s` | step, horizon | 0.1, 600 | 0.1, 600 | s |
| | `norm.p_ref_pa` | $p_{ref}$ | 101325 | 101325 | Pa |
| | `norm.mdot_ref_kg_s` | $\dot m_{ref}$ | $10^{-6}$ | $10^{-6}$ | kg/s |
| | `norm.o2_ref`, `norm.o2_scale` | | 0.30, 0.10 | same | 1 |
| | `norm.xf_ref` | | 0.30 | same | 1 |
| | `norm.tad_ref_k`, `norm.tad_scale_k` | | 2200, 300 | same | K |
| | `norm.zst_ref`, `norm.zst_scale` | | 0.25, 0.20 | same | 1 |
| | `norm.r_ref_mm` | | 25 | same | mm |
| | `norm.mtot_ref_kg_s` | | $3\cdot10^{-6}$ | same | kg/s |
| | `phys.mw_n2`, `phys.mw_o2` | | 28.013, 31.998 | same | g/mol |
| networks | `se.flux.*`, `se.diff.*`, `se.flash.*`, `se.osc.*`, `bt.rise.*`, `se.tau.*` | as written in §1.10 | | | |

**Member tensors, inverse F2** (b0–b3). Where a value differs from normal F2:

| name | value |
|---|---|
| `se.sigma_phi` | 0.3 |
| `se.floor_s` | 2 s |
| `se.shape` | 4 |
| `se.flash_tau_s`, `se.flash_shape` | 1.5 s, 3 |
| `bt.sigma` | 0.25 |
| `bt.limit_w` | $[0.4,0.6,0]$ |
| `ft.atom_t_s` | $[63,64,65,66,67,90,0,0]$ |
| `ft.log_base` | −5.5 (b0, b1); −9 (b2, b3) |
| `norm.mdot_ref_kg_s` | $5\cdot10^{-6}$ |
| `norm.o2_ref` / `o2_scale` | 0.5 / 0.25 |
| `norm.xf_ref` | 0.1 |
| `norm.zst_ref` | 0.45 |

These are unchanged from normal F2:

- `se.rho_d_g_m_s` = 0.0334, `se.hold_log_rate` = −8, `se.osc_shape` = 3;
- `bt.tau_s` = 30, `bt.t0_c` = 30, `bt.shape` = 8;
- `num.*`.

---

### 2. `acme_normal.wsp13` F3 stage 1 (`committee4.rs`)

Release `sha256:d1660331…45aa`, at `configs/models/acme/wsp13/normal/f3-s1`. Model key `acme_normal.wsp13` runs `Forward::Committee4` (`crates/ignis-wsp/src/models.rs:374-377`), which is defined for normal flames only (`committee4.rs:172-175`).

The committee rules of §1.8 hold unchanged. These are the ten slots, the four weight vectors, the era rule, the three planned-stop offset nodes, the weighted mean, the bit-for-bit single-member case and refusal propagation; they now sit in `forecast_once` (`committee4.rs:697`). Only the member kinds change:

| Slots | Member forward | Laws |
|---|---|---|
| a0, a1 | `flame8::forecast` | hinge |
| b0–b3 | `flame9::forecast` | growth + heating |
| c0–c3 | `flame10::forecast` | hinge + heating |

Source: `committee4.rs:10-13`; the calls are at `:875`, `:892` and `:909`.

The prefix may carry six more keys: `diluent_species`, `fuel_blend_x`, `burner_limit_c`, `x_o2_role_b`, `tad_b` and `zst_b` (`committee4.rs:90-106`).

#### 2.1 Two oxygen candidates

The second candidate is $(x_{O_2}^{(b)},T_{ad}^{(b)},Z_{st}^{(b)})$ (`committee4.rs:80-85`). The forward builds two prefixes (`committee4.rs:612-623`):

- $\mathcal P_1$ is the prefix without the three `_b` keys;
- $\mathcal P_2$ is the prefix with `x_o2_role`, `tad` and `zst` replaced by the `_b` values.

Each gets the full committee forecast (`:657`, `:661`), and the two are mixed:

$$P_c(t)=(1-w_b)\,P^{(1)}_c(t)+w_b\,P^{(2)}_c(t),\qquad w_b=\texttt{mix.o2\_b\_w}$$

(`committee4.rs:649`, `:670-677`). Branches:

- With none of the three keys, the committee is formed once on the prefix as given (`:637-639`).
- With one or two of the three keys, the forecast is refused at the condition gate (`:640-647`).
- If either candidate is refused, the forecast is refused, tagged "oxygen candidate 1/2".
- If the two candidates return different time grids, the forward fails (`:665`).

F3-s1 types $w_b=0.5$. Its edits.wsp says this weight is not read in stage 1, because the tests with two oxygen candidates are not handed to it.

#### 2.2 The second fuel table and the diluent table (`fuel2.rs`)

- The six fuel rows are unchanged.
- The bound on `fuel.s` widens to $(-3,3)$ (`fuel2.rs:110-118`).
- Five columns are added: `fuel.sigma`, `fuel.dtad_k`, `fuel.k_rel`, `fuel.zst0` and `fuel.gz` (`:123-133`).
- A diluent table is added, with rows 0 nitrogen and 1 helium (`:134-151`).

**Reading the fuel.** A single species is read as its row (`:284-302`). A blend `a+b` with mole fraction $x$ of `a` (`fuel2.rs:305-335`) is mixed as

$$y_a=\frac{xW_a}{xW_a+(1-x)W_b},\quad w_a=\frac{y_a\sigma_a}{y_a\sigma_a+(1-y_a)\sigma_b},\quad W=xW_a+(1-x)W_b,\quad c=w_ac_a+(1-w_a)c_b$$

- The rule for $c$ applies to `s`, `se_coord`, `ft_coord`, `dosc`, `tad_min_k`, `dtad_k`, `zst0` and `gz`.
- `k_rel` is kept per species.

A blend is refused at the condition gate when (`:404-477`):

- a row has `admit`$\neq1$;
- a species is not in the table;
- `fuel_blend_x` is missing or outside $[0,1]$;
- `fuel_blend_x` is given for a single fuel;
- a stored indicator is nonzero.

**Reading the diluent.** `diluent_species` absent means nitrogen. Any text other than `nitrogen` or `helium` is refused, and so is `dil.admit`$=0$ (`:480-495`).

**Flame temperature** (`fuel2.rs:513-519`), with $T_0=\texttt{phys.t\_stream\_k}=298.15$ K:

$$T_1=\begin{cases}T_{ad}, & Le_{O_2}=1\\[2pt] T_0+(T_{ad}-T_0)\dfrac{1-(1-Z_{st})^{1/Le_{O_2}}}{Z_{st}}, & \text{otherwise}\end{cases}$$

- With $Le_{O_2}\neq1$, $Z_{st}\notin(0,1)$ is refused (`:572`).
- $T_1<\texttt{fuel.tad\_min\_k}$ is refused (`:522-529`, `:578-580`).
- A non-finite `burner_limit_c` is refused (`:498-508`).

**Burner-stream conductivity** (`fuel2.rs:535-541`):

$$\kappa=\sum_{i\in\text{fuel}}x_f\,x_i\,(k_{rel,i}-1)+(1-x_f)(k_{rel,d}-1)$$

#### 2.3 Growth members `flame9.rs` against `flame6.rs`

The laws, the vertex weighing and everything after the clocks are those of `flame6.rs`. The edits after the clocks are only at `flame9.rs:999` (limits) and at `:193` and `:235` (inputs). Below, $d$ is the diluent row and $f$ the fuel row or blend.

**Condition feature** (`flame9.rs:827`):

$$c_{tad}=(T_1+\texttt{fuel.dtad\_k}-T_{ref})/T_{scale}$$

**Reactant mass fraction** (`:810`):

$$Y_R=\frac{x_RW_R}{x_RW_R+(1-x_R)\,\texttt{dil.mw\_g\_mol}[d]}$$

**Critical flux and diffusivity** (`flame9.rs:597-602`):

$$\ln m_{cr}=N^*_{se}(\texttt{se.flux};c)-s_f+\texttt{dil.dflux}[d]-\texttt{fuel.gz}\,\max(0,\,Z_{st}-\texttt{fuel.zst0})$$

$$\ln D=N^*_{se}(\texttt{se.diff};c)+\ln\texttt{dil.d\_far}[d]\quad(\text{skipped when }d_{far}=1)$$

**Quasi-steady radius** (`:474-479`, `:606-607`):

$$r_{ss}=\frac{\dot m_{tot}}{4\pi\,(\rho D)\,\texttt{dil.rho\_d}[d]\,L}\quad(\text{factor skipped when exactly }1)$$

**Burner side.** $b_2=(T_1-T_{ref})/T_{scale}$, without `dtad` (`:621`). With $T_b=\texttt{bt.t0\_c}+273.15$ (`flame9.rs:633-640`):

$$\ln\Delta T_{eq}=N^*_{ft}(\texttt{bt.rise};b)+\texttt{bt.kappa\_exp}\,\mathrm{ln1p}(\kappa)+\ln\frac{T_1-T_b}{\texttt{bt.tflame\_ref\_k}-T_b}$$

- The last term is omitted when `bt.tflame_ref_k`$=0$.
- Otherwise $T_1\le T_b$ is refused (`:795-800`).

**Script limit as an input** (`flame9.rs:545-550`, `:642`). When the prefix carries `burner_limit_c` $=L$:

$$T_{lim,l}=\texttt{bt.limit\_c}[l]+(L-\texttt{bt.limit\_script\_c})$$

The check that a weighted limit lies above `bt.t0_c` is applied to the shifted value (`:554-566`, `:999`).

With nitrogen, inert new columns, $\kappa=0$, `tflame_ref`$=0$ and no $L$, the forecast equals `flame6.rs` bit for bit (`flame9.rs:30-33`).

#### 2.4 Hinge members with the heating law (`flame10.rs`) against `flame7.rs`

The radiative clock gains the diluent shifts (`flame10.rs:533-539`):

$$\ln\tau_R=N^*_{se}\big(\texttt{se.tau};\,c\text{ with }c_{logm}+s_f+\texttt{dil.dlogm}[d]\big)+\texttt{dil.dlnr}[d]$$

The other edits match `flame9.rs`:

| Edit | Where in flame10.rs |
|---|---|
| $c_{tad}$ with $T_1+dtad$ | `:750` |
| $Y_R$ with `dil.mw_g_mol` | `:733` |
| $r_{ss}$ with `dil.rho_d` | `:437`, `:547-548` |
| $b_2$ from $T_1$ | `:553` |
| $\kappa$ and flame-temperature terms in $\ln\Delta T_{eq}$ | `:565-572` |
| $T_1\le T_b$ refusal | `:719` |
| shifted limits | `:501-505`, `:574`, `:928` |

- `fuel.gz`, `fuel.zst0`, `dil.dflux` and `dil.d_far` are not read by this law.
- `flame8.rs` adds only the $T_1$, `dtad`, `dlogm` and `dlnr` lines (`flame8.rs:8-20`). Its slots have weight 0 in F3-s1, so it is not run.

#### 2.5 Which slots run in F3-s1, and what the stored numbers do

The committee lines are those of normal F2 (`edits.wsp:13-22`):

| Path | Early era | Late era |
|---|---|---|
| No planned stop | b0, c0 | b1, c1 |
| Planned stop | b2, c2 | b3, c3 |

The offsets are $\{-1,0,1\}$ s with weights $\{0.25,0.5,0.25\}$.

**Comparison with F2** (`params.npz` of F3-s1 against `55fcf4cb…`, slots b0–c3). Every shared tensor is identical except `fuel.s`. The new tensors are:

- the five fuel columns;
- the diluent table;
- `bt.kappa_exp`, `bt.tflame_ref_k` and `bt.limit_script_c`;
- `phys.t_stream_k`.

**What the stored numbers do:**

- **The conductivity term is inactive.** Every `k_rel` is 1, so $\kappa=0$, and the typed `kappa_exp` 0.45 has no effect.
- **The adiabatic temperature reaches the burner only through the flame-temperature factor.** Nitrogen has $Le=1$, so $T_1=T_{ad}$. `bt.rise` has zero weight on $b_2$, so
  $$\ln\Delta T_{eq}\mathrel{+}=\ln\frac{T_{ad}-303.15}{2346.85}.$$
- **Only `se.flash` reads $c_{tad}$**, through `skip[4]`. Methane's `dtad` $=-350$ K therefore shifts the weak-class logit by
  $$\texttt{skip}[4]\cdot(-350/300)=+7.82\ \text{(b, skip}=-6.7),\qquad +14.0\ \text{(c, skip}=-12).$$
- **The script limit enters as an offset:**
  - early slots: limit 510 with script setting 500, so $T_{lim}=L+10$;
  - late slots: limit 455 with script setting 450, so $T_{lim}=L+5$;
  - without $L$ the limits are 510 and 455 °C, as in F2.
- **The ethylene dilution term acts only in the b slots**, as $-1.0\max(0,Z_{st}-0.30)$ on $\ln m_{cr}$. The c slots have `gz`$=0$.

#### 2.6 Parameter table, F3-s1, running slots

Tensors not listed are F2's slot values (§1.11). Fuel rows are given as [ethane, propane, butane, ethylene, methane, hydrogen].

| symbol | edits.wsp name | meaning | b0–b3 (flame9) | c0–c3 (flame10) | units |
|---|---|---|---|---|---|
| $w_b$ | `mix.o2_b_w` | weight of the second oxygen candidate | 0.5 (committee) | | 1 |
| $\beta_\kappa$ | `bt.kappa_exp` | power of the stream conductivity | 0.45 | 0.45 | 1 |
| $T_{f,ref}$ | `bt.tflame_ref_k` | reference flame temperature of the heating law | 2650 | 2650 | K |
| $T_{script}$ | `bt.limit_script_c` | script limit the member's limits are relative to | 500 (b0, b2); 450 (b1, b3) | 500 (c0, c2); 450 (c1, c3) | °C |
| $T_0$ | `phys.t_stream_k` | stream temperature (constant) | 298.15 | 298.15 | K |
| $s_f$ | `fuel.s` | ln oxygen demand per gram vs ethane | [0, 0, 0, −0.02, **−0.35**, 0.757] | [0, 0, 0, −0.31, **0.35**, 0.757] | 1 |
| $\Delta T_f$ | `fuel.dtad_k` | weak-flame temperature shift | [0, 0, 0, 0, −350, 0] | same | K |
| $Z_{st,0}$ | `fuel.zst0` | threshold of `gz` | [1, 1, 1, **0.30**, 1, 1] | all 1 | 1 |
| $g_z$ | `fuel.gz` | fall of $\ln m_{cr}$ per unit $Z_{st}$ | [0, 0, 0, **1.0**, 0, 0] | all 0 | 1 |
| $k_{rel,f}$ | `fuel.k_rel` | conductivity relative to N₂ | all 1 | all 1 | 1 |
| $\sigma_f$ | `fuel.sigma` | oxygen demand per gram (blend shares) | [3.725, 3.628, 3.579, 3.422, 3.989, 7.936] | same | g/g |
| | `fuel.admit` | answered fuels | [1, 1, 1, 1, 1, **0**] | same | 1 |
| | `fuel.mw_g_mol`, `se_coord`, `ft_coord`, `tad_min_k`, `dosc` | as in F2 | | | |
| | `dil.admit` | [N₂, He] | [1, **0**] | same | 1 |
| $W_d$ | `dil.mw_g_mol` | diluent molar mass | [28.013, 28.013] | same | g/mol |
| | `dil.rho_d`, `d_far`, `le_o2`, `k_rel` | diluent factors | all 1 | same | 1 |
| | `dil.dflux`, `dlogm`, `dlnr` | diluent shifts | all 0 | same | 1 |

The rationale is in `fuel_tables/normal-f3s1-{growth,hinge,diluent}.wsp` and in `edits.wsp:27-31`.

**Status.** The header reads: DATA-INFORMED, "numbers by claude-fable-5-2 (phys_sflame_f3.md E4), transcribed by claude-opus-5-5 without choosing any number". PROTOCOL 13.5 (research repo, `docs/wsp/PROTOCOL.md:1621-1731`):

- registers it as data-informed from the train days of s-Flame, CFI-G and Flame Design;
- scores it on the 46 s-Flame validation nitrogen cases (13.5(a));
- checks it on the CFI-G and Flame Design validation data (13.5(b));
- adopts it in place of F2.

The 13.6 notes add that its methane step is an in-sample result, because no held-out nitrogen days remain.

#### 2.7 F4 rows: methane, helium, hydrogen (comparators, PROTOCOL 13.6)

The chain is f3-s1 → `f4-methane` (`57fe4255…ebe2`) → `f4-helium` (`370ed3cb…62c0`) → `f4-hydrogen` (`4cc2fa60…9436`). All use the wsp13 forward and committee lines, and all are DATA-INFORMED from the s-Flame development days (train and validation), transcribed from `phys_sflame_f4.md` §6. The changes below are from `params.npz` diffs.

**f4-methane.** `fuel.s[methane]` is −0.6 in b0–b3 (from −0.35) and 1.0 in c0–c3 (from 0.35).

**f4-helium**, helium row of the diluent table in all ten slots:

| Column | Value |
|---|---|
| `admit` | 1 |
| `mw_g_mol` | 4.0026 |
| `rho_d` | 1.15 |
| `d_far` | 3.5 |
| `le_o2` | 1.37 |
| `k_rel` | 5.8 |
| `dflux` | 1.15 |
| `dlogm` | 0.6 |
| `dlnr` | −0.75 |

This turns on terms that are idle for nitrogen:

- $T_1$ comes from the Lewis-number formula with $Le=1.37$;
- $\kappa=(1-x_f)\cdot4.8$, so $\ln\Delta T_{eq}\mathrel{+}=0.45\,\mathrm{ln1p}(\kappa)$;
- $\ln m_{cr}\mathrel{+}=1.15$ and $\ln D\mathrel{+}=\ln3.5$;
- $c_{logm}\mathrel{+}=0.6$ and $\ln\tau_R\mathrel{-}=0.75$ in the hinge members.

The helium days are answered as a mixture over 21 % and 30 % oxygen through the two-candidate path of §2.1.

**f4-hydrogen**, hydrogen row of the fuel table in all slots:

| Column | Value |
|---|---|
| `admit` | 1 |
| `se_coord` | (1, 0) |
| `ft_coord` | (1, 0) |
| `tad_min_k` | 1100 |
| `dtad_k` | +600 |
| `k_rel` | 7.0 |
| `fuel.s`, growth members | 1.9 |
| `fuel.s`, hinge members | −2.0 |

Its edits.wsp says blends are refused by the lane's adapter (`sflame_cases.py`), not by the release. This was not verified, and `fuel2.rs` itself mixes a blend whose two rows are both admitted.

By 13.6 all three releases are comparators, and F3 stage 1 stays the model.

---

### 3. Reference baselines

#### 3.1 `wsp12/*/baseline-set`

| Family | Id |
|---|---|
| normal | `sha256:0d4188e189d52e69b60bf5cde4474f29cde3141382cd66bb4c199ac6ec92bb1b` |
| inverse | `sha256:d33a625b66980212a91a42323d727973f5bd7a830c68de3216782de60ccd9634` |

Each is "not a candidate": the lineage root of the spec. It was copied by `tools/wsp/committee3_edits.py` from:

- the roots `acme_*.wsp9 baseline-clock` (slots a);
- `acme_*.wsp10 baseline-theory` (slots b);
- `acme_*.wsp11 baseline-clock-theory` (slots c).

Each uses the neutral fuel table. Its meta.json says no number came from the PSI-159 tables.

**Committee lines:**

- `mix.w_early` = `mix.w_late` = $[1,0,1,0,\dots]$, so a0 and b0 get weight ½ each, exactly, in every era;
- `mix.ws_*` $=0$, so the planned stop is never read and there is one pass on the prefix as given;
- offsets $[0,0,0]$ with weights $[1,0,0]$, unused.

**Neutral fuel table:**

- `admit` $=[1,1,1,0,0,0]$, so ethylene, methane and hydrogen are refused at the condition gate;
- alkane coordinates (1,0), (0,0), (0,1);
- `fuel.s`, `dosc` and `tad_min_k` are all 0.

So the forecast is

$$P_c(t)=\tfrac12P^{(a0)}_c(t)+\tfrac12P^{(b0)}_c(t)$$

**Member a0 (`flame5.rs`).** The stored values are the same for both families except `norm.*`. Every network has only its bias:

- $\tau_R=e^{3.4}=29.96$ s with shape $k_{se}=1$ and floor 0;
- burner-limit clock $\tau_B=e^{4.6}=99.48$ s with shape $k_{ft}=1$, the `ft.tau` law of `flame5.rs` (`flame5.rs:358-372`);
- weak class $p_w=\sigma(-40)\approx4.2\cdot10^{-18}$;
- no atoms ($T_a=0$);
- $\beta=e^{-40}$.

With $k=1$, $\lambda_{LL}(t;\tau,1)=1/(\tau+t)$. Up to the $e^{-40}$ terms and the midpoint quadrature (`flame5.rs:468-501`), the two competing causes therefore have

$$\lambda_e(t)=\frac{1}{29.96+t},\qquad \lambda_o(t)=\frac1{99.48+t},\qquad S(t\,|\,\tau)=\frac{(29.96+\tau)(99.48+\tau)}{(29.96+t)(99.48+t)}$$

$$F_e(t)=\int_\tau^t\lambda_eS,\qquad F_o(t)=\int_\tau^t\lambda_oS$$

These do not depend on any condition.

**Member b0 (`flame6.rs`), the "published theory".** This is the root of `wsp3/*/baseline-theory`. Its header cites the sources below; the citations are the author's and were not checked here. The laws are

$$r_{ss}=\frac{\dot m_{tot}}{4\pi\cdot0.0334\,L}\quad(\text{Kim et al. 2023})$$

$$m_{cr}=e^{-1.609}=0.200\ \text{g fuel m}^{-2}\text{s}^{-1}\ (\text{normal; Irace et al. 2023, ethylene, 1 bar}),\qquad e^{-0.386}=0.680\ \text{g O}_2\ \text{m}^{-2}\text{s}^{-1}\ (\text{inverse})$$

$$D=e^{3.332-c_{logp}}=27.99\ \tfrac{p_{ref}}{p}\ \text{mm}^2/\text{s}$$

The other stored values:

- $\sigma_\phi=0.2$, 15 nodes, $k_{se}=4$, $t_{fl}=0$;
- quench clock $e^{13}$ s;
- hold $e^{-40}$;
- weak class $\sigma(-40)$;
- `bt.limit_w` $=0$, so $S_{bt}\equiv1$;
- stops off, $\beta=e^{-40}$.

Its forecast is thus the erfc growth clock of §1.3 alone, averaged over 15 log-normal nodes of $\Phi$. A node with $\Phi_i\le1$ never extinguishes, apart from the $e^{-40}$ and $e^{13}$-s terms. Flow termination is nearly zero except at a known `scheduled_stop` control, which the member reads if the prefix carries one.

#### 3.2 `baseline-theory`

`baseline-theory` exists for wsp10 (`configs/models/acme/wsp10/{normal,inverse}/baseline-theory`) and wsp3, not for wsp12. The wsp10 release is the b0 member above as a stand-alone `flame6.rs` release (header: copied from `wsp3/normal/baseline-theory` `sha256:3d8d3b7a…15ad` with the neutral table). Its forecast is $P^{(b0)}$ alone.

#### 3.3 `wsp13/normal/baseline-set` (`sha256:9e2c1816…6591`)

It is copied by `tools/wsp/committee4_edits.py` from the wsp12 root slots, with:

- the neutral fuel and diluent tables;
- `bt.kappa_exp` $=0$ and `bt.tflame_ref_k` $=0$;
- the same committee lines (a0 = flame8 and b0 = flame9 at ½ each);
- `mix.o2_b_w` $=0.5$.

b0 keeps `bt.limit_w` $=0$, so a `burner_limit_c` input shifts limits that have no weight. The a0 member was not dumped.

#### 3.4 The incidence baseline

PROTOCOL also scores against an "incidence baseline": "for each cutoff, the fraction of CFI-G train cases of the same family that had ended by each horizon, by cause" (`docs/wsp/PROTOCOL.md:601-603`, research repo). It is not a release of these specs, and its code was not read here.

---

### 4. From prefix to score

**Cases.** A case is a pair (test, cutoff $\tau$) with $\tau\in\{1,3,5\}$ s after the flame onset, the flame still burning (PROTOCOL §11, `docs/wsp/PROTOCOL.md:633`, research repo). The harness passes `--cutoffs 1,3,5` (`tools/wsp/dev_eval.py:283`).

**The prefix.** The prefix (a `PrefixBundle`) holds:

- the conditions of §1.1;
- the fuel;
- optionally `programmed_stop` (§1.9);
- the control stages known at the cutoff;
- the established-phase stamp.

ACME prefixes carry no time series (`flame6.rs:41`). Nothing after the cutoff and none of the forbidden keys (`flame6.rs:100-110`: burn rate, peak burner temperature, duration, extinction time, and others) is admitted by the prefix policy.

**Times.** `settings.times_s` = $[6,8,10,12,15,20,25,30,40,50,60,80,100,120]$ s on the forecast clock (`tools/wsp/dev_eval.py:24`, research repo). All of these are $\ge\tau$, so none is dropped.

**What is emitted** (`committee3.rs:873-918`):

- an artifact of kind `EventProbabilities`, with the four rows of §1.7 at those times;
- the cumulative incidence curves $F_e(t)$ (extinction) and $F_o(t)$ (operator/flow termination, including the burner limit, the programmed stops, the background and a known stop);
- the active mass and the unresolved mass;
- diagnostics: each member's clocks, weight and node weight, $p_{late}$, the date source and the offset nodes.

The uncertainty kind is `assumed_noise`.

**How uncertainty is represented.** There is no posterior and no sampling. The forecast distribution comes only from typed structure:

1. equally likely quadrature nodes over $\ln\Phi$ ($n=15$, sd $\sigma_\phi$) and over $\ln\Delta T$ ($n_b=7$, sd $\sigma_{bt}$);
2. latent weak and main classes, with the weak class re-weighted by survival to $\tau$;
3. a discrete mixture over burner-limit settings ($\bar w_l$);
4. programmed-stop atoms (point masses $p_a$ spread over $\pm0.5$ s);
5. the committee mixture over member kinds and era scenarios ($w_i$ with $p_{late}$);
6. three readings of the planned stop ($o_k$).

Each source is a convex mixture of competing-risk tables.

**Score** (`crates/ignis-lab/src/eval/scores.rs:326-375`, `crates/ignis-lab/src/eval/scoring.rs:560-660`). For each cause $c\in\{$extinction, operator_termination$\}$ and horizon $h$:

$$\mathrm{BS}_c(h)=\frac1n\sum_{i\,\text{eligible}}w_i\big(\mathbf 1\{T_i\le h,\ \text{cause}_i=c\}-F_{c,i}(h)\big)^2$$

- The weights are IPCW with a Kaplan–Meier censoring estimate (Graf/Gerds): $w_i=1/\hat G(T_i)$ for an event before $h$, and $1/\hat G(h)$ otherwise.
- A case censored before $h$ has weight 0 but stays in $n$.
- $F_{c,i}(h)$ is interpolated from the emitted row (`interp_row`).

$$\mathrm{IBS}_c=\frac{1}{h_{last}-h_0}\sum_{m}\frac{(h_m-h_{m-1})(\mathrm{BS}_c(h_m)+\mathrm{BS}_c(h_{m-1}))}{2}$$

This is the trapezoid over 6 to 120 s with NaN entries dropped (`scores.rs:362-375`).

So the forward's emitted CIF rows are exactly the quantities scored. Lower $\mathrm{IBS}$ for extinction and for termination is the target.

---

### 5. Zero-shot versus data-informed

Sources: `docs/wsp/README.md:63-64`, research repo; release `meta.json`; fuel-table headers. Every release is `state = experimental`, `trained_on = not_trained` and `construction_method = literal_revision`.

**Normal F2: data-informed.** Its exposure lists CFI-G train (conditions, endpoints, post-facts, raw channels, script files) and aggregate validation scores of earlier candidates. The changes from F1 are:

- the ethylene row, revised once from the Flame Design train days after the blind F1 run (PROTOCOL 12.8, 12.10): `se_coord` (1,0), hinge shift −0.31, growth shift −0.02;
- the planned stop from `acme_script2.py`;
- the era days 2021235 and 2021256 from the script log.

The methane row is unchanged from F1. F1's tables were typed from `phys_spherical_fuels.md` and are described as zero-shot for the new fuels with respect to per-test outcomes, but "not blind to the aggregate arms of 11.3" (PROTOCOL 12.8).

**Inverse F2: data-informed** (train, published theory), with these parts:

- inverse F1 / wsp5 P1 members, unchanged;
- the ethylene row zero-shot with respect to Flame Design: "no per-test outcome was read" (`fuel_tables/inverse-f1.wsp:3-4`). It is not blind to the aggregate arms, and it was left unchanged after the blind run;
- the only change from F1 is the planned-stop computation.

**F3-s1 and F4:** data-informed (§2.6, §2.7).

**Baselines:** reference roots, not candidates. They take no number from PSI-159. The published-theory member uses literature constants.

**Scores** reported in README (not recomputed here):

| Release | Data | Without planned stop (extinction / termination) | With planned stop |
|---|---|---|---|
| normal F2 | CFI-G validation, alkanes | 0.154 / 0.096 | 0.147 / 0.080 |
| normal F2 | Flame Design validation days, ethylene | 0.032 / 0.393 | 0.007 / 0.072 |
| inverse F2 | alkanes | 0.129 / 0.088 | 0.135 / 0.048 |

---

### 6. Not determined here

- Which adapter writes `programmed_stop`, and the stage-sum fallback, into the prefix. Only the computation in `acme_script2.py` was read.
- Whether $\hat\Phi>1$ (an active growth clock) at typical conditions in inverse F2, where $m_{cr}=0.01$ g m⁻² s⁻¹. No forward was run for this document.
- Exactly how the IPCW status and causes are derived from the event labels, beyond `scores.rs:280-358`.
- The code of the incidence baseline.
- The `flame8.rs` a0 parameters of the wsp13 baseline-set.
- Whether CFI-G prefixes ever carry `burner_limit_c`.
- The s-Flame case lists behind the PROTOCOL 13.5 and 13.6 numbers. Those numbers are quoted, not recomputed.
- The literature citations in the baseline-theory header.


## Part 3. BASS flame front and the Saffire transfer

Source of every formula: the frozen Rust forwards in `crates/ignis-wsp/src/` (byte-identical in `ignis-web` and in the research repo at `542dad8e`; checked with `cmp` for `front.rs`, `front3.rs`, `front4.rs`, `front5.rs`, `front_tools.rs`, `bass.rs`, `nn.rs`, `common.rs`). Typed numbers come from the `edits.wsp` of each release folder under `configs/models/bass/`. Paths are relative to the `ignis-web` root unless marked *research repo*. Nothing below is fitted: every number was typed by an author model, and the forward only evaluates it (`ForwardProfile::StrictNoFitting`, `front.rs:244`; `fit` bails, `front.rs:1079-1081`).

### Standing of each model

| Model | Release | How written |
|---|---|---|
| `bass_front.wsp` D1 (adopted) | `sha256:ac15e49f…855c`, `configs/models/bass/front/d1-fable/` | Data-informed. The author read the front tracks of four train burns (T5, T6, T16, T10) and a review of published concurrent-spread theory (`docs/wsp/research/lit_flame_spread.md`), and also the *inputs only* (no positions, times or outcomes) of the 17 validation tracks (`d1-fable/edits.wsp:1-10`). README: "data-informed (four train burns, published theory); the validation burns were measured blind" (*research repo* `docs/wsp/README.md:65`). |
| `bass_front.wsp` persistence | `sha256:ce53fe1f…de46`, `configs/models/bass/front/baseline-persistence/` | Reference baseline, not a candidate. Written from the forward alone (`release.json` exposure: `crates/ignis-wsp/src/front.rs` only). |
| `bass_front.wsp5` Q0 | `sha256:00a8bc35…9e72`, `configs/models/bass/front5/q0-fable/` | Tip law: D1's stored values, copied (data-informed). Boundary law: zero-shot, from `docs/wsp/research/phys_quench_w0.md` §6.4 (`q0-fable/header.wsp:4-11`). |
| `bass_front.wsp3` joint (Astra) | `sha256:79d27b02…a1e0`, `configs/models/bass/front3/joint-astra/` | Data-informed, not zero-shot (`joint-astra/CARD.md:3`). |
| `bass_front.wsp4` window-matched (Astra) | `sha256:01a4edf4…c87a`, `configs/models/bass/front4/window-matched-astra/` | Same numbers as the joint, with a new observation operator. |
| `bass.wsp` W0 (phase 1) | `sha256:9ed0451d…9e0a`, `configs/models/bass/wsp/w0-fable/` | Zero-shot: "No BASS contour, calibration or video was looked at" (`w0-fable/edits.wsp:2-4`). |

The Saffire transfer (section 4) is blind. No weight changed (`"weights_changed": false`, *research repo* `docs/wsp/feedback/saffire-transfer/freeze.json`). The tracks were measured by a sub-agent the author never read from. Only the reader rule for `geometry.sample_width_m` was added (`front_tools.rs:195-198, 312-319`).

---

### 1. `bass_front.wsp` D1 (`front.rs`)

#### 1.1 State, units, clock

Positions are in **sample widths** $w$, measured from the upstream (igniter) edge, positive downstream. Speeds are in $w/\mathrm{s}$ and the flow $U$ in cm/s. Time $t$ is in seconds after the igniter is switched off. The igniter is switched on at $t_0\le 0$ (`front.rs:247-248`, `front.rs:414-431`). The flame is three places (`front.rs:7-16`):

- pyrolysis front $x(t)$;
- visible tip $x+h$, where $h$ is the overhang;
- base, the burnout end: $x(t-\tau_b)$.

#### 1.2 Features and the law network

With $U' = \max(U, u_{\min})$, $u_{\min}=0.1$ cm/s:

$$u = \frac{U'}{u_{\mathrm{ref}}} - 1,\qquad c_{O_2} = \frac{X_{O_2} - o_{\mathrm{ref}}}{o_{\mathrm{scale}}},\qquad c_w = \ln r,\quad r = W/W_{\mathrm{ref}}$$

(`front.rs:382-386`), with $u_{\mathrm{ref}}=5$ cm/s, $o_{\mathrm{ref}}=0.19$, $o_{\mathrm{scale}}=0.02$ (`front.rs:197-200`).

Each of the four laws (speed, over, burn, quench) is a one-hidden-layer hinge network of width 6 on $\mathbf c=(u,c_{O_2},c_w)$ (`nn.rs:69-84`, `front.rs:69-73`):

$$N(\mathbf c) = b_0 + \mathbf{s}\cdot\mathbf c + \sum_{j=1}^{6} w_{2,j}\,\mathrm{relu}\!\left(b_{1,j} + W_{1,j}\cdot \mathbf c\right)$$

The laws are, with $o = X_{O_2}/o_{\mathrm{ref}}$ (`front.rs:387-395`):

$$v = \max(N_{\mathrm{speed}}, v_{\min})\; o^{p_{v,0}}\, r^{p_{v,1}},\qquad h_{ss} = \max(N_{\mathrm{over}},0)\; o^{p_{h,0}}\, r^{p_{h,1}}$$

$$\tau_b = e^{N_{\mathrm{burn}}},\qquad \lambda_q = e^{N_{\mathrm{quench}}}$$

with $v_{\min}=0.001\ w/\mathrm{s}$ (`front.rs:202`).

#### 1.3 D1 laws in closed form

D1 types no hidden unit for speed, over or burn, and no weight on $c_w$ anywhere (`d1-fable/edits.wsp:36-96`; `release.json` per-tensor nonzero counts: `speed.W1` 0, `over.W1` 0, `burn.W1` 0, `quench.W1` 3, all in columns 0 and 1). Substituting the typed values:

$$v = (0.0535 + 0.032\,u)\, o^{1.5}\, r^{-1} = (0.0215 + 0.0064\,U')\, o^{1.5}\, r^{-1}\quad [w/\mathrm{s}]$$

The floor $v_{\min}$ never binds for D1: the smallest network value is $0.0535+0.032\cdot(-0.98)=0.0221$.

$$h_{ss} = (0.325 + 0.325\,u)\, o^{3}\, r^{-1} = 0.065\,U'\, o^{3}\, r^{-1}\quad [w]$$

$$\tau_b = e^{1.84} = 6.30\ \mathrm{s}\quad\text{(constant in flow, oxygen and width)}$$

The quench hazard has two hidden units (`d1-fable/edits.wsp:84-96`). Since $5u = U'-5$, unit 0 reads $\mathrm{relu}(U_q - U')$ with $U_q = 2.27 - 1.33\,c_{O_2}$:

$$\ln\lambda_q = -7 + 2.5\,\mathrm{relu}\!\left(2.27 - 1.33\,c_{O_2} - U'\right) + 12\,\mathrm{relu}\!\left(-0.9 - c_{O_2}\right)\quad[\ln(1/\mathrm{s})]$$

The three terms:

- Far from the boundary, $\lambda_q=e^{-7}=9.1\times10^{-4}\,\mathrm{s^{-1}}$.
- Each cm/s below the quench velocity $U_q$ adds 2.5 to $\ln\lambda_q$.
- Below $X_{O_2}=0.172$, each 0.02 of oxygen adds 12.

This is a piecewise-exponential hazard. There is no Weibull and no separate ignition model in D1. The "ignition outcome" score uses this same hazard (section 1.8).

#### 1.4 The law path (prefix-free, from igniter-on)

The path is integrated on the grid $t_i = t_0 + i\,\Delta t$, $\Delta t = 0.25$ s (`front.rs:206`). There are $n=\lceil (t_{\max}-t_0)/\Delta t\rceil\vee 1 + 1$ steps, with $x_0=h_0=\Lambda_0=0$ (`front.rs:580-588`). The laws are evaluated at the midpoint $t_m=t_i+\Delta t/2$, at the flow in force there. The flow is a right-continuous step function, $U(t)$ = the last entry with start $\le t$ (`front.rs:500-503`).

$$g(t_m)=\begin{cases}0 & t_m < t_0 + \text{ign.delay}\\ \text{ign.gain} & t_m < 0\\ 1 & \text{otherwise}\end{cases}\qquad(\texttt{front.rs:592-598})$$

$$x_{i+1} = x_i + g\,v\,\Delta t,\qquad h_{i+1} = \begin{cases} h_{ss} + (h_i - h_{ss})\,e^{-\Delta t/\tau_h} & g>0\\ h_i & g=0\end{cases}$$

$$\Lambda_{i+1} = \Lambda_i + \lambda_q\,\Delta t\;\mathbf 1[t_m\ge 0]\qquad(\texttt{front.rs:599-605})$$

$X(t)$, $H(t)$ and $\Lambda(t)$ are read by linear interpolation. Times outside the grid are clamped to its ends, so $X(t)=0$ for $t\le t_0$ (`front.rs:484-488`).

#### 1.5 What the prefix adds: the run offset $E$

The prefix is the tip and base stills with $0\le t\le \tau$, where $\tau$ is the cutoff (`front.rs:612-625`). On the track route these are stills whose `state` is `flame` and whose `quality` is not `edge` (`front_tools.rs:371-387`).

**With at least one tip still** (`front.rs:626-710`):

1. **Windows.** Let $t_f$ and $t_l$ be the first and last still. The windows are $[t_f, t_f+4]$ and $[t_l-4, t_l]$ (`enc.end_s` = 4 s), each reduced to its mean time and mean value (`front.rs:506-517, 628-632`). The late window gives $(t_e, p_e)$ and the early one $(t_a, p_a)$. The span is $\Delta = t_e - t_a$.

2. **Tip offset.** It is read only if $\Delta\ge 3$ s (`enc.min_span_s`) and $X(t_e)-X(t_a) > 10^{-9}$ (`front.rs:636-645`):

$$e_{\mathrm{tip}} = \begin{cases}\ln\dfrac{(p_e - H(t_e)) - (p_a - H(t_a))}{X(t_e)-X(t_a)} & \text{numerator} > 0\\[4pt] -\text{e\_clip}_0 & \text{otherwise}\end{cases}$$

3. **Base offset.** It needs both base windows, $b_e$ at $t_{be}$ and $b_a$ at $t_{ba}$, with $t_{be}-t_{ba}\ge 3$ s and a positive law advance. $\tau_b$ is evaluated at the flow of the *later base window*, not at the cutoff (`front.rs:646-663`):

$$e_{\mathrm{base}} = \ln\frac{b_e - b_a}{X(t_{be}-\tau_b) - X(t_{ba}-\tau_b)}\quad(\text{or } -\text{e\_clip}_0 \text{ if } b_e\le b_a)$$

   The base condition is checked only inside the tip condition $\Delta\ge 3$ s.

4. **Mix.** The offsets are combined over those available (`front.rs:665-671, 693`):

$$e_{\mathrm{obs}} = \frac{\sum_k m_k e_k}{\sum_k m_k},\qquad m=(0.8,\,0.2)$$

   If none is available, $e_{\mathrm{obs}}=0$.

5. **Settled share.** Take $U_e$, the flow at $t_e$. Walking back from $t_e$, find the most recent flow entry with $|U-U_e|\ge 0.8$ cm/s (`enc.change_cm_s`). The flow last changed at $t_{\mathrm{ch}}$, the start of the entry after it, never earlier than $t_0$. If no entry differs, $t_{\mathrm{ch}}=t_0$ (`front.rs:677-684`). Then

$$\text{settled} = \mathrm{clamp}\!\left(\frac{t_e - t_{\mathrm{ch}}}{10\,\mathrm{s}},0,1\right)\qquad(\texttt{front.rs:685})$$

6. **Gain, offset and spread** (`front.rs:686-694, 705`):

$$G = \sigma\!\left(b_G + w_{G,0}\ln\frac{\max(\Delta,10^{-6})}{10} + w_{G,1}\,\text{settled}\right)\quad(G=0 \text{ if no offset is available})$$

$$\ln E = \mathrm{clamp}\!\left(G\,e_{\mathrm{obs}},\;-\text{e\_clip}_0,\;\text{e\_clip}_1\right)$$

$$s = \sqrt{G\,s_{\mathrm{floor}}^2 + (1-G)\,s_{\mathrm{pop}}^2 + (s_{\mathrm{far},0}\,c_{O_2})^2 + (s_{\mathrm{far},1}\,c_w)^2}$$

7. **Anchor.** $x_e = p_e - H(t_e)$, $h_e = H(t_e)$, $\sigma_x = \sigma_w$ (`front.rs:706-708`).

For D1, $G=\sigma(-2.3 + 2.5\ln(\Delta/10) + 3.5\,\text{settled})$, and $E\in[e^{-0.7}, e^{0.7}]=[0.497, 2.014]$. For example, $\Delta=35$ s settled gives $G=0.987$.

**Without a tip still** (`front.rs:711-727`): $t_e=\tau$, $G=0$, $\ln E=0$, $s=\sqrt{s_{\mathrm{pop}}^2 + \text{far}^2}$, $x_e = X(\tau)$, $h_e = H(\tau)$ and $\sigma_x = \sigma_{x0}$. The front then starts at 0 at igniter-on.

#### 1.6 The forecast

For the uncut tip, at a forecast time $t\ge\tau$ (`front.rs:731-740`):

$$A(t) = E\,\big(X(t) - X(t_e)\big),\qquad \mu(t) = x_e + A(t) + H(t)$$

$$\mathrm{sd}(t) = \sqrt{\sigma_x^2 + \big(s\,A(t)\big)^2 + \big(\text{rel\_over}\,(H(t)-h_e)\big)^2 + \text{drift}^2\,\max(t-t_e,0)}$$

Lead is measured from $t_e$, the mean time of the last window, not from the cutoff. The run offset scales the law's *advance* only. The overhang $H(t)$ is carried unscaled.

**Quantiles.** The levels default to $p\in\{0.05,0.25,0.5,0.75,0.95\}$ (`common.rs:14, 65-71`). Each quantile is Gaussian and cut at the sample end $L_w$ when its length is known (`front.rs:745-751`):

$$q_p(t) = \min\!\big(\mu(t) + \Phi^{-1}(p)\,\mathrm{sd}(t),\;L_w\big),\qquad \text{mean}(t)=\min(\mu(t),L_w)$$

**Base (median only)** (`front.rs:752-755`). $\tau_b$ is taken at the flow at $t$. The base is not cut at $L_w$, and no overhang is added:

$$\text{base}(t) = \max\!\Big(0,\; x_e + E\,\big(X(t-\tau_b) - X(t_e)\big)\Big)$$

**Quenched by $t$** (`front.rs:756`). The hazard is counted only after the cutoff, and the flame is assumed to exist at $\tau$:

$$P_q(t) = 1 - \exp\!\big(-\max(\Lambda(t)-\Lambda(\tau),0)\big)$$

**Reached the end** (`front.rs:757-775`). Take $t^\ast = \max(t-\text{linger}, t_e)$:

$$P_{\mathrm{end}}(t) = \Phi\!\left(\frac{\mu(t^\ast) - L_w}{\mathrm{sd}(t^\ast)}\right)\quad(\text{a step if sd}=0;\ 0 \text{ if } L_w \text{ unknown})$$

   A running maximum over the forecast times, in time order, then makes $P_{\mathrm{end}}$ non-decreasing.

**Event rows** in the metric artifact (`front.rs:974-980`). The fourth row, `unresolved_support`, is left at 0:

$$\text{active}=(1-P_q)(1-P_{\mathrm{end}}),\quad \text{extinction}=P_q,\quad \text{reached\_end}=(1-P_q)P_{\mathrm{end}}$$

**What is emitted.** In sample widths on the track route (`Forecast`, `front.rs:459-471`): tip quantiles, mean, base median, `p_quenched`, `p_reached_end`, and the encoder diagnostics ($E$, $G$, offsets, laws at the cutoff). The metric route multiplies positions by $W$ (`front.rs:970-1001`).

#### 1.7 Inputs, clamps and refusals

**Refusals** in `run` (`front.rs:522-577`), each a scientific refusal:

- $X_{O_2}\notin(0,1)$;
- $r\le 0$ or $r$ not finite;
- an empty or unsorted flow history, or one with negative or non-finite entries;
- $t_0>0$;
- $\tau<0$;
- a forecast time outside $[\tau, \tau+600]$ s (`num.max_lead_s`).

**Clamps and branches:**

- $U'=\max(U,0.1)$;
- $v\ge v_{\min}$;
- $h_{ss}\ge 0$;
- $\ln E$ clipped to $[-\text{e\_clip}_0, \text{e\_clip}_1]$;
- a non-positive observed advance is set to $-\text{e\_clip}_0$;
- the law advance must be $>10^{-9}$;
- the span must be $\ge$ 3 s;
- the settled share lies in $[0,1]$;
- quantiles are cut at $L_w$;
- the base is floored at 0;
- $P_{\mathrm{end}}$ is a running maximum.

**Metric route** (`front.rs:846-968`):

- $r = W/0.022$ m (`front.rs:952`);
- $L_w = \text{length}/W$ when a positive length is given (`front.rs:953`);
- flow in cm/s = `value_si`·100 (`front.rs:914`);
- times are shifted by the igniter-off time;
- $t_0 = \min(t_{\mathrm{on}} - t_{\mathrm{off}}, 0)$ (`front.rs:950`);
- default forecast times: 41 equally spaced points over 60 s (`common.rs:75-96`, `front.rs:937-942`).

**Track route** (`front_tools.rs:286-423`), in order of precedence:

- **Width ratio:** `geometry.sample_width_m`/0.022; else a band width under 44 px gives 1.2/2.2; else the label ("1 cm" gives 1.2/2.2, "2 cm" gives 1); else a refusal (`front_tools.rs:317-333`).
- **Window:** `geometry.window_length_w`; else $(10/2.2)/r$ (`front_tools.rs:341-346`).
- **Igniter-on:** `igniter.on_t_s`; else $-6$ s (`front_tools.rs:405-410`).
- **Flow:** the calibrated `u_cm_s` of a fan entry; else linear interpolation through the day's calibration rows, with a zero-flow row at fan 38 when there is none (`front_tools.rs:251-284`).
- **Other refusals:** a sample that is not SIBAL, and a non-concurrent `conditions.flow` (`front_tools.rs:300-340`).

#### 1.8 Parameter table (spec `bass_front-wsp-v1`)

Spec bounds and descriptions: `front.rs:92-207`. D1 values: `d1-fable/edits.wsp:36-130`. Persistence values: `baseline-persistence/edits.wsp:13-48`. Every hidden-layer element not listed is 0 in both releases (sparse tensors).

| Symbol | `edits.wsp` name | Meaning | D1 | Persistence | Units |
|---|---|---|---|---|---|
| $b_{0,v}$ | `speed.b0` | spread rate at $u=c=0$ | 0.0535 | 0.05 | w/s (ref. widths) |
| $\mathbf s_v$ | `speed.skip[0:3]` | linear part on $(u,c_{O_2},c_w)$ | [0.032, 0, 0] | [0, 0, 0] | w/s |
| $p_{v,0},p_{v,1}$ | `speed.pow[0:2]` | powers of $o$ and $r$ in $v$ | [1.5, −1.0] | [0, 0] | 1 |
| $b_{0,h}$ | `over.b0` | steady overhang at $u=c=0$ | 0.325 | 0 | w |
| $\mathbf s_h$ | `over.skip[0:3]` | linear part | [0.325, 0, 0] | [0, 0, 0] | w |
| $p_{h,0},p_{h,1}$ | `over.pow[0:2]` | powers of $o$ and $r$ in $h_{ss}$ | [3.0, −1.0] | [0, 0] | 1 |
| $\tau_h$ | `over.tau_s` | overhang relaxation time | 2.0 | 2.0 | s |
| $b_{0,b}$ | `burn.b0` | $\ln\tau_b$ | 1.84 ($\tau_b$=6.30 s) | 1.79 ($\tau_b$=5.99 s) | ln s |
| $\mathbf s_b$ | `burn.skip[0:3]` | linear part | [0, 0, 0] | [0, 0, 0] | ln s |
| $b_{0,q}$ | `quench.b0` | $\ln\lambda_q$ at the reference | −7.0 | −40.0 | ln(1/s) |
| $\mathbf s_q$ | `quench.skip[0:3]` | linear part | [0, 0, 0] | [0, 0, 0] | ln(1/s) |
| $W_{1,0}$, $b_{1,0}$, $w_{2,0}$ | `quench.W1[0,0]`, `quench.W1[0,1]`, `quench.b1[0]`, `quench.w2[0]` | below-quench-velocity unit | −5.0, −1.33, −2.73, 2.5 | 0 | — |
| $W_{1,1}$, $b_{1,1}$, $w_{2,1}$ | `quench.W1[1,1]`, `quench.b1[1]`, `quench.w2[1]` | low-oxygen unit | −1.0, −0.9, 12.0 | 0 | — |
| — | `ign.gain` | spread multiple while the igniter is on | 1.0 | 1.0 | 1 |
| — | `ign.delay_s` | delay before the front moves | 0.0 | 0.0 | s |
| — | `end.linger_s` | flame still seen after the tip reaches the end | 5.0 | 5.0 | s |
| $w_{G}$ | `enc.gain_w[0:2]` | gain-logit weights on $\ln(\Delta/10)$ and settled | [2.5, 3.5] | [0, 0] | 1 |
| $b_G$ | `enc.gain_b` | gain-logit bias | −2.3 | 10.0 | 1 |
| $m$ | `enc.mix[0:2]` | tip / base weights | [0.8, 0.2] | [1.0, 0] | 1 |
| e_clip | `enc.e_clip[0:2]` | $\ln E$ bounds (below, above) | [0.7, 0.7] | [3.0, 3.0] | 1 |
| $s_{\mathrm{pop}}$ | `unc.s_pop` | rel. sd of the speed before the prefix | 0.25 | 0.5 | 1 |
| $s_{\mathrm{floor}}$ | `unc.s_floor` | rel. sd of the speed when fully trusted | 0.06 | 0.15 | 1 |
| $s_{\mathrm{far}}$ | `unc.s_far[0:2]` | added rel. sd per $\lvert c_{O_2}\rvert$, per $\lvert c_w\rvert$ | [0.06, 0.25] | [0, 0] | 1 |
| $\sigma_w$ | `unc.sigma_w` | tip sd at the cutoff | 0.04 | 0.04 | w |
| $\sigma_{x0}$ | `unc.sigma_x0_w` | tip sd without stills | 0.3 | 1.0 | w |
| rel_over | `unc.rel_over` | sd share of the overhang change | 0.5 | 0 | 1 |
| drift | `unc.drift_w` | drift sd | 0.01 | 0.01 | w s$^{-1/2}$ |
| $u_{\mathrm{ref}}$ | `norm.u_ref_cm_s` (constant) | flow reference | 5.0 | 5.0 | cm/s |
| $u_{\min}$ | `norm.u_min_cm_s` (constant) | lowest flow evaluated | 0.1 | 0.1 | cm/s |
| $o_{\mathrm{ref}}$, $o_{\mathrm{scale}}$ | `norm.o2_ref`, `norm.o2_scale` (constants) | oxygen reference and scale | 0.19, 0.02 | same | 1 |
| $W_{\mathrm{ref}}$ | `norm.w_ref_m` (constant) | reference sample width | 0.022 | same | m |
| $v_{\min}$ | `norm.v_min` (constant) | spread-rate floor | 0.001 | same | w/s |
| — | `enc.end_s`, `enc.min_span_s`, `enc.change_cm_s` (constants) | window length, minimum span, change threshold | 4.0, 3.0, 0.8 | same | s, s, cm/s |
| $\Delta t$, — | `num.dt_s`, `num.max_lead_s` (constants) | step, longest lead | 0.25, 600 | same | s |

D1 holds 35 nonzero authored elements of 159; persistence holds 15 (`release.json` → `architecture.parameters`).

#### 1.9 How the forecasts were scored (front-feedback)

Scorer: *research repo* `crates/ignis-wsp-dev/src/front.rs`. It is not in `ignis-web`.

**Cases** (`dev/front.rs:82-90, 417-446`):

- cutoffs at 5, 10, 20 and 40 s;
- at least 6 flame stills per track, at least 2 at or before the cutoff, and at least 3 targets in $(\tau, \tau+60]$;
- the targets are forecast at their own times.

**Per-target scores** (`dev/front.rs:637-649`):

- CRPS from the five quantiles, $\mathrm{CRPS}=2\sum_k \tfrac{\tau_{k+1}-\tau_{k-1}}{2}\,\rho_{\tau_k}(y-q_k)$ with $\tau_0=0$, $\tau_6=1$ and the pinball $\rho_\tau(y-q)=(\tau-\mathbf 1[y<q])(y-q)$ (*research repo* `crates/ignis-lab/src/eval/scores.rs:55-57, 74-88`);
- absolute error of $q_{0.5}$;
- coverage of $[q_{0.25},q_{0.75}]$ and $[q_{0.05},q_{0.95}]$, ends inclusive.

**Track ended** (`dev/front.rs:663-669`). At $h\in\{10,20,30,45,60\}$ s the forecast is $1-(1-P_q)(1-P_{\mathrm{end}})$, scored by Brier against the observation:

- `extinction` or `left_view` with $t_{\mathrm{end}}\le\tau+h$ counts as 1;
- `lost` or `record_end` before $\tau+h$ is censored (`dev/front.rs:159-167`).

**Ignition outcome** (`dev/front.rs:679-698`). The forecast is $P_q(20\,\mathrm s)$ from cutoff 0 with no stills, against `end.kind == no_spread`. For D1 this is $1-\exp(-\int_0^{20}\lambda_q)$. Worked example from the formula above: at 16.4 % O₂ and 6 cm/s, $\ln\lambda_q=-7+12\cdot0.4=-2.2$, which gives $P_q(20)=0.89$.

---

### 2. Persistence baseline, exactly

Same forward, `front.rs`. Substituting `baseline-persistence/edits.wsp` (spec constants as in D1):

- **Laws:** $v=0.05$ w/s at every flow, oxygen and width (all powers 0; the floor is irrelevant). $h_{ss}=0$, so $H\equiv0$ and the tip is the front. $\tau_b=e^{1.79}=5.99$ s. $\lambda_q=e^{-40}=4.2\times10^{-18}\,\mathrm{s^{-1}}$, so $P_q\approx0$.
- **Path:** $X(t)=0.05\,(t-t_0)$ (`ign.gain` = 1, delay 0).
- **Gain:** $G=\sigma(10)=0.99995$ whenever the tip offset exists. The mix $[1,0]$ means the base is never used.
- **Run offset:** $\ln E=\mathrm{clamp}(G\ln\frac{p_e-p_a}{0.05\,\Delta},-3,3)$, or $-3$ if $p_e\le p_a$. For the usual case where the stills are after igniter-off, so $X(t_e)-X(t_a)=0.05\Delta$:

$$\mu(t) \approx p_e + \frac{p_e - p_a}{\Delta}\,(t - t_e)\qquad\text{(exponent } G=0.99995 \text{ on the ratio, not exactly 1)}$$

$$\mathrm{sd}(t)=\sqrt{0.04^2 + (s\,A)^2 + 0.01^2\,(t-t_e)},\qquad s=\sqrt{G\cdot0.15^2+(1-G)\cdot0.5^2}\approx0.15$$

- **Without stills:** $\mu=0.05(\tau-t_0)+0.05(t-\tau)$, $s=0.5$, $\sigma_x=1.0$.
- **Unchanged:** quantile cut at $L_w$, `linger` = 5 s and the $P_{\mathrm{end}}$ rule.
- Persistence has no flow, oxygen or width term at all. It "keeps the speed the prefix shows, whatever the flow does" (`baseline-persistence/edits.wsp:2-4`).

---

### 3. Other forwards (differences from D1 only)

#### 3.1 `bass_front.wsp5` Q0 (`front5.rs`)

**Tip.** The tip law, prefix reading and quantiles are `front.rs` line for line (`front5.rs:1-29`, with a bit-identity test claimed in `tests/front5_forward.rs`; I did not run it). Q0 carries D1's speed, over, burn, enc and unc values (`q0-fable/edits.wsp`; the values not written equal the spec baseline, which equals D1 for `over.tau_s`, `ign.*`, `end.linger_s`, `unc.sigma_w` and `unc.drift_w`). There is no `quench.*` network.

**Boundary law.** The quench hazard is replaced by a boundary law. Indices: $m$ is the material (0 SIBAL, 1 PMMA 1 mm) and $d$ the direction (0 concurrent, 1 opposed); $p$ is in atm. The width is in cm: $W=r\cdot0.022\cdot100$ (`front5.rs:614-616`). The limit is computed in `front5.rs:668-697`:

$$U_e=\begin{cases}\sqrt{U^2+u_s[m]^2}&\text{concurrent}\\ U+u_s[m]&\text{opposed}\end{cases},\qquad A = a_r[m,d] + \frac{a_w[m]}{W}$$

$$X_{\lim}(U) = x_0[m] + c_p[m]\ln\frac1p + s_b[m,d]\,U_e + \frac{A}{p\,U_e}$$

$$s^2 = \sigma_{x0}^2 + (\sigma_{cp}\ln\tfrac1p)^2 + (\sigma_{sb}U_e)^2 + \frac{\sigma_{ar}^2 + (\sigma_{aw}/W)^2}{(pU_e)^2},\qquad P_{\mathrm{sus}}=\Phi\!\left(\frac{X-X_{\lim}}{s}\right)$$

The limit is refused if $U_e\le 0$.

Over 41 equal-probability normal nodes $e_k$ with $w_k=1/41$, one node per burn and the same node at every time (`front5.rs:1040-1042, 713-723`):

$$\text{lit}_k = w_k\,\Phi\!\left(\frac{X - X_{\lim}(U_{\mathrm{ign}}) - d_x[m] - s(U_{\mathrm{ign}})e_k}{\sigma_a}\right)\Phi\!\left(\frac{\ln(t_{\mathrm{on}}/t_{\mathrm{req}}[m])}{\sigma_{\ln t}}\right)$$

$$\text{rate}_k = \left(\frac{w_r}{\tau[m,0]}+\frac{1-w_r}{\tau[m,1]}\right)\Phi\!\left(-\frac{X-X_{\lim}(U(t))-s(U(t))e_k}{\text{ext.width}}\right)$$

$$w_r=\frac{r_A}{r_A+b},\quad r_A=\max\!\left(\frac{A}{pU_e},0\right),\quad b=s_b U_e\quad(w_r=\tfrac12 \text{ if } r_A+b=0)$$

$\Lambda_k$ is integrated exactly on the flow steps from igniter-off (`front5.rs:949-998`).

**Event probabilities** (`front5.rs:1044-1080`):

- With no flame in the prefix: $P_{\mathrm{alive}}=\sum_k\text{lit}_k e^{-\Lambda_k}$.
- With a flame seen at $t_c$, the time of the last prefix tip still (`front5.rs:1176-1182`): the nodes are reweighted, $\pi_k\propto w_k\Phi(\cdot)e^{-\Lambda_k(t_c)}$, and $P_{\mathrm{alive}}(t)=\sum_k\pi_k e^{-\max(\Lambda_k(t)-\Lambda_k(t_c),0)}/\sum\pi_k$.
- In both cases $P_{\mathrm{alive}}$ is clipped at 1, and the forward reports $P_q=1-P_{\mathrm{alive}}$ (`front5.rs:1380`). With no flame in the prefix this includes "never lit".
- For an opposed flow or for PMMA, only the events are returned and the tip is refused (`front5.rs:1183-1203`).

**Q0 boundary values** (`q0-fable/edits.wsp`):

| Name | Value |
|---|---|
| `sus.x0` | [0.148, 0.136] |
| `sus.cp` | [0.020, 0.020] |
| `sus.sb` | [[0.0008, 0.0016], [0.0008, 0.0015]] |
| `sus.ar` | [[0.075, 0.040], [0.074, 0.039]] |
| `sus.aw` | [0.030, 0.030] |
| `sus.us` | [0.15, 0.020] |
| `sus.sd_x0` | [0.010, 0.015] |
| `sus.sd_cp` | [0.020, 0.020] |
| `sus.sd_sb` | [[0.0003, 0.0007], [0.0004, 0.0007]] |
| `sus.sd_ar` | [[0.026, 0.014], [0.030, 0.016]] |
| `sus.sd_aw` | [0.030, 0.030] |
| `ign.dx` | [0.004, 0.006] |
| `ign.sd_a` | 0.005 |
| `ign.t_req` | [2.0, 10.0] s |
| `ign.sd_lnt` | 0.50 |
| `ext.tau` | [[12.0, 1.5], [100.0, 3.0]] s (branch 0 radiative, 1 blow-off) |
| `ext.width` | 0.0005 |

`front5/carry-d1` is D1 with the boundary switched off by neutral values (`carry-d1/header.wsp:1-19`). Q0 was not in the Saffire test. The README says a zero-shot ignite/stay-lit law was "tested and not adopted" (*research repo* `docs/wsp/README.md:50`); I did not confirm that this sentence refers to Q0.

#### 3.2 `bass_front.wsp3` joint (`front3.rs`) and `bass_front.wsp4` (`front4.rs`)

**`front3.rs`.** This forward is `front2.rs`, whose additions are all switched off in these releases (`speed.split`=0, `speed.kappa`=0, `over.tau_up_s`=0, flow paths `unc.flow_w`=[0,1,0]; `front3/r1-astra/patch.json`), plus an oxygen-supply speed (`front3.rs:110-126`):

$$Y=\frac{X M_{O_2}}{X M_{O_2}+(1-X)M_c},\quad \mathcal A=\frac{n_f\rho Y}{m''\,\nu_s},\quad v_{\mathrm{SI}} = \tfrac12\mathcal A^2\nu\,t_r\,U + \sqrt{\left(\tfrac12\mathcal A^2\nu\,t_r\,U\right)^2 + (\mathcal A\alpha)^2}$$

Here $U$ is in m/s and $\nu_s$ is the stoichiometric O₂/fuel mass ratio. The speed is divided by $0.022\,r$ to give w/s (`front3.rs:594-606`).

The joint values (r1 → joint) are:

| Name | Value |
|---|---|
| `oxygen.rho_kg_m3` | 1.15 |
| `oxygen.nu_m2_s` | 1.78e-4 |
| `oxygen.alpha_m2_s` | 2.41e-4 |
| $M_{O_2}$ | 0.0319988 |
| $M_c$ | 0.0280134 |
| $m''$ | 0.1354 kg/m² |
| `oxygen.faces` | 2 |
| $\nu_s$ | 1.185 |
| `oxygen.residence_s` | 10 s |
| `oxygen.mix` | 1 |
| `oxygen.member_weight` | 0.5 (the only op of the joint patch) |

`member_weight` 0.5 mixes the full position CDFs of the D1-speed scenario and the oxygen-speed scenario, not their quantiles. With both switches positive, events use a "coherent first-event" closure (endpoint exceedance converted to competing hazards with the inherited D1 quench law; `front3.rs:16-22, 1268-1293`). The other numbers are D1's, transcribed.

**`front4.rs`.** Same weights as the joint (`member_weight` 0.5). The tip and delayed-base offsets, and the anchor, average the modelled path at the same timestamps as the observations, instead of at the window mean time (`front4.rs:1-4`). I did not derive the front3 or front4 event closures in formula form.

---

### 4. The Saffire transfer

**Inputs changed** (*research repo* `docs/wsp/SAFFIRE_TRANSFER.md` §4):

| Input | Value |
|---|---|
| width ratio $r$ | $0.406/0.022 = 18.4545$ (written 18.5), via `front_tools.rs:317-318` |
| window $L_w$ | $94/40.6=2.315$ w |
| $X_{O_2}$ | 0.21, nominal cabin air (Saffire I S1, III S1) |
| flow | one commanded entry, 20 cm/s (Saffire I) or 30 cm/s (Saffire III), held to the end |
| `igniter.on_t_s` | minus the tabulated igniter time (Saffire I: 8 s); for Saffire III, the first glow frame |

Opposed sheets are refused at `front_tools.rs:334-340`. **Zero weights changed** (`freeze.json`: `"weights_changed": false`; the release ids are the four above). I did not open the held tracks. That they carry `window_length_w` = 2.315 is the protocol's statement. If it were absent, the reader would fall back to $(10/2.2)/18.45=0.246$ w.

**How $r$ enters D1.** $c_w=\ln 18.4545=2.915$, but every D1 weight on $c_w$ is zero. So $r$ acts only through the two powers $r^{-1}$ and through $s_{\mathrm{far},1}$:

- $v$ and $h_{ss}$ in sheet widths are the 2 cm-class values divided by 18.45. In metres they are unchanged, since a width power of −1 means the same mm/s.
- $\tau_b$ (6.30 s) and $\lambda_q$ do not depend on width.

With $o=0.21/0.19$, $o^{1.5}=1.162$ and $o^3=1.350$:

| $U$ | $u$ | $v$ (ref. w/s) | $v$ (sheet w/s) | $v$ (cm/s) | $h_{ss}$ (sheet w) | $h_{ss}$ (cm) | $L_w/v$ |
|---|---|---|---|---|---|---|---|
| 20 cm/s | 3 | 0.1737 | 0.00941 | 0.382 | 0.0951 | 3.86 | ≈ 246 s |
| 30 cm/s | 5 | 0.2481 | 0.01344 | 0.546 | 0.1427 | 5.79 | ≈ 172 s |

The last column is the time to cross 2.315 w at the law speed, overhang ignored.

**Quench.** At 21 % O₂ ($c_{O_2}=1$), $U_q=0.94$ cm/s, far below 20 cm/s, so both units are off. $\lambda_q=e^{-7}$ and $P_q(60\,\mathrm s)=1-e^{-60e^{-7}}=0.053$ after the cutoff.

**Spread.** $\text{far}^2=(0.06\cdot1)^2+(0.25\cdot2.915)^2=0.535$, so $s=0.734$ at $G=1$ and $0.773$ at $G=0$. A 2 cm strip in the same oxygen has $s=0.085$. The run-offset term of the tip sd is $\approx0.73\,A(t)$ at the sheet whatever the prefix says, and $E$ itself stays bounded in $[0.497, 2.014]$. The forward accepts any positive $r$ (`front.rs:528-533`). At 18.5 every width term is an extrapolation from two classes (0.55 and 1).

**Result.** From *research repo* `docs/wsp/feedback/saffire-transfer/results/README.md`, read as aggregates only: on the two wide concurrent sheets, 8 cases, D1 tip CRPS was 0.058 w against persistence 0.250. D1 was lower on 2 of 2 burns. Its 50 %/90 % coverage was 0.93/1.00, so the bands were wider than needed. This fits the inflated $s$ above, but the report does not attribute the coverage to that term. Persistence uses no width at all.

---

### 5. `bass.wsp` W0, phase 1 (`bass.rs`)

This is a metric contour forecast on a transverse coordinate $\xi\in[0,1]$ with $K=4$ cosine modes, $\phi_0=1$ and $\phi_m=\sqrt2\cos(m\pi\xi)$ (`bass.rs:152-159`):

- $a_m=\int f\phi_m\,d\xi$ by the trapezoid rule (`bass.rs:161-169`).
- $v_m$ is the finite difference of the window means over the last two windows, each spanning 0.34 of the prefix (`bass.rs:339-362`).
- Propagation (`bass.rs:236-241`): $a_m(t_n+D)=a_m e^{-\delta_m D} + g_m v_m T_m(1-e^{-D/T_m})$.
- The unresolved residual of the last contour is carried unchanged.
- The mean front is $\int f\,d\xi$, with sd $\sqrt{\sigma^2 + (\text{rel}_v\,|g_0 v_0| + \sigma_v)^2 D^2}$ and Gaussian quantiles (`bass.rs:385-388, 403-415`).

The forecast is refused for pixel-only (uncalibrated) contours, for fewer than 4 usable contours, or for a lead over 60 s.

| Name | W0 value |
|---|---|
| `op.gain` | [1.0, 0.5, 0.25, 0.0] |
| `op.persist_s` | [1000, 5, 3, 1] s |
| `op.damp_per_s` | [0, 0.099, 0.395, 0.888] 1/s |
| `unc.sigma_m` | 3e-4 m |
| `unc.rel_v` | 0.2 |
| `unc.sigma_v_m_s` | 1e-4 m/s |

W0 has no flow, oxygen or width law and no event output. Its own header says it "never" was scored and "cannot be yet", because no metric BASS contour exists (`w0-fable/edits.wsp:3-4`).

---

### 6. Not determined here

- The `front3` and `front4` coherent first-event closures and the window-matched operator are described in words, not in formulas.
- The bit-identity tests (`tests/front5_forward.rs`, `tests/front2_forward.rs`) are cited, not run.
- The Saffire tracks' own `window_length_w`, `igniter.on_t_s` and flow entries were not read (held files).
- Whether the README's "zero-shot law … not adopted" sentence refers to Q0 was not confirmed.
- No score in this note was recomputed.


## Part 4. The generic law forward (all other investigations)

Sources. Paths without a prefix are relative to the root of `ignis-web`. `research:` marks a file that exists only in `SpaceApps-Ardealu` (branch `feat/handwritten-science`, commit `542dad8`). `law.rs`, `law_tools.rs`, `law_slugs.rs`, `common.rs`, `quad.rs`, `spec.rs` and `nn.rs` under `crates/ignis-wsp/src/` are identical in both repositories, and so is `configs/models/law/` (a recursive diff shows no difference). The typed numbers below are the full tensors of each release. They were read with `ignis wsp inspect --release <id> --edits` (`research:target-merged/release/ignis.exe`), which resolves the whole lineage (root → W0 → …). The comment lines quoted from each `edits.wsp` are the author's own reading of those numbers. Every release id was checked against `research:tools/wsp/final/releases_final_gen7.json`, and all of them match.

Notation: $\operatorname{relu}(a)=\max(0,a)$, $\sigma(a)=1/(1+e^{-a})$, $\operatorname{sp}(a)=\ln(1+e^{a})$, and $\Phi$ is the standard normal CDF. An indicator feature is written as a plain letter.

---

### A. The kernel (`crates/ignis-wsp/src/law.rs`)

#### A.1 What a declaration declares

A slug is one line in `crates/ignis-wsp/src/law_slugs.rs:35-` plus the file `configs/models/law/<slug>/decl.json`, which is compiled in (`law_slugs.rs:19-33`). A declaration holds names only and no numbers (`law.rs:151-164`):

- `features` (at most 16, `law.rs:43`): a name, an SI unit, and a `transform` from {`identity`, `ln`, `reciprocal`} (`law.rs:70-86`, `96-104`).
- `targets` (1 to 8, `law.rs:44`), each of `kind` (`law.rs:88-94`, `113-131`):
  - `positive`: needs an SI unit. It may name a `shared_quantity`, which must not be `phase_end_time` (`law.rs:208-228`).
  - `binary`: has no unit (`law.rs:229-233`).
  - `event`: 1 to 3 named competing `causes` (`law.rs:45`, `241-254`) and an optional `scheduled_stop.condition` (`law.rs:255-259`). Times are in seconds since the time origin.
- `status` ∈ {`declared`, `placeholder`, `synthetic`} (`law.rs:59-68`). No release of a placeholder can be assembled (`law.rs:64`).

The checks of names, units, uniqueness and kinds are at `law.rs:172-270`. The artifact categories are `yes`/`no` for a binary target, and `active`, the causes, then `scheduled_stop` for an event target (`law.rs:133-145`).

Tensors of the spec (`law.rs:374-509`). For each feature: `x.<f>.centre`, `x.<f>.scale`, `x.<f>.lo`, `x.<f>.hi` (`law.rs:390-437`). For each network `p` (`net_tensors`, `law.rs:301-354`): scalar `p.b0`, then `p.skip[n]`, `p.W1[8,n]`, `p.b1[8]` and `p.w2[8]` (`HIDDEN = 8`, `law.rs:42`; the last three are structural-zero "sparse" tensors, `spec.rs:47-`). For each kind:

| kind | networks and scalars | declared range of `b0` / scalar (spec) | lines |
|---|---|---|---|
| positive | `<t>.median` (ln of the unit), `<t>.spread` (ln-sd), `<t>.tail_weight` $w\in[0,1]$, `<t>.tail_width` $k\in[1,100]$ | median $b_0\in[-80,80]$, spread $b_0\in[0.001,10]$ | `law.rs:442-468` |
| binary | `<t>.logit` | $b_0\in[-40,40]$ | `law.rs:469-477` |
| event | `<t>.<c>.median` (ln s), `<t>.<c>.shape` $k_c\in[0.1,50]$; with a declared stop: `<t>.stop.offset_s[3]` $\in[-10^5,10^5]$ s, `<t>.stop.offset_w[3]` $\in[0,1]$ | median $b_0\in[-20,25]$ | `law.rs:478-507` |
| all | constant `num.substeps` = 512 | fixed | `law.rs:510-515` |

#### A.2 Features and domain refusals

For feature $i$ with raw SI value $x_i$ (read from a resolved, finite prefix condition: `common.rs:50-62`) and typed $c_i,s_i,\ell_i,h_i$ (`law.rs:690-737`):

$$
z_i=\begin{cases}(x_i-c_i)/s_i & \text{identity}\\ \ln(x_i/c_i)/s_i & \text{ln}\\ (1/x_i-1/c_i)/s_i & \text{reciprocal}\end{cases}\qquad\text{admitted only if } \ell_i\le x_i\le h_i
$$

(`law.rs:8-9`, `714-736`). The reasons for refusal are collected for every feature together (`law.rs:747-749`), under the gate `Condition` (`law.rs:994-1003`).

#### A.3 The network of every law

$$
N(z)=b_0+\sum_{i}\mathrm{skip}_i\,z_i+\sum_{j=0}^{7}w2_j\,\operatorname{relu}\!\Big(b1_j+\sum_i W1_{ji}\,z_i\Big)
$$

(`law.rs:10`, `657-671`; `crates/ignis-wsp/src/nn.rs:69-84`). With zero features, $N=b_0$ (`law.rs:658-660`). A hinge unit whose `w2` is zero does not contribute, even when its `W1` and `b1` are typed. Several releases below carry such inert rows, which are inherited from parents.

#### A.4 Positive target: a two-component log-normal mixture of one median

$$
m=N_{\text{median}}(z),\qquad s=N_{\text{spread}}(z)>0,\qquad
F(y)=(1-w)\,\Phi\!\Big(\frac{\ln y-m}{s}\Big)+w\,\Phi\!\Big(\frac{\ln y-m}{k s}\Big)
$$

(`law.rs:11-12`, `1007-1019`; `docs/wsp/research/law_forward.md:113-120`). The quantile at level $p$ is $q_p=\exp(m+Q_p)$, where $Q_p$ solves $(1-w)\Phi(Q/s)+w\Phi(Q/(ks))=p$ (`law.rs:757-780`, `1029-1032`):

$$
Q_p=\begin{cases}s\,\Phi^{-1}(p) & w=0\ \text{or}\ k=1\\ k s\,\Phi^{-1}(p) & w=1\\ \text{200-step bisection on }[\min,\max](s\Phi^{-1}(p),\,ks\Phi^{-1}(p)) & \text{otherwise}\end{cases}
$$

The mean is reported when finite (`law.rs:1039-1040`):

$$
\mathbb E[Y]=e^{m}\big[(1-w)e^{s^2/2}+w\,e^{k^2s^2/2}\big]
$$

$\Phi^{-1}$ is Acklam's rational approximation with relative error below $1.2\times10^{-9}$ (`crates/ignis-wsp/src/quad.rs:25-73`), and $\Phi(x)=\tfrac12\operatorname{erfc}(-x/\sqrt2)$ (`quad.rs:76-78`). The default levels are $p\in\{0.05,0.25,0.5,0.75,0.95\}$ (`common.rs:14`, `64-71`). The release's uncertainty kind is `assumed_noise`: the spread tensors are the author's statement (`law.rs:1188`, `law_tools.rs:55-59`).

#### A.5 Binary target

$$P(\text{yes})=\sigma\big(N_{\text{logit}}(z)\big)$$

(`law.rs:13`, `1051-1062`; `nn.rs:44-51`). The artifact carries $(P,1-P)$ as categories `yes` and `no` (`law.rs:1207-1210`).

#### A.6 Event target: competing log-logistic clocks

The kernel's clocks are log-logistic, not Weibull. It returns cumulative incidence per cause; it does not return a hazard or a survival curve as the artifact (`law.rs:14-15`, `782-864`, `1064-1150`; `law_forward.md:129-158`). For each cause $c$, with $\tau_c=\exp N_c(z)$ and the scalar shape $k_c$:

$$
S_c(t)=\frac{1}{1+(t/\tau_c)^{k_c}},\qquad
\ln S(t)=-\sum_c \operatorname{sp}\!\big(k_c(\ln t-\ln\tau_c)\big),\qquad
h_c(t)=\frac{k_c}{t}\,\sigma\!\big(k_c(\ln t-\ln\tau_c)\big)
$$

(`law.rs:789-807`; the declared meaning of the shape is $\operatorname{sd}(\ln T)=\pi/(k_c\sqrt3)$, `law.rs:495`). Conditioned on no event before the cutoff $\tau$ (`prefix.cutoff_s`) and with a stop $t_s>\tau$:

$$
\text{active}(t)=\frac{S(t)}{S(\tau)}\ (t<t_s),\qquad
F_c(t)=\frac{1}{S(\tau)}\int_\tau^{\min(t,t_s)} h_c(u)\,S(u)\,du,\qquad
\text{scheduled\_stop}(t)=\frac{S(t_s)}{S(\tau)}\ (t\ge t_s)
$$

The numerical form (`law.rs:811-864`) works as follows. Each requested interval up to $\min(t_j,t_s)$ is cut into $n=512$ equal steps (`num.substeps`). Over a step the exact survival loss $\Delta=S(u_0)-S(u_1)$ is computed, with $S(u)=\min(1,e^{\ln S(u)-\ln S(\tau)})$ held monotone. That loss is shared among the causes in proportion to $h_c$ at the step midpoint, or equally if the total hazard is 0 (`law.rs:836-851`).

The stop (`law.rs:923-934`, `1094-1141`):
- The control stop $t_\text{ctl}$ is the earliest `t_start_s` of a known control named `scheduled_stop` or `flow_stop`.
- Suppose the declaration names a stop condition $C$ (resolved, finite, $>0$) and $W=\sum_{k:\,w_k>0}w_k>0$, with $w_k$ = `offset_w[k]`. Then the table is a mixture over the passes $k$ with $w_k>0$, each weighted $w_k/W$ and run with stop $\min(t_\text{ctl},\,C+\text{offset\_s}[k])$.
- Otherwise there is one pass of weight 1 with stop $t_\text{ctl}$, or no stop at all.

The rows sum to one at every time (`law_forward.md:147`).

#### A.7 Refusals

| condition | outcome | lines |
|---|---|---|
| a release built with another forward hash | hard refusal `forward_mismatch` | `law.rs:947-962` |
| feature tensors cannot define the transform ($\ell>h$, $s\le0$, ln centre $\le0$, reciprocal centre $=0$) | hard refusal `release_tensors` | `law.rs:694-703` |
| release state `Failed` or `Unsupported`; prefix family ≠ spec family; unknown target | `Semantic` | `law.rs:963-986`, `904-921` |
| cutoff not finite or $<0$ | `Horizon` | `law.rs:987-993` |
| condition missing, unresolved or non-finite; $x\notin[\ell,h]$; $x\le0$ under ln; $x=0$ under reciprocal; non-finite $z$ | `Condition` (all reasons listed) | `common.rs:50-62`, `law.rs:704-749` |
| positive: $s\le0$ or non-finite, or $m$ non-finite | `Numerical` | `law.rs:1010-1015` |
| positive: levels not strictly increasing in $(0,1)$ | `Semantic` | `law.rs:1021-1028` |
| positive: a quantile not finite and $>0$ | `Numerical` | `law.rs:1033-1038` |
| binary: non-finite logit | `Numerical` | `law.rs:1053-1058` |
| event: non-finite clock | `Numerical` | `law.rs:1073-1078` |
| event: neither `times_s` nor `horizon_s` given, or no time $\ge\tau$ | `Horizon` | `law.rs:1079-1093` |
| event: a stop at or before the cutoff | `Semantic` | `law.rs:1126-1131` |

Nothing is fitted at request time (`law_tools.rs:60-64`). What the form cannot express is listed in `law_forward.md:361-376`: a feature-dependent clock shape or tail weight, clocks other than log-logistic, more than 3 causes, and so on.

#### A.8 Scores applied downstream (not part of the forward)

The forward emits quantiles, probabilities and cumulative-incidence rows. The scores are computed by the lane scorers (research repo):

- **Quantile score ×2, equal weights** (SPICE, CLD radiometer, plastics). For SPICE: `research:tools/wsp/ext/spice_cases.py:140-146`, $\operatorname{QS}=\tfrac{2}{5}\sum_{k}\big(\tau_k-\mathbb 1[y<q_k]\big)(y-q_k)$. CLD uses the same form (`research:tools/wsp/ext/cld_cases.py:512-514`).
- **Interval form** (plastics, where the outcome is the interval $[y_\text{lo},y_\text{hi}]$): $\tfrac25\sum_k \rho_{\tau_k}(q_k;[y_\text{lo},y_\text{hi}])$ with $\rho=\tau(y_\text{lo}-q)$ if $q<y_\text{lo}$, $(1-\tau)(q-y_\text{hi})$ if $q>y_\text{hi}$, and 0 inside the interval (`research:tools/wsp/ext/bass1_cases.py:641-651`).
- **Quantile CRPS, trapezoid weights** (E-FIELD current, SAME-R $\ln d_{10}$): $2\sum_k\frac{\tau_{k+1}-\tau_{k-1}}{2}\,\rho_{\tau_k}(q_k,y)$ with $\tau_0=0$ and $\tau_{K+1}=1$ (`research:tools/wsp/ext/bre_cases.py:502-509`, `samer_cases.py:78-83`, `efield_cases.py:528-535`).
- **Brier**: $(p-y)^2$ (SLICE, SAME-R detector, plastics kept burning).
- **IPCW Brier and integrated Brier** (BRE, CLD, E-FIELD events). At horizon $h$: $\frac1n\sum_i \hat G^{-1}\,(I_i-\hat F_i(h))^2$, with $\hat G$ the Kaplan–Meier censoring survival. The integrated Brier is the trapezoid mean over the horizons, $\frac{1}{h_K-h_1}\sum_i (h_i-h_{i-1})\frac{B_i+B_{i-1}}{2}$ (`research:tools/wsp/ext/transfer_acme.py:185-272`).

For event targets the final list scores the sum over every declared cause (`research:tools/wsp/final/releases_final_gen7.json`, key `scoring`).

---

### B. The two published-law baselines in `law_score.py` (exact)

#### B.1 "Lin and Faeth coflow law, train sigma" (SPICE, T2 only)

Built at `research:tools/wsp/ext/law_score.py:240-246`:

```python
240  # T2: the Lin and Faeth coflow law at its published constants, one log-normal spread from the train residuals
241  r = [math.log(c["outcome"]["L_sp_mm"] / L.lin_faeth_coflow_mm(c)) for c in train if L.lin_faeth_coflow_mm(c)]
242  sig = statistics.pstdev(r)
243  lf = {c["case_id"]: L.lognormal_q(L.lin_faeth_coflow_mm(c), sig) for c in rows
244        if c.get("inputs_given_flow") and L.lin_faeth_coflow_mm(c)}
```

The median and the quantiles are defined at `research:tools/wsp/ext/spice_cases.py:186-197` and `:47`:

$$
L_\text{LF}\,[\text{mm}]=\frac{\dot m\,[\text{mg/s}]\times10^{-3}}{4\pi\,\mu_\text{air}\,[\text{g m}^{-1}\text{s}^{-1}]\;Z_\text{st}}\times10^{3},
\qquad
q_p=L_\text{LF}\,e^{z_p\hat\sigma},\quad z_p\in\{-1.6449,-0.6745,0,0.6745,1.6449\}
$$

- The docstring states it as Spalding / Lin & Faeth, "L/d = Re/(16 Zst) … ⇒ L = mdot/(4 pi mu Zst)", with unity coefficients (`spice_cases.py:191-192`).
- $\dot m$ is `inputs_given_flow.mdot_jet_mg_s`, the investigators' total nozzle mass flow. $\mu_\text{air}$ is `mu_air_mean_g_m_s` and $Z_\text{st}$ is `zst_jet` = `mdot_total_mg_s / mdot_over_zst`. All three are columns of the "Smoke Point Datasheet" (`spice_cases.py:50-53`, `96-98`, `116-123`).
- $\hat\sigma$ is the population standard deviation of $\ln(L_{sp}/L_\text{LF})$ over the train cases (`law_score.py:241-242`). Recomputed here from `research:runs/dev/data_all/work/transfer_spice/cases_train.jsonl`: 42 of 42 train cases, $\hat\sigma=0.1218$. `research:docs/wsp/feedback/spice_feasibility.md:115` reports "sigma_ln 0.122".
- The baseline is scored only for T2 and against $L_{sp}$ (`law_score.py:246`, `251-252`; the default target of `spice_cases.score` is `L_sp_mm`, `spice_cases.py:149`).

#### B.2 "Bhattacharjee et al. 2016, v1 train log width" (plastics quench speed)

Built at `research:tools/wsp/ext/law_score.py:593-605`. This is the branch for coding `v2`, which is the coding of the final list's `bass` entry (`"coding": "v2"`):

```python
593  s = b["quench_bhattacharjee_2016"]["log_width_fitted"]
...
598  for r in rows:
599      if r.get("x_o2_initial") is not None:
600          u = L.v_crit_bhattacharjee(r["x_o2_initial"], r.get("pressure_atm"))
601          if u is not None:
602              bh[r["case_id"]] = L.lognormal_quantiles(u, s)
605  "Bhattacharjee et al. 2016, v1 train log width": {"quench": bh},
```

The formula is at `research:tools/wsp/ext/bass1_cases.py:693-705` and `716-718`:

$$
V_{g,\text{cr}}\,[\text{cm/s}]=100\,\frac{\varepsilon\,\sigma_\text{SB}\,(T_v^4-T_\infty^4)}{\rho_g\,c_g\,(T_f-T_v)},\qquad
T_f=T_\infty+(2199-T_\infty)\,\frac{X_{O_2}}{0.21},\qquad
\rho_g=\frac{101325\,p_\text{atm}}{287\,T_v}
$$

with $\varepsilon=1$, $\sigma_\text{SB}=5.670\times10^{-8}$, $T_v=673$ K, $T_\infty=300$ K, $c_g=1070$ J kg⁻¹ K⁻¹, and $p_\text{atm}=1$ when absent. The baseline is undefined (no forecast) when $T_f\le T_v+50$.

- The docstring cites the restatement as NASA/TM-20210011385 §2.2.7 eq. 2.5, and states that $\varepsilon$, $T_\infty$, $c_g$, the oxygen scaling of $T_f$ and $\rho_g$ were "set here" (`bass1_cases.py:694-698`).
- Quantiles: $q_p=V_{g,\text{cr}}\,e^{s z_p}$, with the same five $z_p$ as in B.1.
- The width is $s=0.4$ (`research:runs/dev/data_all/work/transfer_bass1/baselines_train.json`, `quench_bhattacharjee_2016.log_width_fitted`). It is the grid value $s\in\{0.05,0.10,\dots,2.00\}$ that minimises the train interval score, mean of day means, on the v1 train file (`bass1_cases.py:729-740`). The same file gives the median $1.304$ cm/s at $X=0.21$ (`median_at_x_o2`).
- It is scored with the interval form of A.8.

---

### C. Per investigation: the adopted release and its law

The final list is `research:tools/wsp/final/releases_final_gen7.json`, frozen on 15 November. "ZS" means zero-shot and "DI" means data-informed, as defined in `research:docs/wsp/COVERAGE_19.md:4-6`. The exposure comes from each release's `meta.json`.

| Investigation | adopted (final list) | exposure | also noted |
|---|---|---|---|
| BRE (PSI-20) | `bre/d3-fable`, read by the `lasts` rule | DI (train + validation; PROTOCOL 13.6) | |
| CLD Flame (PSI-21) | `cld3/s1-fable` | DI (train + validation; typed, not run, by its author) | `cld/d1-fable` is the one served after the final run (`COVERAGE_19.md:74`); DI (train) |
| E-FIELD (PSI-22) | `efield3/s1-fable` | DI (train + validation) | |
| SLICE (PSI-106) | `slice2/s1-fable` | DI (second law of PROTOCOL 13.4; train + validation aggregates) | |
| SPICE (PSI-107) | `spice/p3-fable` | DI (train + validation) | `spice/lit-fable`: no dataset exposure, but T1 uses the investigators' published fit of the same smoke points, so it is not blind (`meta.json`) |
| SAME-R (PSI-101) | `samer/s1-fable` | DI (train + validation) | |
| SAME (PSI-102), DAFT (PSI-47) | not in the list: "no test days (held whole)" (`same_and_daft`) | `same/w0-fable`, `daft/w0-fable`: ZS | |
| Plastics and fabrics (PSI-26, PSI-25) | `bass4/s1-fable` | DI (bass1 train + validation); the SIBAL lines ZS, the Nomex/Ultem floor DI from ten burns | `bass5/s1-fable`: post-final, in-sample on every partition, no held-out score (PROTOCOL 13.8); served (`COVERAGE_19.md:59`). `bass1/w0-fable` (ZS) is a listed comparator |
| s-Flame (PSI-23) | not a law slug | | `sflame_event` is `status: placeholder`, with no features and one dummy target (`configs/models/law/sflame_event/decl.json`); no release exists. s-Flame is served by `acme_normal.wsp13` / `f4-hydrogen`, which are not `law.rs` (`COVERAGE_19.md:27`, `59`) |

All feature scales are 1 unless stated otherwise. Every $\ln$ transform below is $z=\ln(x/c)$.

#### C.1 SPICE: `spice/p3-fable` (`sha256:000b88b8…98a0`), with `lit-fable` beside it

What is scored (`research:tools/wsp/ext/law_score.py:18-19`, `219-229`):
- T1 is `smoke_point_length`, with `fuel_mass_flow` set to its centre $10^{-6}$ kg/s, a feature the smoke-point targets give weight 0.
- T2 is `flame_length` at the given smoke-point flow.
- Both are taken as quantiles ×1000 (mm) and scored with the equal-weight quantile score (A.8), mean of day means.
- The other two targets, `smoke_point_fuel_flow` and `emits_soot`, are typed but not scored.

The lineage is root → `w0-fable` (variant B) → `p2-fable` → `p3-fable`. P2 replaced the smoke-point length and P3 replaced the flame length (`configs/models/law/spice/p2-fable/edits.wsp`, `p3-fable/edits.wsp`).

| symbol | edits.wsp name | meaning | centre $c$ | domain $[\ell,h]$ | unit |
|---|---|---|---|---|---|
| $z_d$ | `x.nozzle_diameter.*` | ln nozzle inner diameter | 0.001 | [0.0003, 0.004] | m |
| $z_u$ | `x.coflow_speed.*` | ln duct air speed | 0.25 | [0.03, 0.9] | m/s |
| $z_m$ | `x.fuel_mass_flow.*` | ln nozzle mass flow | 1e-6 | [5e-8, 2e-5] | kg/s |
| $z_x$ | `x.x_fuel.*` | ln fuel mole fraction | 1.0 | [0.4, 1.0] | 1 |
| $f_p, f_{py}, f_m$ | `x.is_propane/propylene/methane.*` | indicators (identity; ethylene is the reference) | 0 | [0, 1] | 1 |

**T1, smoke-point length (the P1 law, unchanged in P3).** The author's reading: $L_{sp}=\max(L_0,\,c\,u^{0.55}(d/1.6\,\text{mm})^{-0.33}/x)\,(d_c/d)^3$ for $d<d_c$, with $L_0$ = 21, 28, 14 mm and $d_c$ = 0.96, 0.80, 0.56 mm for ethylene, propane, propylene (`configs/models/law/spice/p2-fable/edits.wsp:3-7`). As typed:

$$
\ln\frac{L_{sp}}{1\,\text{m}}=-3.863+0.288f_p-0.405f_{py}+2.0f_m
+\operatorname{relu}(0.970-0.33z_d+0.55z_u-z_x-0.045f_p-0.630f_{py})
+3.0\,\operatorname{relu}(-0.041-z_d-1.1z_x-0.182f_p-0.539f_{py})
$$

The spread is $s=0.18$ (`smoke_point_length.spread.b0`). The spread hinge rows are typed but their `w2` are 0. The tails are $w=0.1$ and $k=3.0$. The value $e^{-3.863}$ is 0.021 m, the ethylene still-air floor (`spice/w0-fable/edits.wsp:34`).

**T2, flame length at a given flow (P3).** The author's reading: $L=c_f[\text{fuel}]\,K\dot m/Z_\text{st}\,(r/0.3)^{-0.08}+1.3d$ with $r=u_\text{air}/u_\text{fuel}$ (`configs/models/law/spice/p3-fable/edits.wsp:3-8`). Here $K=C_f\,Sc/(4\pi\mu)=1296$ m per kg/s (`research:docs/wsp/research/phys_spice_w0.md:223`), and $c_f$ = 1.079 for ethylene, 1.169 for propane, $1.324\,x^{0.22}$ for propylene (`research:docs/wsp/research/phys_spice_study.md:223-224`). As typed:

$$
\ln\frac{L}{1\,\text{m}}=\underbrace{-3.797-0.16z_d-0.08z_u+1.08z_m+0.858z_x+0.099f_p+0.173f_{py}+0.188f_m}_{\ln L_\text{far}}
+0.31\operatorname{relu}(t+2.2)+0.38\operatorname{relu}(t)+0.31\operatorname{relu}(t-2.2)
$$

$$
t=-2.848+1.16z_d+0.08z_u-1.08z_m-0.858z_x-0.099f_p-0.173f_{py}-0.188f_m=\ln\!\big(1.3\,d/L_\text{far}\big)
$$

The equality on the right is an identity of the typed numbers, because $\ln(1.3\times10^{-3})+3.797=-2.848$. The three hinges are the three-piece approximation of $\ln(1+e^{t})$, that is of $\ln(1+1.3d/L_\text{far})$ (`p2-fable/edits.wsp:17-19`). Hinge units 0 and 1, the W0 velocity-ratio step, have `w2 = 0` (`p3-fable/edits.wsp:11-12`). The spread is $s=0.10$, with $w=0.1$ and $k=3.0$.

| tensor | value |
|---|---|
| `flame_length.median.b0` | −3.797 |
| `flame_length.median.skip[0:7]` | [−0.16, −0.08, 1.08, 0.858, 0.099, 0.173, 0.188] |
| `flame_length.median.W1[2..4, 0:7]` | [1.16, 0.08, −1.08, −0.858, −0.099, −0.173, −0.188] |
| `flame_length.median.b1[2..4]` | −0.648, −2.848, −5.048 |
| `flame_length.median.w2[2..4]` | 0.31, 0.38, 0.31 |
| `flame_length.spread.b0` | 0.10 |
| `smoke_point_length.median.b0`, `.skip[0:7]` | −3.863; [0, 0, 0, 0, 0.288, −0.405, 2.0] |
| `smoke_point_length.median.W1[0,:]`, `b1[0]`, `w2[0]` | [−0.33, 0.55, 0, −1.0, −0.045, −0.630, 0]; 0.970; 1.0 |
| `smoke_point_length.median.W1[1,:]`, `b1[1]`, `w2[1]` | [−1.0, 0, 0, −1.1, −0.182, −0.539, 0]; −0.041; 3.0 |
| `smoke_point_length.spread.b0`; tails | 0.18; `tail_weight` 0.1, `tail_width` 3.0 |

**`spice/lit-fable`** (`sha256:40c345b0…d3bf`; the W0 section 7.4 main variant, typed verbatim). The source of T1 is Dotson et al. 2011, the published fit of the same smoke points, so T1 is not blind. The comment reads "1.342 = ln(21.2 mm * 25^0.414 / 21 mm); exponents -0.91 (diameter) and 0.414 (coflow), Dotson 2011" (`configs/models/law/spice/lit-fable/edits.wsp:34-49`):

$$
\ln\frac{L_{sp}}{1\,\text{m}}=-3.863-0.82z_x+0.287f_p-1.030f_{py}+2.0f_m
+\operatorname{relu}\!\big(h\big)+0.65\operatorname{relu}(-0.288-z_x),\quad
h=1.342-0.91z_d+0.414z_u-0.423f_p-0.224f_{py}
$$

$$
s=0.30-0.2\operatorname{relu}(h)+0.2\operatorname{relu}(h-0.5)\quad(\text{from }0.30\text{ at the floor to }0.20),\qquad w=0.1,\ k=3.0
$$

Its flame length (`lit-fable/edits.wsp:95-116`; $L=K\dot m F/Z_\text{st}$ with a soot correction) is:

$$
\ln\frac{L}{1\,\text{m}}=-3.895+z_m+0.66z_x+0.055f_p+0.143f_m+0.148\big[\operatorname{relu}(\rho)-\operatorname{relu}(\rho-3.219)\big]-0.14\big[\operatorname{relu}(a)-\operatorname{relu}(a-1.1)\big]
$$

with:
- $\rho=0.784-2z_d-z_u+z_m-0.27z_x-0.452f_p-0.405f_{py}+0.559f_m$, which is $\ln(0.5/r)$ per the comment;
- $a=1.374-0.91z_d+0.414z_u-z_m-1.48z_x-0.191f_p-1.254f_{py}+1.857f_m$;
- spread $s=0.22$, $w=0.1$, $k=3.0$.

#### C.2 Plastics and fabrics: `bass4/s1-fable` (`sha256:ceefc15d…ab86`), with `bass1` W0 and `bass5`

What is scored (`research:tools/wsp/ext/law_score.py:537-589`, `502-506`; coding v2):
- quench speed: `quench_speed` for rods, spheres and sections ("nose"), `flat_quench_speed` for films and slabs, ×100 (cm/s), interval score (A.8);
- kept burning: $P=\min(P_\text{low},P_\text{high})$ of the geometry's two `holds` targets, scored with Brier;
- `oxygen_limit` and `blowoff_speed` are typed but not scored.

The lineage is root → bass3 s1 (which carries bass2 s1, bass1 D1 and bass1 W0 lines written in place) → bass4 (`configs/models/law/bass4/s1-fable/edits.wsp:1-4`).

| symbol | name | meaning | $c$ | domain | unit |
|---|---|---|---|---|---|
| $z_x$ | `x.x_o2` | ln O₂ mole fraction entering the duct | 0.18 | [0.12, 0.25] | 1 |
| $z_u$ | `x.flow_speed` | ln duct speed | 0.01 | [0.003, 0.6] | m/s |
| $z_r$ | `x.nose_radius` | ln radius (0.01 for flat samples) | 0.01 | [0.002, 0.03] | m |
| $z_t$ | `x.heat_time` | ln time from ignition (200 s default) | 200 | [20, 4000] | s |
| conc | `x.is_concurrent` | flat, flame spreading with the flow | 0 | [0, 1] | 1 |
| rod | `x.is_rod` | rod | 0 | [0, 1] | 1 |
| $z_{th}$ | `x.flat_thickness` | ln flat thickness (0.0005 for nose samples) | 0.0005 | [5e-5, 0.03] | m |
| $z_p$ | `x.u_pot1` | ln of the burn's fan curve at display 46.5 (pot 1) | 0.044 | [0.01, 0.15] | m/s |
| os | `x.is_one_sided` | one side burning | 0 | [0, 1] | 1 |
| sib, nu | `x.is_sibal`, `x.is_nomex_ultem` | fabric classes | 0 | [0, 1] | 1 |

**Nose quench speed** ("out at pot 0.31 whatever radius, oxygen, time"; $b_0=\ln(0.941\times0.31\times0.044\ \text{m/s})$, `bass4/s1-fable/edits.wsp:282-286`):

$$
\ln\frac{u_q}{1\,\text{m/s}}=-4.356+z_p\quad\Longleftrightarrow\quad u_q=e^{-4.356}\,\frac{u_\text{pot1}}{0.044}=0.941\times0.31\times u_\text{pot1}=0.292\,u_\text{pot1},\qquad s=0.15,\ w=0.1,\ k=2.0
$$

The four W0 hinge rows of $\ln(1-e^{-v})$ remain typed with `W1[:,5] = −20` for rods, and the median and spread `w2` are 0 since bass3. So nothing depends on $X$, $R$ or $t$.

**Flat quench speed** (`edits.wsp:287-305`). Floor level, two-class threshold, and cap:

$$
\ell_0=-4.126-0.6z_x+0.405\,\text{conc}+z_p,\qquad
g=30(z_c-z_x),\ \ z_c=-0.204+0.147z_{th}+0.28\,\text{os},\qquad
\ell_\text{cap}=-2.744+0.3z_{th}+0.2\,\text{os}
$$

$$
\ln\frac{u_{fq}}{1\,\text{m/s}}=\ell_0+\operatorname{relu}(g)-\operatorname{relu}(\ell_0+g-\ell_\text{cap})
$$

$$
s=0.15+0.7\operatorname{relu}(g)-1.18\operatorname{relu}(g-0.93)+0.48\operatorname{relu}(g-1.87),\qquad w=0.15,\ k=2.5
$$

The meanings are:
- $\ell_0$ is pot 0.39 at 18 % O₂ (`b0` $=\ln(0.941\times0.39\times0.044)$).
- $z_c$ is the starting oxygen below which a slab reaches its limit during the burn: 18.0 % at 2 mm and 20.6 % at 5 mm, two sides.
- $\ell_\text{cap}$ is 11 cm/s $\times(\text{thickness}/3\,\text{mm})^{0.3}$, times 1.22 one-sided.

The typed tensors are unit 3 (`W1[3,{0,6,8}]` = −30, 4.41, 8.4; `b1[3]` = −6.12; `w2[3]` = 1.0) and unit 4 (`W1[4,{0,4,6,7,8}]` = −30.6, 0.405, 4.11, 1.0, 8.2; `b1[4]` = −7.502; `w2[4]` = −1.0). The second unit's row is exactly $\ell_0+g-\ell_\text{cap}$.

**Kept burning, nose** (W0 lines; $l_f=28\ln(X/0.134)$, `edits.wsp:28-41`):

$$
\operatorname{logit}P_\text{low}=8.26+28z_x-\operatorname{relu}(8.04+21.7z_x-1.58z_u+3.67z_r-1.31z_t)
$$

$$
\operatorname{logit}P_\text{high}=8.26+28z_x-\sum_{j=0}^{3}c_j\operatorname{relu}(b_j+z_u-0.75z_r),\quad b=(-0.917,-2.527,-3.402,-4.319),\ c=(1.09,1.54,3.0,3.86)
$$

**Kept burning, flat**, including the bass4 fabric floors (`edits.wsp:175-190`, `336-345`). SIBAL moves the floor to 17.0 % ($-20\ln(0.170/0.138)$) and Nomex/Ultem to 23.4 %; at high flow the floors are 15.8 % and 21.5 %:

$$
\operatorname{logit}P_\text{low}=5.31+20z_x-4.17\,\text{sib}-10.56\,\text{nu}-\operatorname{relu}(5.33+15.5z_x-3.75z_u+1.52\,\text{conc}-4.17\,\text{sib}-10.56\,\text{nu})
$$

$$
\operatorname{logit}P_\text{high}=5.31+20z_x-2.71\,\text{sib}-8.87\,\text{nu}-\sum_{j=0}^{4}c_j\operatorname{relu}(b_j+z_u-0.693\,\text{conc}),\quad b=(0,-1.609,-2.303,-2.996,-3.689),\ c=(0.56,0.98,1.32,2.1,2.3)
$$

**`bass1/w0-fable`** (`sha256:97c0daf9…39a2`; ZS, comparator). The author's physics reads $u_q=2Ra_q/3-D_c/R$ for the nose, with $v=\ln(\text{forced}/\text{diffusive})$, and $u_{fq}=c_\text{dir}\,0.151\ \text{cm/s}/(X-0.03)$ for flat samples (`bass4/s1-fable/edits.wsp:104-111`, `208`). The features are the first five above:

$$
\ln\frac{u_q}{1\,\text{m/s}}=-4.067-2.4z_x+z_r-0.5z_t-\sum_{j=0}^{3}c_j\operatorname{relu}(b_j+2.4z_x-2z_r+0.5z_t),\quad b=(2.334,0.534,-0.066,-0.366),\ c=(0.2,0.53,1.12,2.15)
$$

$$
s=0.75+0.3\operatorname{relu}(0.534+2.4z_x-2z_r+0.5z_t)+0.6\operatorname{relu}(-0.066+2.4z_x-2z_r+0.5z_t),\quad w=0.1,\ k=2.0
$$

$$
\ln\frac{u_{fq}}{1\,\text{m/s}}=-4.601-1.2z_x+0.405\,\text{conc},\quad s=0.5,\ w=0.15,\ k=2.5
$$

The hinge arguments equal $v_j-v$ with $v=0.666-2.4z_x+2z_r-0.5z_t$ and knots $v_j$ = 3.0, 1.2, 0.6, 0.3. `bass1/d1-fable` adds $+1.44\operatorname{relu}(z_r)$ to the flat median and $+0.43\operatorname{relu}(z_r)$ to its spread; slabs thicker than 0.5 mm are supplied a nose radius of 0.02 (`configs/models/law/bass1/d1-fable/edits.wsp`).

**`bass5/s1-fable`** (`sha256:050a4ceb…b82d`; post-final, in-sample). It is bass4 plus $z_w=\ln(W/0.02\,\text{m})$, domain [0.002, 0.1] m, entered as `W1[{3,4},11] = −3.3` in the flat-quench median and `W1[{3,4,5},11] = −3.3` in its spread. That is $z_c\to z_c-0.11z_w$, which the author explains as "a thick slab in a narrow frame loses heat sideways into the holder (4 lambda dT d / W^2)" (`configs/models/law/bass5/s1-fable/edits.wsp:349-352`). Every other number is bass4's (diff of the two dumps).

#### C.3 BRE: `bre/d3-fable` (`sha256:27e2af51…b955`), read by the `lasts` rule

The physics is self-extinction of a flame over a porous burner by radiation as it grows by diffusion (`configs/models/law/bre/decl.json`). The time origin is ignition. The lineage is root → W0 → d3.

| symbol | name | $c$ | domain | unit |
|---|---|---|---|---|
| $z_d$ | burner_diameter | 0.025 | [0.02, 0.06] | m |
| $z_m$ | burner_mass_flux | 0.004 | [0.0003, 0.015] | kg m⁻² s⁻¹ |
| $z_o$ | x_o2 | 0.265 | [0.18, 0.45] | 1 |
| $z_p$ | pressure | 101325 | [5e4, 1.05e5] | Pa |
| $z_f$ | x_fuel | 1.0 | [0.2, 1.0] | 1 |
| $z_c$ | is_methane (identity) | 0 | [0, 1] | 1 |
| $z_t$ | stated_time | 30 | [1, 3000] | s |

The author's reading (`configs/models/law/bre/d3-fable/edits.wsp:5-11`):
- size ratio $b=0.514+1.01z_d+0.47z_m-0.93z_o+0.43z_f+0.11z_c$;
- clock $\text{base}=2.451+2.05z_d+1.12z_m-0.03z_o+1.12z_f+0.08z_c$;
- stand-off $s_o=2.307+z_d+z_m+z_f=\ln(\Lambda_0 d/25\,\text{mm})$;
- $\operatorname{logit}=5.5(H(v)-b)-9\operatorname{relu}(0.90-s_o)$ with $v=0.950+z_t-\text{base}$.

As typed:

$$
\operatorname{logit}P(\text{lasts to }t)=-2.83-5.56z_d-2.59z_m+5.12z_o-2.37z_f-0.61z_c
+\sum_{j=0}^{5}c_j\operatorname{relu}(b_j+2.05z_d+1.12z_m-0.03z_o+1.12z_f+0.08z_c-z_t)-9\operatorname{relu}(-1.41-z_d-z_m-z_f)
$$

with $b=(2.51,0.26,-0.35,-0.86,-1.30,-1.72)$ and $c=(0.97,0.56,0.67,0.87,1.14,1.62)$. Pressure has weight 0. Unit 7 is typed but `w2[7] = 0`. The skip and $b_0$ are $-5.5\times$ the $b$ coefficients, and each hinge argument is $b_j-1.501-v$.

The scoring rule (`research:docs/wsp/research/phys_bre2.md:19-30`; implemented at `research:tools/wsp/final/run_final_gen7.py:207-235`) uses cutoff $c\ge1$ s, horizons $h$ (6 to 210 s, `research:tools/wsp/ext/bre_cases.py:70`), and a stop $t_s=$ planned stop $+0.569-0.796$ s (the train medians in `research:runs/dev/data_all/work/transfer_bre/baselines_train.json`):

$$
P(\text{ext by }h)=\operatorname{clip}_{[0,1]}\Big(1-\frac{P_\text{lasts}(\min(h,t_s))}{P_\text{lasts}(c)}\Big),\qquad
P(\text{operator termination by }h)=\min\Big(1,\frac{P_\text{lasts}(t_s)}{P_\text{lasts}(c)}\Big)\ \ (h\ge t_s)
$$

These are scored with the IPCW integrated Brier for each cause (A.8).

The `flame_end` clock is not used by the rule. For reference: $\ln\tau=2.09+1.39z_d+0.81z_m+0.57z_o+0.84z_f+0.01z_c+\sum_{j<7}c_j\operatorname{relu}(b_j-1.01z_d-0.47z_m+0.93z_o-0.43z_f-0.11z_c)-6\operatorname{relu}(-1.41-z_d-z_m-z_f)$, with $b$ = (1.286, 0.786, 0.486, 0.256, 0.056, −0.114, −0.264) (that is, knots of $b$ at 1.8 … 0.25), $c$ = (0.29, 0.36, 0.49, 0.70, 1.09, 2.09, 5.03), and shape 3.0.

#### C.4 CLD Flame: `cld3/s1-fable` (`sha256:b9df5239…9e4b`), with `cld/d1-fable` served

The physics is loss of a dilute coflow flame by blow-off, plus its radiometer excess. The time origin is flame onset, and the scheduled stop is the planned fuel-off (`configs/models/law/cld3/decl.json`). What is scored:
- `loss` = the sum of the cause rows at horizons 1 to 120 s, with the planned fuel-off as the declared stop, scored with the integrated Brier;
- radiometer = CRPS of $\log_{10}$ of the volt quantiles (`research:tools/wsp/ext/law_score.py:27-28`).

Features: $z_x=\ln x_\text{final}$ ([0.04, 1]); $z_u=\ln(U_{co}/0.2)$ ([0.02, 1.5] m/s); $z_w=\ln(W/60\,\text{s})$ ([5, 600]); $e$ = ethylene; $z_s=\ln x_\text{stage}$ ([0.04, 1]); $z_v=\ln(v_\text{stage}/0.2)$ ([0.02, 1.5] m/s); $z_T=\ln(t_\text{severe}/20\,\text{s})$ ([2, 600]).

C1's sharp limit has $R=U_{co}/(10.2\,\text{s/m}\;S_L^2)$, $S_L$ linear in $\ln x$ (slope 1.3), and $R_c$ = 0.93 for methane, 1.8 for ethylene (`configs/models/law/cld3/s1-fable/edits.wsp:113-115`). With $-g=3.246+2.6z_x-z_u+3.092e$, so that $g=\ln(R/R_c)$ in that reading:

$$
\ln\tau_\text{blowoff}=4.094+z_w+5\operatorname{relu}(-g)-2\operatorname{relu}(g),\quad k=4.0\qquad(4.094=\ln 60:\ \tau=W\,e^{5\operatorname{relu}(-g)-2\operatorname{relu}(g)})
$$

$$
\ln\tau_\text{crossing}=6.242+z_T-g+41\operatorname{relu}(-g),\quad k=6.0\qquad(\tau=t_\text{severe}R_c/R\ \text{beyond the limit; out of reach below})
$$

$$
\ln\tau_\text{other}=11.156,\quad k=0.5\qquad(\text{"4 \% in 120 s, hazard falling with time"})
$$

The crossing law is `research:docs/wsp/research/phys_cld_study.md:193-209`. The stop is `offset_w` = [1, 0, 0], `offset_s` = [0, 0, 0]: the planned fuel-off itself. Blow-off units 2 and 3 are typed but have `w2 = 0`.

Radiometer: excess = $c_V\chi Q$, plus soot as a three-piece $\ln(1+\chi_\text{soot}x^{2.5}/0.05)$ (`edits.wsp:82-89`):

$$
\ln\frac{V}{1\,\text{V}}=-0.66+0.5e+z_s+z_v+0.31\operatorname{relu}(a+3.299)+0.38\operatorname{relu}(a+1.099)+0.31\operatorname{relu}(a-1.101),\quad a=0.847e+2.5z_s;\quad s=0.8,\ w=0.1,\ k=3.0
$$

**`cld/d1-fable`** (`sha256:a03ec603…2d4f`; served). It has the same features without $z_T$ and causes `blowoff` and `other` only. Its blow-off clock is the W0 form $\ln\tau=\ln W+\ln2.5-\ln\Phi_\text{final}$ after one step:

$$
\ln\tau_\text{blowoff}=6.03+0.55z_x-z_u+z_w+1.21e-0.55\operatorname{relu}(z_x+0.47e)-0.5\operatorname{relu}(-0.693-z_x-0.47e)-0.6\operatorname{relu}(-1.204-z_x-0.47e)-1.0\operatorname{relu}(-1.609-z_x-0.47e),\quad k=3.4
$$

Its `other` clock, stop and radiometer are as in cld3. D1 changed only `blowoff.b0` (6.03), `blowoff.skip[3]` (1.21) and `radiometer.b0` (−0.66) of W0 (`configs/models/law/cld/d1-fable/edits.wsp:3-5`).

#### C.5 E-FIELD: `efield3/s1-fable` (`sha256:8575550b…6f04`)

The physics is the ion current of a flame in a field, plus flame-out by ion wind, by press and by other causes. The time origin of `flame_end` is the field switched on (`configs/models/law/efield3/decl.json`). What is scored: the ion current in µA per planned level (CRPS, trapezoid weights), and the integrated Brier of the sum of the `flame_end` cause rows (`research:tools/wsp/ext/law_score.py:20`; the docstring names blowoff + other, and the final list sums every declared cause).

Features: $z_c=\ln(\dot C/10^{19}\,\text{s}^{-1})$ ([1e18, 1e20]); $z_x=\ln x$ ([0.2, 1]); $e$ = ethylene; $p$ = mesh positive; $z_v=\ln(V/3000\,\text{V})$ ([100, 1e4]); $z_M=\ln(V_\text{max}/5000\,\text{V})$ ([100, 1e4]); $z_w=\ln(W/100\,\text{s})$ ([5, 2000]); gas = gas-jet burner; scb = still-coflow burner; str = $u_{co}\ge0.1$ m/s.

**Ion current.** The author's reading: a rising branch of 0.25 µA/kV² below 1 kV with exponent 1.4 above, capped by an ion-pair yield of 0.8×10⁻⁶ per carbon (mesh negative) or by fixed positive ceilings (`configs/models/law/efield3/s1-fable/edits.wsp:110-151`). As typed:

$$
\ln\frac{I}{1\,\text{A}}=-13.664+0.9p+1.4z_v+0.35\,\text{gas}-\operatorname{relu}(u_0)-\operatorname{relu}(u_1)-0.6\operatorname{relu}(u_2)-\operatorname{relu}(u_3)-\operatorname{relu}(u_4)
$$

| unit | argument $u_j$ | author's meaning |
|---|---|---|
| 0 | $-0.095-z_c-0.4z_x-1.1e-20p+1.4z_v+0.35\,\text{gas}$ | negative-mesh cap (yield per carbon, $x^{0.4}$, ethylene ×3) |
| 1 | $-18.255-0.25z_c-0.95e+20p+1.4z_v-20\,\text{gas}+1.2\,\text{scb}-20\,\text{str}$ | positive ceiling of the coflow burner (off for the gas jet and a real coflow) |
| 2 | $-1.0986-z_v$ | steeper rise below 1 kV |
| 3 | $-37.8-1.754e+20p+1.4z_v+20\,\text{gas}$ | positive ceiling of the gas jet, 0.45 µA methane, 2.6 µA ethylene |
| 4 | $-38.545-z_c-0.4z_x-1.1e+20p+1.4z_v+20\,\text{str}$ | positive mesh with a real coflow: negative ceiling ×0.52 |

The spread is $s=0.45$ (one spread hinge is typed with `w2 = 0`), with $w=0.1$ and $k=3.0$.

**Flame end.** There is no declared stop:

$$
\ln\tau_\text{blowoff}=7.09-z_M+z_w-2.106\operatorname{relu}(\text{gas}-e-p),\ k=12;\qquad
\ln\tau_\text{press}=7.09-z_M+z_w-2.148\operatorname{relu}(-1+\text{gas}-e+p),\ k=4
$$

$$
\ln\tau_\text{other}=8.006-4.2\operatorname{relu}(-1+\text{scb}-e+p)-1.6\operatorname{relu}(\text{scb}-e-p),\ k=1.5
$$

$7.09=\ln(100\,\text{s}\times60\,\text{kV}/5\,\text{kV})$, and the offsets $-2.106=\ln(7.3/60)$ and $-2.148=\ln(7.0/60)$ correspond to the author's 7.3 kV (gas-jet methane, mesh negative) and 7.0 kV (mesh positive). For `other`, $e^{8.006}$ = 3000 s, giving 45 s with mesh positive and 600 s negative in still gas on the coflow burner (`edits.wsp:155-171`).

#### C.6 SLICE: `slice2/s1-fable` (`sha256:d586d0d3…0b61`)

The physics is lifting and blow-off of a jet-flame base through a stability number (`research:docs/wsp/research/phys_slice_w0.md:17-19`, `160-167`):

$$
v_k=0.66U_{co}+8\,l_j\,(Z_\text{st}/Z_{\text{st,CH}_4})\,U_j/d,\ \ l_j=3.3\,\mu\text{m};\qquad \Phi=v_k/(1.4S_L);\quad\text{lifted at }\Phi>1,\ \text{blown off at }\Phi>1.3
$$

Features: $z_{co}=\ln(U_{co}/0.3)$ ([0.02, 1.5] m/s); $z_j=\ln(U_j/1)$ ([0.02, 100] m/s); $z_d=\ln(d/0.0016)$ ([0.0003, 0.004] m); $z_x=\ln x$ ([0.3, 1]); $z_o=\ln(X_{O_2}/0.21)$ ([0.18, 0.25]); $e_2$ = ethane; $e_3$ = propane; $z_m=\ln(\dot m/10^{-6})$ ([2e-8, 5e-5] kg/s).

$$
\ln\Phi=-0.987+z_{co}-0.55z_x-3.0z_o-0.076(e_2+e_3)+\operatorname{sp}_3(t)+0.5\operatorname{relu}(-0.693-z_x),\quad
t=-2.485-z_{co}+z_j-z_d-1.3z_x+0.065e_2+0.088e_3
$$

with $\operatorname{sp}_3(t)=0.31\operatorname{relu}(t+2.2)+0.38\operatorname{relu}(t)+0.31\operatorname{relu}(t-2.2)$. The typed logits are exact multiples of this (`configs/models/law/slice2/s1-fable/edits.wsp:38-109`):

$$
\operatorname{logit}P_\text{blow\_off}=-2.184+2.0\ln\Phi\ \ (b_0=-4.158),\qquad
\operatorname{logit}P_\text{past\_lift}=3.4\ln\Phi\ \ (b_0=-3.356),
$$

$$
\operatorname{logit}P_\text{past\_blow}=3.4\ln\Phi-0.892\ \ (b_0=-4.248;\ 0.892=3.4\ln1.3),\qquad
\operatorname{logit}P_\text{lift\_noted}=0
$$

The scored `lifted` label is the scoring-time rule $P_\text{noted}\,(P_\text{past\_lift}-P_\text{past\_blow})/(1-P_\text{past\_blow})$ (`configs/models/law/slice2/decl.json`). Both labels are scored with Brier, mean of day means.

The flame length is unscored: $\ln L=-3.752+1.3z_x-0.065e_2-0.088e_3+z_m+0.148[\operatorname{relu}(0.511-z_{co}+z_j)-\operatorname{relu}(-2.708-z_{co}+z_j)]$, with $s=0.22$.

#### C.7 SAME-R: `samer/s1-fable` (`sha256:e33ee766…1b5f`)

The physics is smoke size by condensation and Brownian coagulation, plus the ISS detector scatter channel. In W0, $D_{30}=D_n(1+t/t_c)^{1/3}$ with $t_c=1/(KN_0)$ (`research:docs/wsp/research/phys_samer_w0.md:21-32`). S1 sets one $t_c=450$ s for every smoke and $d_{10}\propto(1+t/t_c)^{0.55}$ (`research:docs/wsp/research/phys_samer_study.md:197-205`):

$$
\ln d_{10}=\text{level}[m]+0.20\,\tfrac{T-T_\text{ref}}{50\,\text{K}}+f(U)+\text{no-flow terms}+0.30\ln\tfrac{t_h}{60\,\text{s}}+0.55\operatorname{sp}(y),\quad y=\ln\tfrac{t}{450\,\text{s}}+0.30\tfrac{U-8.1\,\text{cm/s}}{1\,\text{cm/s}}+0.30[U=0]
$$

Features: indicators $k$ (Kapton), $t$ (Teflon), $s$ (silicone), $p$ (Pyrell), with lamp wick as the reference; $z_T=(T-673\,\text{K})/50\,\text{K}$ ([450, 950]); $z_v=(U-0.081)/0.01$ ([0, 0.12] m/s); nf = no flow; $z_a=\ln(t/100\,\text{s})$ ([5, 5000]); $z_h=\ln(t_h/60\,\text{s})$ ([20, 300]). As typed:

$$
\ln\frac{d_{10}}{1\,\text{m}}=-15.412-1.416k-1.133t+0.125s+0.704p+0.2z_T-0.039z_v+0.236\,\text{nf}+0.3z_h
+0.0373\operatorname{relu}(-3.1-z_v)-0.0763\operatorname{relu}(-6.1-z_v)+0.149\operatorname{relu}(k+t+\text{nf}-1)
$$

$$
\qquad+\sum_{j}c_j\operatorname{relu}(b_j+z_a+0.3z_v+0.3\,\text{nf})-0.5\operatorname{relu}(p+\text{nf}-1),\quad b=(0.996,-0.604,-2.404,-4.004),\ c=(0.11,0.165,0.165,0.11)
$$

Since $b_j=\beta_j-\ln4.5$ with $\beta_j=2.5,0.9,-0.9,-2.5$, the four chords put knots at $y=-2.5,-0.9,0.9,2.5$ and sum to slope 0.55. The spread is

$$
s=0.40+0.06\,\text{nf}+0.03\operatorname{relu}(z_a-1),\quad w=0.1,\ k=2.5
$$

The detector, $|V|\ge6.0$ V, is:

$$
\operatorname{logit}P_\text{iss}=3.956-7.57k-7.56t-1.57s-0.47p+0.6z_T-0.1z_v-2.6\,\text{nf}+0.5z_h+1.5\operatorname{relu}(s+\text{nf}-1)+0.3\operatorname{relu}(2k+2t+z_a-3)
$$

Unit 0 of the detector is typed with `w2 = 0`. What is scored: CRPS of $\ln d_{10}$ (trapezoid weights) and the Brier of the detector. $d_{30}$ is typed but not scored.

#### C.8 SAME: `same/w0-fable` (`sha256:e1362b55…2062`), ZS, held whole

Features: $k$, $t$, $s$, dbp (lamp wick is the reference); $z_v=(U-0.081)/0.01$; $z_a=\ln(t/100\,\text{s})$. The growth term is $0.413\ln(1+t/t_c)$ with $t_c$ = 1000 s (lamp wick), 400 (Kapton), 500 (Teflon), 1400 (silicone), 2500 (DBP) (`configs/models/law/same/w0-fable/edits.wsp`, comments):

$$
\ln\frac{d_{10}}{1\,\text{m}}=-15.614-0.586k-0.463t+0.215s+0.718\,\text{dbp}-0.039z_v+0.0373\operatorname{relu}(-3.1-z_v)-0.0763\operatorname{relu}(-6.1-z_v)+\sum_{j}c_j\operatorname{relu}(b_j+\mu)
$$

where $\mu=0.917k+0.693t-0.336s-0.916\,\text{dbp}+z_a$, $b$ = (0.197, −1.403, −3.203, −4.803), and $c$ = (0.0826, 0.124, 0.124, 0.0826). The spread is $s=0.42+0.1\,\text{dbp}+0.03\operatorname{relu}(z_a-1)$, with $w=0.1$ and $k=2.5$. The detector:

$$
\operatorname{logit}P_\text{iss}=1.1-2.83k-3.3t-0.25s+1.1\,\text{dbp}-0.1z_v+0.9\operatorname{relu}(2k+2t-0.164z_v-2)+0.3\operatorname{relu}(2k+2t+z_a-3)
$$

#### C.9 DAFT: `daft/w0-fable` (`sha256:061f93de…5e5d`), ZS, held whole

The physics is what a modified P-Trak reads, by kind of air, from the DustTrak reading. Features: indicators $c$ (cabin), $v$ (Velcro), $b$ (bag dust), with filtered air as the reference; $z_m=\ln(m/10^{-6}\,\text{kg m}^{-3})$ on the floored reading, domain [1e-9, 2e-4]. Each indicator owns a hinge pair $\omega[\operatorname{relu}(12\,\text{ind}+z_m-12)-\operatorname{relu}(12\,\text{ind}-z_m-12)]$. Since $|z_m|<12$ on the domain, that pair equals $\omega z_m$ when ind = 1 and 0 otherwise, so exactly:

$$
\ln\frac{N}{1\,\text{m}^{-3}}=10.82+5.144c+6.828v+9.798b+(0.30c+0.50v+0.95b)\,z_m,\qquad s=1.5-0.6b,\ w=0.15,\ k=2.5
$$

The score is the CRPS of $\ln(1+N\ \text{per cm}^3)$ (`research:tools/wsp/ext/law_score.py:471`).

---

### D. What could not be determined

- **BRE**: why d3 is called "data-informed". Its `meta.json` lists the train and validation partitions as exposure, but its rationale does not use the words zero-shot or data-informed. `COVERAGE_19.md:100` lists d3 among the second laws on development days.
- **CLD radiometer**: the physical constants behind `radiometer.median.b0` = −0.66. The comment derives W0's ln(0.668 V) = −0.403, and D1's step to −0.66 is not explained in the edits file. I did not open `phys_cld_d1.md`.
- **E-FIELD and SAME-R**: I did not reconcile the derivations of every hinge constant (for example −18.255, −37.8 and −38.545 in E-FIELD, and the per-material levels of SAME-R) against their dossiers. They are reported as typed numbers with the author's one-line meaning.
- **SPICE**: I did not derive the separate coefficients 0.858 ($z_x$) and 0.099/0.173/0.188 (fuels) in P3's $\ln L_\text{far}$. They are reported as typed; the dossier gives $c_f$ and the $-0.08$ velocity-ratio exponent.
- **Plastics, v2 coding**: the field `x_o2_initial` used by the Bhattacharjee baseline on v2 rows was not traced into the v2 case files.

## Part 5. How to reproduce a forecast

The forwards are deterministic, so a release run on the same prefix gives the same forecast, bit for
bit. The commands below run from the root of this repository.

```bash
# 1. Build the CLI (toolchain pinned by rust-toolchain.toml; see docs/REPRODUCIBILITY.md)
cargo build --release -p ignis-cli
export PATH="$PWD/target/release:$PATH"

# 2. Rebuild the releases from the committed edits; the ids must equal configs/models/**/release.json
python tools/wsp/reassemble.py --ignis target/release/ignis            # every lineage it lists
python tools/wsp/reassemble.py --only flex_methanol2,acme_normal12      # or a subset, by model-key prefix

# A release folder that reassemble.py does not list (for example the seventh-generation law revisions,
# configs/models/law/<slug>/<candidate>/) is rebuilt one step at a time:
ignis wsp compile  --family spice --edits configs/models/law/spice/p3-fable/edits.wsp \
                   --meta configs/models/law/spice/p3-fable/meta.json --out /tmp/p3.patch.json
ignis wsp assemble --family spice --patch /tmp/p3.patch.json --binding configs/models/law/spice/binding.json
# The parent release must already be in the registry; build the lineage in order (root, w0, p2, p3).

# 3. Read back every typed number of a release (the tables in this document)
ignis wsp inspect --release sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce --edits

# 4. Print the frozen spec and the digest of a family's forward sources
ignis wsp spec --family flex_methanol2
ignis wsp forward-hash --family flex_methanol2

# 5. Run the strict forward on a prefix (the evidence available at the cutoff)
ignis wsp forward --release sha256:55e0cb12a5a07321df89210e32f74b5ea668dacefcea3684698d6c9d511f43ce \
                  --prefix prefix.json --settings '{"horizon_s":10}' --out forecast.json
#   burner flames: --settings '{"times_s":[6,8,10,12,15,20,25,30,40,50,60,80,100,120]}'
```

**Model keys.** The model keys for `--family` are listed in `crates/ignis-wsp/src/models.rs` and
`crates/ignis-wsp/src/law_slugs.rs`. A release finds its forward through its `model_name`, for example
`flex_methanol.wsp2` → `droplet3.rs`.

**Forward tests.** The tests in `crates/ignis-wsp/tests/` exercise each forward. Some of them, such as
`front5_forward.rs` and `front2_forward.rs`, check bit-identity with an earlier forward:

```bash
cargo test -p ignis-wsp
```

**What the scores need beyond this repository.**
- The prefixes and outcomes come from NASA's public archives. They are rebuilt by the data pipeline
  ([../DATA_PIPELINE.md](../DATA_PIPELINE.md)).
- The scoring chains live in the research repository: `tools/wsp/final/`, `tools/wsp/ext/` and
  `crates/ignis-wsp-dev/`.
- Their commands and scores are in [COMPARISONS.md](COMPARISONS.md).
