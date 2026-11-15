# Train-data analysis tools of the WSP lane

Read-only measurement scripts used while studying the experiments (docs/wsp/EXPERIMENTAL_SCIENCE.md).
They print tables; none of them fits, searches or writes a model parameter. `lib.py` returns run ids
of the train or validation partition only; the test partition cannot be requested.

| Script | Output |
|---|---|
| `flex_sheets.py <flex\|flex_heptane> out.md` | NASA test sheets of the train runs: conditions, kavg, Dext, narrative |
| `flex_profile.py <split> out.json` + `flex_profile_print.py out.json [main\|early]` | per train run: windowed finite-difference rates of D^2, knee, gaps, early-frame artefacts |
| `flex_series.py <split> t0 t1 step <run> …` | raw droplet series of train runs at reduced resolution |
| `neardup.py` | one video analysed under two test ids (position tracks compared; nothing printed for runs outside train) |
| `acme_table.py <acme_normal\|acme_inverse> out.md` | train rows: conditions, endpoint, post-run facts, comments, control timelines |
| `droplet_compare.py feedback.json profile.json [release-index] [crps\|cond]` | per train run: a droplet release's laws and state filter next to what the run did |
| `fb_summary.py feedback.json …` | mean, median and trimmed CRPS and coverage per release |
| `flex_heptane_train_rates.py main` / `series t0 t1 step <run> …` | n-heptane train runs: sheet burn time and extinction diameter, the clock at the end of the hot flame, windowed rates before and after it (the cool flame) |
| `partition_aggregate.py feedback.json <split> A B [--skip run,…]` | two releases of one `wsp-dev feedback` file on a development partition with named runs left out: aggregates and session counts only, no per-case or per-session value |
| `inverse_separation.py` | inverse train flames alive at 5 s: for each input, the range where both outcomes occur and how many flames lie in it; the train tests inside the mixture-fraction band |

Run from the repository root, e.g. `python tools/wsp/analysis/flex_profile.py flex runs/dev/analysis/flex_profile.json`.
