# ACME burner flames, a committee of laws that can read the planned stop (`acme_normal.wsp5`, `acme_inverse.wsp5`)

**State: experimental, hand-written, data-informed. Both carried since the fifth generation
(14 November 2026).** Forward `committee.rs`. A committee release holds one complete set of numbers
for each of up to six existing hand-written laws (two slots of the hinge-law forward `flame.rs`,
four of the growth-clock forward `flame3.rs`), a weight per slot, a second weight vector used when
the prefix carries the planned stop of the test, and one offset. Its forecast is the weighted mean of
what the member forwards return for the same prefix. Not NASA-trained; no Bayesian posterior is
claimed; uncertainty is typed.

| Model | Candidate | Release | Non-zero / authored numbers |
|---|---|---|---|
| `acme_normal.wsp5` | P1 (carried) | `sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7` | 275 / 2715 |
| | carry of `acme_normal.wsp4` D1 | `sha256:dce4d7f0b3ce75b6908d8be60313ecf7f7b94d0c25aefd79120af08a71f94a35` | 202 / 2715 |
| | baseline set (lineage root, reads no train number) | `sha256:6044b69bd5d10a150734bea2b6d549a8903dbdc49ddc911f7ff1b6bf2c0b2bff` | 134 / 2715 |
| `acme_inverse.wsp5` | P1 (carried) | `sha256:d7292e8974eed0f0ade50df68e14e3b9134c5c41ec3b8ddd664e02db7561fcc2` | 261 / 2715 |
| | carry of `acme_inverse.wsp3` D1 | `sha256:ff3449c4123f0ae5e1434f72b9a031588bb93a39a6514369624a906337035c26` | 162 / 2715 |
| | baseline set | `sha256:3d7d010fd8488d7071e756719f4bf8e7a2fb1f8a768179b063f55ec38d226424` | 134 / 2715 |

Edits: `configs/models/acme/wsp5/{normal,inverse}/p1-fable/edits.wsp`. Almost every statement is the
stored value of a member release, copied with its slot prefix by `tools/wsp/committee_edits.py`,
which chooses and computes nothing. The lines typed for these models are the two weight vectors and
the offset.

## What it computes

```text
P(t) = sum_k w_k P_k(t) / sum_k w_k        for each row of (active, extinction, operator termination, unresolved)

w = mix.w          when the prefix carries no planned stop
w = mix.w_script   when it carries the condition programmed_stop and mix.w_script has a positive entry;
                   every member then receives a scheduled stop of the flow at
                   programmed_stop - script.onset_s on the record's clock
```

## Members and weights

| Model | Slot | Member release | What it is | `mix.w` | `mix.w_script` |
|---|---|---|---|---|---|
| normal | a0 | `acme_normal.wsp2` D3 `sha256:32822270…94d7` | hinge laws in the reduced flow, weak-flame class, burner-limit clock, stop atoms ([card](acme_clock.md)) | 1 | 0 |
| | a1 | the same with its guessed stops switched off `sha256:b1a334f3…02ea` | | 0 | 1 |
| | b0 | `acme_normal.wsp3` D2 `sha256:4fa8de0b…1991` | the growth clock of the published theory with the burner heating law ([card](acme_growth.md)); D2 differs from the D1 of the fourth generation in one statement, the weights of the two burner limits, `bt.limit_w = [0.15, 0.85, 0]` | 1 | 0 |
| | b1 | the same with its guessed stops switched off `sha256:9354b037…6514` | | 0 | 1 |
| inverse | b0 | `acme_inverse.wsp3` D1 `sha256:19c156e6…22eb` | the law carried until now | 2 | 0 |
| | b1 | `acme_inverse.wsp3` D5 `sha256:9bdd4d27…f9bc` | D1 plus one law: in the marginal band of the mixture fraction a flame at a high oxygen flow is held in a limit cycle (four statements) | 1 | 0 |
| | b2, b3 | the same two with their guessed stops switched off `sha256:e6fbed49…be0d`, `sha256:446e5ba7…1f23` | | 0 | 2, 1 |

The guessed stops of a member are the stop atoms it typed from train sessions (campaign policy: many
tests were stopped at 65 to 66 s). When the plan of the test is known they are replaced by it.

## Where the typed numbers come from

- **Weights 1 : 1 (normal)**: the equal weights of `acme_normal.wsp4` D1, typed knowing nothing that
  ranks the two laws.
