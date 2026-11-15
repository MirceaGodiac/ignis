# FLEX droplets, fourth generation (`flex_methanol.wsp3`, `flex_heptane.wsp3`)

**State: experimental, hand-written, data-informed. Not adopted: negative results.** Forward
`droplet4.rs`. The physics of the third generation ([card](flex_droplet3.md)) stands; this generation
changed what the model does when the tracker cannot be trusted, wrote a law for helium, and sharpened
what looked too soft or too wide on train. All three candidates below passed on train and none
improved on unseen sessions, so **the carried droplet models are still the third generation's**
(`flex_methanol.wsp2` D3, `flex_heptane.wsp2` D2). The third-generation forward, releases and ids
are unchanged.

| Model | Candidate | Release | Non-zero numbers |
|---|---|---|---|
| `flex_methanol.wsp3` | D4 | `sha256:4e975d3b5b034c4529bf62823e45a7356f49490190bfe989cc663c9d1b8e2d02` | 87 / 391 |
| | D5, the broad component alone | `sha256:0b57dba76eda173ffa57c8061cbb17b3873efb931de210ca944ce9e0cadef2c7` | 80 / 391 |
| | carry of D3 | `sha256:33bb9ae6212dac998b9824608e640acd9ac526bf2ac7eba515049bb2bfdea659` | the third-generation numbers, new entries neutral |
| `flex_heptane.wsp3` | D3 | `sha256:d3cf6b7c2569c455329c17009d1d30f055017dcc3f4763677b28d962d3771d6f` | 107 / 588 |
| | D4, the broad component alone | `sha256:b9a121a1a71109b90273900283f6626a38ced7058eb271f1f0cdd67d74d964cc` | 104 / 588; failed the train rule, never taken to validation |
| | carry of D2 | `sha256:bb12a888795604dd0c09951562b90f0c387c6e43bf0b4b567df84b51480e5c3a` | the third-generation numbers, new entries neutral |

The carry releases exist because this forward reads one more prospective condition (the sheet's
initial diameter), so its cases cannot be paired with older releases by the evaluator. A carry gives
the third generation's forecasts on the new forward (identical on every train case) and is what the
candidate is compared with.

## What changed in the forward

**1. A broad component when the prefix shows an unstable tracker.** 39 % of the methanol train error
and 57 % of the heptane train error is tracker artefact: a soot shell read as the droplet, a flame
hiding it, a bubble, a droplet leaving and re-entering the view. No law forecasts those values. Four
signals in the prefix say that the tracker is in trouble, and when one fires the forecast is mixed
with a band 0.3–0.4 mm wide that grows by 0.03 mm per second.

| Signal | Fires when | Train: what it catches | Weight (methanol / heptane) |
|---|---|---|---|
| size | the prefix's own D0 is more than 10 % from the sheet's | a soot shell (+21 %), a droplet hidden by the flame (−47 %), a bubble (+26 %); nothing else beyond 6 % | 0.7 / 0.8 |
| gated | three or more frames dropped by the validity gate after 0.2 s | methanol: a bubble, a droplet re-entering the view, nothing else. Heptane: fifteen cases, twelve of them forecast well | 0.7 / 0.15 |
| growth | D² rises by more than 3 % between 0.2 s windows | the above, saturated backlight, one run at 2 atm in 70 % CO2; two false alarms | 0.3 / 0.35 |
| rate | the end rate is outside −1.5 to 3 times the law | a collapsing soot shell (97 times the law), three already caught; one false alarm | 0.7 / 0.7 |

Not visible in any prefix: a droplet that leaves the view later, values frozen after a disruption, a
video filed under the wrong atmosphere. The droplet position would show the first; it is in NASA's
files but not in the stored observations, and adding it would change every FLEX prefix of the
shared dataset, so this generation does without it.

**2. What follows a flame lost in helium** (heptane). After the visible flame is lost in 30–35 %
helium the droplet keeps shrinking at 0.83 of the hot rate, falling by e every 3.4 s (three train
runs, one session). Published work reads this as a cool flame at 0.8–0.87 of the hot rate, transient
at high helium dilution. The no-flame law's hot term is multiplied by `exp(3.85 x_He)` after such a
loss (3.2 times at 30 %, 3.8 times at 35 %), and the radiative clock runs 0.18 faster per 20 % of
helium. No train run tests 5–25 % helium.

## What changed in the numbers

