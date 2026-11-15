# Which investigations serve which habitat

Built by `tools/wsp/atlas/build_cards.py` from the cards of this folder. The ranking is a rule, not a similarity search:
investigations are ordered by N, then by the coverage score C = 2 A + G + V + D + P.

| Part | Weight | What it measures, and why it has that weight |
|---|---|---|
| N | ranks first | need: 3 when the brief names the investigation for the need, 2 when NASA's objective states it, 1 when the investigation is foundational for it. It is NASA's statement, not this project's, so nothing computed here overrides it. |
| A | 2 | atmosphere: 3 when five or more tests lie inside the habitat's oxygen and pressure, 2 when one to four do or NASA's objective says the atmosphere is examined, 1 when only the oxygen matches, 0 otherwise. Weight 2: flammability depends strongly on oxygen, and a habitat is defined by its atmosphere. |
| G | 1 | gravity: for a microgravity cabin every flight investigation counts 2. For a surface habitat PSI holds no partial-gravity data, so every investigation counts 0, or 1 when it offers a normal-gravity comparison or mimics partial gravity. |
| V | 1 | ventilation: 1 when an imposed flow is part of the tests. Air flow matters in microgravity; crew cabins are ventilated. |
| D | 1 | data: 2 when 50 or more tests have a sensor series or processed data, 1 when any has, 0 when only images or video exist. |
| P | 1 | predictive support: 2 when a hand-written model of this project is scored on the investigation, 1 when one was tested on it. |

Limits:

- The ranking orders what PSI holds for a need and a habitat; it does not say that a habitat is safe.
- Need tiers and the flags for gravity and flow are typed judgements, each with the sentence of NASA's it rests on.
- Atmosphere counts use the test catalogue, where many tables give no pressure per test; a count of zero can mean unknown.
- No test in PSI lies at the spacesuit atmosphere, and none at partial gravity.

## Spacecraft and space stations at an Earth-like atmosphere (the ISS)

Atmosphere: oxygen 0.20 to 0.22, 14.7 ± 0.5 psi (tolerances of the test catalogue). Gravity: microgravity. From the brief: "the Earth's pressure at sea level is 14.7 psi"

### Prevention

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | ACME BRE (PSI-20) | named in the challenge brief | 8 | 2 | 2 | 0 | 2 | 0 | NASA's objective says this atmosphere is examined; the table gives no pressure per test; 187 tests with a sensor series or processed data |
| 2 | BASS-II (PSI-25) | named in the challenge brief | 7 | 1 | 2 | 1 | 0 | 2 | 6 tests match the oxygen; pressure unknown or outside; images or video only: values have to be measured first |
| 3 | SAFFIRE-I (PSI-98) | named in the challenge brief | 6 | 1 | 2 | 1 | 1 | 0 | 1 tests match the oxygen; pressure unknown or outside; 2 tests with a sensor series or processed data |
| 4 | SAFFIRE-II (PSI-99) | named in the challenge brief | 6 | 1 | 2 | 1 | 1 | 0 | 1 tests match the oxygen; pressure unknown or outside; 9 tests with a sensor series or processed data |
| 5 | SAFFIRE-III (PSI-100) | named in the challenge brief | 4 | 0 | 2 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 2 tests with a sensor series or processed data |
| 6 | BASS (PSI-26) | named in the challenge brief | 3 | 0 | 2 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 7 | FLEX (PSI-69) | stated in NASA's objective for the investigation | 12 | 3 | 2 | 0 | 2 | 2 | 35 tests inside this atmosphere; 278 tests with a sensor series or processed data |
| 8 | ACME CFI-G (PSI-159) | foundational | 12 | 3 | 2 | 0 | 2 | 2 | 14 tests inside this atmosphere; 337 tests with a sensor series or processed data |
| 9 | ACME/Flame Design (PSI-10) | foundational | 11 | 3 | 2 | 0 | 2 | 1 | 64 tests inside this atmosphere; 317 tests with a sensor series or processed data |
| 10 | CFI (PSI-39) | foundational | 10 | 3 | 2 | 0 | 2 | 0 | 63 tests inside this atmosphere; 123 tests with a sensor series or processed data |
| 11 | ACME s-Flame (PSI-23) | foundational | 10 | 3 | 2 | 0 | 2 | 0 | 10 tests inside this atmosphere; 286 tests with a sensor series or processed data |
| 12 | FLEX-2 (PSI-68) | foundational | 4 | 0 | 2 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 586 tests with a sensor series or processed data |
| 13 | SLICE (PSI-106) | foundational | 4 | 0 | 2 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 13 tests with a sensor series or processed data |