- **Weights 2 : 1 (inverse)**: the author's probability, one third, that a marginal flame at a high
  oxygen flow is held. Train shows it twice in two flames at 9 mg/s (8.9 % and 14 % ambient fuel, 1.1
  and 2.2 atm) and is silent for a leaner ambient, another fuel or a higher pressure.
- **`bt.limit_w = [0.15, 0.85, 0]`** (normal, slot b0): the burner limit was about 500 °C (trips at
  502 to 522 °C) in every normal-flame train session from March to June and about 450 °C in the one
  session of October. The model before had given the lower limit 0.4, from pooling the inverse-flame
  sessions with the normal ones. A forward cannot see the date of a test, so the weights are the
  share of normal-flame train sessions under each setting: one of seven, six of seven.
- **`script.onset_s = 0.4`**: the stored planned stop is the sum of the script's stage durations; on
  the 29 train tests that ran to the scripted end the flow stopped at that sum minus 0.4 s on the
  record's clock (median; −1.3 to +1.7 s).
- **The planned stop itself** is not a model number. It is read per test from NASA's script parameter
  file by `tools/wsp/ext/acme_script.py` into a private copy of the data root as the condition
  `programmed_stop`. It exists there for 108 normal and 97 inverse tests. The shared records do not
  carry it.

## Scores

Integrated Brier score over 6 to 120 s, extinction / flow termination; lower is better.

| | Train (210 normal, 204 inverse cases) | Validation, planned stop known | Validation, shared records |
|---|---|---|---|
| `acme_normal.wsp5` P1 | 0.0536 / 0.0215 with the stop; 0.0518 / 0.0274 without | **0.1452 / 0.0747** | **0.1552 / 0.0968** |
| carried before (`acme_normal.wsp4` D1) | 0.0517 / 0.0275 | 0.1553 / 0.0984 | 0.1553 / 0.0984 |
| `acme_inverse.wsp5` P1 | 0.0281 / 0.0341 with the stop; 0.0271 / 0.0608 without | **0.1348 / 0.0478** | **0.1288 / 0.0880** |
| carried before (`acme_inverse.wsp3` D1) | 0.0357 / 0.0639 | 0.1606 / 0.0939 | 0.1606 / 0.0939 |

Validation: 84 cases of 28 normal tests, 54 cases of 21 inverse tests, two independent sessions each.
These are development numbers: the partitions had been read as aggregate tables about seven times
before this look, and with two sessions no interval is informative. Adoption followed the rule
written beforehand (replace unless the extinction score is worse), applied with and without the
planned stop ([PROTOCOL.md](../PROTOCOL.md) section 10, [RESULTS_GEN5.md](../RESULTS_GEN5.md)).

## What to distrust

- **Whether the planned stop may be an input at all.** The script file is NASA's "as run" record. On
  train it holds the planned stage durations for 99 of 99 early-ended tests, which is why it is
  classed as known before ignition; a reader who rejects that should use the "shared records"
  column, where the normal-flame release is level with the model before it.
- **Extinction with the planned stop** is slightly worse than without it for inverse flames (0.135
  against 0.129) and for normal flames on train (0.0536 against 0.0518): removing the guessed stops
  also removes probability they had placed, by accident, near real extinctions.
- **The hold scenario** rests on two train flames. Its weight is a judgment.
- **Wrong table inputs**: six normal and three or four inverse train rows have table flows that the
  readbacks contradict; the members read the table.
- Everything the members' own cards distrust: the four corrections of the growth clock that have no
  mechanism, propane in enriched oxygen, the stop atoms.

## Exposure

Data-informed. The author had read the train partition, the aggregate validation tables of every
earlier candidate, and the conditions (inputs) of the validation tests; conditions of consecutive
tests in one chamber fill encode the previous test's outcome (X-18 of the dossier's ledger). No
validation outcome was read per test. The test partition was not read.

## Reproduce

```bash
python tools/wsp/reassemble.py --only acme/wsp5
```

```bash
IGNIS_DATA=runs/dev/data_ext target/release/wsp-dev feedback --release sha256:f71fce5f22f272cc1c336d89e32530f53a21640c91797e6a50e6219f985e69a7 --partition train --times 6,8,10,12,15,20,25,30,40,50,60,80,100,120 --out runs/dev/gen5/card_fb_normal_p1.json
```

Validation tables: section 9.7 of `docs/wsp/dossier/09_reproduce.md` (a validation look; registered
looks only).