| Number | Third generation | Fourth | From |
|---|---|---|---|
| Methanol: radiative class at the critical diameter | slope 10, 0.67 at D_crit | slope 25, boundary 7 % below D_crit | five train droplets within −2 to +9 % of D_crit all went out radiatively; two at −10 and −20 % did not |
| Methanol: heat-up at 1 atm | as at 0.7 atm | 15 % slower at ignition, fading by 0.2 clock units | 7 of 11 sessions at 1 atm below the law, none above |
| Measurement band | 20 µm / 25 µm | 8 µm / 10 µm | frame scatter 1.5 µm; prefix and continuation share the tracker |
| Rate spread before / after a prefix | 0.10 / 0.06 | 0.08 / 0.05 (methanol), 0.07 / 0.04 (heptane) | plateau scatter 0.05 (methanol), realised rate error 0.026 (heptane); a margin for unseen sessions stays |
| Heptane: diffusive extinction sharpness | 10 | 15 | six burn-downs end within ±0.05 of the written progress |
| Heptane: cool-flame offset gain | 0.5 | 0.5 (0.2 was tried and withdrawn) | the law alone is better in 9 of 13 cases, but the offset is what corrects the law at 2 atm, 0.7 atm and 14 % oxygen |

## Results

Selection was made on train by the session rule of [PROTOCOL.md](../PROTOCOL.md) section 8 and the
candidates were committed (`f1f7a30`) before any validation score of them existed.

| | Train mean / median CRPS | Train sessions better or level | Train coverage 50 % / 90 % |
|---|---|---|---|
| Methanol D4 | 26.9 / 16.4 µm | 24 of 26 | 0.67 / 0.93 |
| Methanol D3 (carry) | 29.9 / 17.7 µm | | 0.77 / 0.94 |
| Heptane D3 | 39.6 / 19.1 µm | 18 of 20 | 0.62 / 0.91 |
| Heptane D2 (carry) | 46.2 / 21.8 µm | | 0.76 / 0.93 |

**Validation** (aggregate tables; the candidates and the carry on the same cases):

| | CRPS | Coverage 50 % / 90 % | Paired, carry minus candidate | Decision |
|---|---|---|---|---|
| Methanol D4 | 44.8 µm | 0.61 / 0.82 | −5.3 [−15.4, +1.4] µm | not adopted |
| Methanol D5 (second look, registered) | 40.1 µm | 0.72 / 0.90 | | not adopted: 0.6 µm above the carry |
| Methanol D3 (carry) | 39.5 µm | 0.71 / 0.90 | | carried |
| Heptane D3 | 57.6 µm | 0.50 / 0.87 | −11.6 [−33.0, +0.9] µm | not adopted |
| Heptane D2 (carry) | 46.1 µm | 0.71 / 0.97 | | carried |

59 methanol cases in 9 sessions; the intervals are a bootstrap over sessions.

## What this says

- **Sharpening from train does not transfer.** D4 and the heptane D3 made a class boundary steeper
  and the bands narrower because five to thirteen train runs looked that way. On unseen sessions the
  90 % band then covers 0.82 and 0.87, and the median is worse too. What a band has to cover on a
  new session is the error of the law there, not the scatter of the tracker.
- **The tracker flags are a train phenomenon.** D5 changes nothing but the band of flagged prefixes.
  It is better or level in 25 of 26 train sessions and 0.6 µm worse on validation: the artefacts it
  was typed for (a soot shell, a bubble, a droplet re-entering the view) are particular runs, and a
  flag raised on a healthy validation prefix only costs.
- **The helium law is unbounded.** `exp(3.85 x_He)` was typed from 30–35 % helium; nothing holds it
  below the hot rate at higher fractions. That needed no validation to see.

The forward stays in the tree with its tests: the candidates are reproducible, and the carry
releases show that the third-generation numbers give identical forecasts on it.

## Limits

- The broad component widens a band; it does not move the median. In a flagged case the forecast is
  still wrong, and says so.
- The size signal needs the sheet's D0 as a prospective condition. If a prefix does not carry it the
  signal is off, never a refusal.
- Helium: one session, 30–35 % only.
- The laws are the third generation's; its limits (the helium clock for methanol from three runs, the
  radiative clock above D_crit from CO2-diluted pairs, no pressure above 1 atm for methanol) stand.
- Near the critical diameter, at the end of a cool flame and at a disruption the outcome is not
  decided by anything a diameter prefix holds; the model gives a mixture there.