### Detection

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | SAME-R (PSI-101) | named in the challenge brief | 5 | 0 | 2 | 1 | 2 | 0 | no test at this atmosphere in the catalogue; 66 tests with a sensor series or processed data |
| 2 | SAME (PSI-102) | named in the challenge brief | 4 | 0 | 2 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 42 tests with a sensor series or processed data |
| 3 | BASS-II (PSI-25) | stated in NASA's objective for the investigation | 7 | 1 | 2 | 1 | 0 | 2 | 6 tests match the oxygen; pressure unknown or outside; images or video only: values have to be measured first |
| 4 | DAFT / DAFT-2 (PSI-47) | stated in NASA's objective for the investigation | 3 | 0 | 2 | 0 | 1 | 0 | no test at this atmosphere in the catalogue; 6 tests with a sensor series or processed data |
| 5 | SPICE (PSI-107) | foundational | 3 | 0 | 2 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |

### Suppression

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | FLEX (PSI-69) | named in the challenge brief | 12 | 3 | 2 | 0 | 2 | 2 | 35 tests inside this atmosphere; 278 tests with a sensor series or processed data |
| 2 | BASS-II (PSI-25) | stated in NASA's objective for the investigation | 7 | 1 | 2 | 1 | 0 | 2 | 6 tests match the oxygen; pressure unknown or outside; images or video only: values have to be measured first |
| 3 | BASS (PSI-26) | stated in NASA's objective for the investigation | 3 | 0 | 2 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 4 | CFI (PSI-39) | foundational | 10 | 3 | 2 | 0 | 2 | 0 | 63 tests inside this atmosphere; 123 tests with a sensor series or processed data |
| 5 | ACME s-Flame (PSI-23) | foundational | 10 | 3 | 2 | 0 | 2 | 0 | 10 tests inside this atmosphere; 286 tests with a sensor series or processed data |
| 6 | ACME CLD Flame (PSI-21) | foundational | 5 | 0 | 2 | 1 | 2 | 0 | no test at this atmosphere in the catalogue; 352 tests with a sensor series or processed data |
| 7 | FLEX-2 (PSI-68) | foundational | 4 | 0 | 2 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 586 tests with a sensor series or processed data |

## Moon and Mars surface bases and pressurized rovers

Atmosphere: oxygen 0.30 or more, 10.2 psi or less. Gravity: partial gravity. From the brief: "The intermediate oxygen concentration used on the Moon will likely be 30% or greater, with a corresponding pressure of 10.2 psi or less." The brief states this atmosphere for the Moon Base and pressurized rovers; it gives none for a Mars base, which is listed here for its gravity.

### Prevention

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | ACME BRE (PSI-20) | named in the challenge brief | 6 | 2 | 0 | 0 | 2 | 0 | NASA's objective says this atmosphere is examined; the table gives no pressure per test; 187 tests with a sensor series or processed data |
| 2 | BASS-II (PSI-25) | named in the challenge brief | 4 | 0 | 1 | 1 | 0 | 2 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 3 | SAFFIRE-I (PSI-98) | named in the challenge brief | 3 | 0 | 1 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 2 tests with a sensor series or processed data |
| 4 | SAFFIRE-II (PSI-99) | named in the challenge brief | 3 | 0 | 1 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 9 tests with a sensor series or processed data |
| 5 | SAFFIRE-III (PSI-100) | named in the challenge brief | 3 | 0 | 1 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 2 tests with a sensor series or processed data |
| 6 | BASS (PSI-26) | named in the challenge brief | 2 | 0 | 1 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 7 | FLEX (PSI-69) | stated in NASA's objective for the investigation | 4 | 0 | 0 | 0 | 2 | 2 | no test at this atmosphere in the catalogue; 278 tests with a sensor series or processed data |
| 8 | ACME CFI-G (PSI-159) | foundational | 10 | 3 | 0 | 0 | 2 | 2 | 17 tests inside this atmosphere; 337 tests with a sensor series or processed data |
| 9 | ACME/Flame Design (PSI-10) | foundational | 3 | 0 | 0 | 0 | 2 | 1 | no test at this atmosphere in the catalogue; 317 tests with a sensor series or processed data |
| 10 | SLICE (PSI-106) | foundational | 3 | 0 | 1 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 13 tests with a sensor series or processed data |
| 11 | FLEX-2 (PSI-68) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 586 tests with a sensor series or processed data |
| 12 | CFI (PSI-39) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 123 tests with a sensor series or processed data |
| 13 | ACME s-Flame (PSI-23) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 286 tests with a sensor series or processed data |

### Detection

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | SAME-R (PSI-101) | named in the challenge brief | 3 | 0 | 0 | 1 | 2 | 0 | no test at this atmosphere in the catalogue; 66 tests with a sensor series or processed data |
| 2 | SAME (PSI-102) | named in the challenge brief | 2 | 0 | 0 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 42 tests with a sensor series or processed data |
| 3 | BASS-II (PSI-25) | stated in NASA's objective for the investigation | 4 | 0 | 1 | 1 | 0 | 2 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 4 | DAFT / DAFT-2 (PSI-47) | stated in NASA's objective for the investigation | 1 | 0 | 0 | 0 | 1 | 0 | no test at this atmosphere in the catalogue; 6 tests with a sensor series or processed data |
| 5 | SPICE (PSI-107) | foundational | 2 | 0 | 1 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |

### Suppression

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | FLEX (PSI-69) | named in the challenge brief | 4 | 0 | 0 | 0 | 2 | 2 | no test at this atmosphere in the catalogue; 278 tests with a sensor series or processed data |
| 2 | BASS-II (PSI-25) | stated in NASA's objective for the investigation | 4 | 0 | 1 | 1 | 0 | 2 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 3 | BASS (PSI-26) | stated in NASA's objective for the investigation | 2 | 0 | 1 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 4 | ACME CLD Flame (PSI-21) | foundational | 3 | 0 | 0 | 1 | 2 | 0 | no test at this atmosphere in the catalogue; 352 tests with a sensor series or processed data |
| 5 | FLEX-2 (PSI-68) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 586 tests with a sensor series or processed data |
| 6 | CFI (PSI-39) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 123 tests with a sensor series or processed data |
| 7 | ACME s-Flame (PSI-23) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 286 tests with a sensor series or processed data |

## Spacesuit

Atmosphere: oxygen 0.99 or more, 4.3 ± 0.5 psi. Gravity: any. From the brief: "spacesuits are filled with 100% oxygen to only 4.3 psi"

### Prevention

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | BASS-II (PSI-25) | named in the challenge brief | 3 | 0 | 0 | 1 | 0 | 2 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 2 | ACME BRE (PSI-20) | named in the challenge brief | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 187 tests with a sensor series or processed data |
| 3 | SAFFIRE-I (PSI-98) | named in the challenge brief | 2 | 0 | 0 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 2 tests with a sensor series or processed data |
| 4 | SAFFIRE-II (PSI-99) | named in the challenge brief | 2 | 0 | 0 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 9 tests with a sensor series or processed data |
| 5 | SAFFIRE-III (PSI-100) | named in the challenge brief | 2 | 0 | 0 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 2 tests with a sensor series or processed data |
| 6 | BASS (PSI-26) | named in the challenge brief | 1 | 0 | 0 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 7 | FLEX (PSI-69) | stated in NASA's objective for the investigation | 4 | 0 | 0 | 0 | 2 | 2 | no test at this atmosphere in the catalogue; 278 tests with a sensor series or processed data |
| 8 | ACME CFI-G (PSI-159) | foundational | 4 | 0 | 0 | 0 | 2 | 2 | no test at this atmosphere in the catalogue; 337 tests with a sensor series or processed data |
| 9 | ACME/Flame Design (PSI-10) | foundational | 3 | 0 | 0 | 0 | 2 | 1 | no test at this atmosphere in the catalogue; 317 tests with a sensor series or processed data |
| 10 | FLEX-2 (PSI-68) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 586 tests with a sensor series or processed data |
| 11 | CFI (PSI-39) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 123 tests with a sensor series or processed data |
| 12 | ACME s-Flame (PSI-23) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 286 tests with a sensor series or processed data |
| 13 | SLICE (PSI-106) | foundational | 2 | 0 | 0 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 13 tests with a sensor series or processed data |

### Detection

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | SAME-R (PSI-101) | named in the challenge brief | 3 | 0 | 0 | 1 | 2 | 0 | no test at this atmosphere in the catalogue; 66 tests with a sensor series or processed data |
| 2 | SAME (PSI-102) | named in the challenge brief | 2 | 0 | 0 | 1 | 1 | 0 | no test at this atmosphere in the catalogue; 42 tests with a sensor series or processed data |
| 3 | BASS-II (PSI-25) | stated in NASA's objective for the investigation | 3 | 0 | 0 | 1 | 0 | 2 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 4 | DAFT / DAFT-2 (PSI-47) | stated in NASA's objective for the investigation | 1 | 0 | 0 | 0 | 1 | 0 | no test at this atmosphere in the catalogue; 6 tests with a sensor series or processed data |
| 5 | SPICE (PSI-107) | foundational | 1 | 0 | 0 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |

### Suppression

| Rank | Investigation | Tie to the need | Coverage | A | G | V | D | P | Why |
|---|---|---|---|---|---|---|---|---|---|
| 1 | FLEX (PSI-69) | named in the challenge brief | 4 | 0 | 0 | 0 | 2 | 2 | no test at this atmosphere in the catalogue; 278 tests with a sensor series or processed data |
| 2 | BASS-II (PSI-25) | stated in NASA's objective for the investigation | 3 | 0 | 0 | 1 | 0 | 2 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 3 | BASS (PSI-26) | stated in NASA's objective for the investigation | 1 | 0 | 0 | 1 | 0 | 0 | no test at this atmosphere in the catalogue; images or video only: values have to be measured first |
| 4 | ACME CLD Flame (PSI-21) | foundational | 3 | 0 | 0 | 1 | 2 | 0 | no test at this atmosphere in the catalogue; 352 tests with a sensor series or processed data |
| 5 | FLEX-2 (PSI-68) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 586 tests with a sensor series or processed data |
| 6 | CFI (PSI-39) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 123 tests with a sensor series or processed data |
| 7 | ACME s-Flame (PSI-23) | foundational | 2 | 0 | 0 | 0 | 2 | 0 | no test at this atmosphere in the catalogue; 286 tests with a sensor series or processed data |

